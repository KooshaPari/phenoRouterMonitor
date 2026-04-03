# phenotype-infrakit Architecture

## Overview

phenotype-infrakit is a Rust workspace of four independent infrastructure crates extracted from
the Phenotype ecosystem. Each crate is domain-agnostic, has no inter-crate dependencies, and can
be consumed individually.

## Workspace Structure

```
phenotype-infrakit/
  Cargo.toml                         # Workspace root (resolver = "2")
  crates/
    phenotype-event-sourcing/        # Append-only event log with hash chains
    phenotype-cache-adapter/         # Two-tier LRU + DashMap cache with TTL
    phenotype-policy-engine/         # Rule-based policy evaluator with TOML config
    phenotype-state-machine/         # Generic FSM with transition guards
```

## Crate Responsibilities

### phenotype-event-sourcing

Provides an append-only event store backed by a SHA-256 hash chain for integrity verification.

Key types:
- `EventEnvelope<T>` -- wraps any serializable event with metadata (seq, timestamp, hash)
- `EventStore` -- trait for pluggable storage backends
- `InMemoryEventStore` -- reference implementation (in-memory, hash-verified)
- `SnapshotStore` -- optional snapshot support for aggregate rebuilding

Design decisions:
- Hash chain: each event's hash covers `sha256(prev_hash || payload_json)`.
  An empty store uses a zero-hash genesis value.
- Snapshots are stored separately and do not affect the event hash chain.

### phenotype-cache-adapter

A generic two-tier cache combining an LRU L1 layer (bounded, fast eviction) and a DashMap L2
layer (unbounded, concurrent). Entries carry per-key TTL tracked via `Instant`.

Key types:
- `TieredCache<K, V>` -- main cache handle, clone-cheap (Arc-backed)
- `MetricsHook` -- trait for plugging in hit/miss/eviction counters
- `CacheEntry<V>` -- internal value wrapper with expiry timestamp

Design decisions:
- L1 uses `lru::LruCache` under a `parking_lot::Mutex` for bounded eviction.
- L2 uses `DashMap` for concurrent reads without a global lock.
- Writes go to both tiers. Reads check L1 first; on miss, promote from L2 to L1.
- TTL is checked lazily on read; expired entries return `None` and are evicted.

### phenotype-policy-engine

A rule-based policy engine that evaluates a `PolicyContext` (key-value map) against a set of
`Rule` objects. Rules support allow, deny, and require semantics with optional severity levels.

Key types:
- `PolicyEngine` -- holds the rule set; evaluates contexts
- `PolicyContext` -- key-value bag passed to `evaluate`
- `Rule` -- allow/deny/require rule with field matchers and optional regex
- `PolicyResult` -- outcome: passed, list of violations
- TOML loader -- `engine.load_toml_str(&str)` for declarative rule files

Design decisions:
- Rules are evaluated in declaration order; first matching deny wins.
- Require rules fail if the specified field is absent or empty.
- TOML config uses `[[rules]]` arrays for human-editable policy files.

### phenotype-state-machine

A generic finite state machine (FSM) parameterized over a user-defined state enum implementing
the `State` trait. Supports transition guards, forward-only enforcement (via `ordinal()`), skip
states, and full history tracking.

Key types:
- `State` -- trait requiring `ordinal() -> u32`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`
- `StateMachine<S>` -- FSM handle; holds current state and history
- `TransitionGuard` -- closure-based guard attached to specific transitions
- `TransitionError` -- typed error for invalid transitions, guard failures, etc.

Design decisions:
- `ordinal()` enforces forward-only progression; transitions to lower ordinals are rejected
  unless the transition is explicitly configured as a skip or the guard overrides it.
- History is stored as `Vec<S>` in insertion order (oldest first).
- Guards are keyed by `(from, to)` state pairs and receive a `&PolicyContext`-style bag.

## Dependency Graph

```
phenotype-event-sourcing   --depends-on-->  serde, serde_json, chrono, sha2, thiserror, hex, uuid
phenotype-cache-adapter    --depends-on-->  serde, dashmap, lru, parking_lot, moka
phenotype-policy-engine    --depends-on-->  serde, serde_json, thiserror, dashmap, toml, regex
phenotype-state-machine    --depends-on-->  serde, serde_json, chrono, thiserror

Inter-crate dependencies: NONE
```

## CI

The repository uses a single GitHub Actions workflow (`.github/workflows/ci.yml`) that runs on
every push to `main` and on every pull request targeting `main`.

Steps in order:
1. `cargo fmt --all -- --check` -- format gate (no diff allowed)
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings` -- lint gate
3. `cargo build --workspace --all-targets` -- build gate
4. `cargo test --workspace --all-targets` -- test gate (all 76 tests must pass)

Toolchain: stable Rust via `dtolnay/rust-toolchain@stable`.
Caching: cargo registry and `target/` keyed on `Cargo.lock` hash.
