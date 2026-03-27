# Architecture Decision Records — phenotype-infrakit

**Version:** 1.1.0

---

## ADR-001: Independent Crates with No Cross-Crate Source Dependencies

**Status:** Accepted
**Context:** Infrastructure crates serve different consumers: some services need event sourcing,
others only need caching, others only the policy engine. Forcing a single mega-crate bloats
compile times and dependency trees for consumers who only need one feature.
**Decision:** Each crate (`phenotype-contracts`, `phenotype-event-sourcing`,
`phenotype-cache-adapter`, `phenotype-policy-engine`, `phenotype-state-machine`) is
self-contained with zero imports from sibling workspace crates at the source level.
Workspace-level `[workspace.dependencies]` pins all shared transitive deps to consistent
semver ranges.
**Consequences:**
- Consumers add exactly the crates they need; no transitive bloat.
- Each crate compiles independently in CI; no cascading rebuild on unrelated changes.
- Shared contracts require duplication or a separate `phenotype-contracts` import; current
  design keeps `phenotype-contracts` as the one allowed cross-crate dependency (ports only).
**Alternatives considered:** Single `phenotype-infrakit` crate with feature flags (harder to
reason about; feature combinations create hidden coupling).
**Code location:** `Cargo.toml` workspace definition; each crate's `Cargo.toml`.

---

## ADR-002: SHA-256 Hash Chain for Event Integrity

**Status:** Accepted
**Context:** Event sourcing stores immutable domain events. Without integrity verification,
a compromised or buggy store could silently corrupt historical events. The system must detect
tampering deterministically.
**Decision:** Each `EventEnvelope` stores a `hash` (SHA-256 of the current event fields +
`prev_hash`) and `prev_hash` (hash of the preceding event, or 64 zero hex chars for genesis).
`verify_chain()` walks the sequence and recomputes each hash, failing on first mismatch.
Hash input construction order is canonicalized: UUID bytes, timestamp, event\_type, JSON payload,
actor, prev\_hash (all length-prefixed with big-endian u32).
**Consequences:**
- Any post-hoc modification to a stored event breaks the chain at that sequence number.
- Hash computation adds ~microseconds per append; acceptable for event sourcing workloads.
- Canonical hash input format must never change without a migration strategy.
**Alternatives considered:** HMAC with shared secret (requires secret management);
Merkle tree (more complex, not needed for sequential log); no integrity check (unacceptable).
**Code location:** `crates/phenotype-event-sourcing/src/hash.rs`,
`crates/phenotype-event-sourcing/src/store.rs`

---

## ADR-003: TOML for Policy Rule Configuration

**Status:** Accepted
**Context:** Policy rules need to be authored by operators, reviewed in code, and loaded
programmatically. The format must be human-readable and structurally validatable.
**Decision:** TOML files define `Policy` documents with `name`, `description`, `enabled`, and
`rules` arrays. The `loader` module in `phenotype-policy-engine` reads TOML strings or files.
Rule fields: `name`, `description`, `rule_type` (allow/deny/require), `field`, `pattern`,
`severity`.
**Consequences:**
- TOML is diffable and readable in PRs.
- Invalid TOML or missing required fields fail loudly at load time.
- TOML arrays are ordered; rule evaluation order matches declaration order.
**Alternatives considered:** JSON (no comments; harder to author manually); YAML (ambiguous
type coercion; less idiomatic for Rust); HCL (not standard in Rust ecosystem).
**Code location:** `crates/phenotype-policy-engine/src/loader.rs`,
`crates/phenotype-policy-engine/src/policy.rs`

---

## ADR-004: Forward-Only State Machine with Guard Callbacks

**Status:** Accepted
**Context:** Workflow state machines in agent orchestration systems must prevent invalid
regressions (e.g., a completed task reverting to pending) while still allowing some controlled
skip-ahead transitions.
**Decision:** `StateMachine<S>` enforces forward-only transitions by comparing state ordinal
values; transitions to lower ordinals are rejected. Guard callbacks `(from: S, to: S) -> bool`
run before each transition and can reject it. An optional skip-state config allows specific
non-sequential jumps.
Full transition history is maintained for audit and replay.
**Consequences:**
- Accidental regression to earlier states is prevented at the type level.
- Guards enable domain-specific pre-conditions (e.g., "cannot complete without required fields").
- Skip-state config must be explicitly declared; no implicit jumps.
**Alternatives considered:** Typestate pattern (compile-time safety but inflexible for dynamic
workflows); pure enum matching with no enforcement (too permissive).
**Code location:** `crates/phenotype-state-machine/src/lib.rs`

---

## ADR-005: DashMap for Thread-Safe Policy Engine State

**Status:** Accepted
**Context:** The `PolicyEngine` is designed for use in async/multi-threaded Phenotype services.
Policy evaluation must not block on writer locks for unrelated reads.
**Decision:** `PolicyEngine` backs its policy registry with `DashMap<String, Policy>` (sharded
concurrent hash map). Reads and writes are lock-free at the shard level.
`PolicyEngine` is intentionally not wrapped in `Arc`/`Mutex`; callers manage sharing.
**Consequences:**
- Concurrent `evaluate_all` and `add_policy` calls do not contend.
- `DashMap` is a well-maintained crate; wraps `parking_lot::RwLock` per shard internally.
- Policy iteration (for `evaluate_all`) holds read locks per shard briefly.
**Alternatives considered:** `RwLock<HashMap>` (global lock; contention under load);
`Arc<Mutex<HashMap>>` (same problem); `flurry` HashMap (less ergonomic API).
**Code location:** `crates/phenotype-policy-engine/src/engine.rs`

---

## ADR-006: Two-Tier Cache with LRU + moka/DashMap

**Status:** Accepted
**Context:** Service caches benefit from an in-process hot tier (small, fast, single-shard LRU)
backed by a larger concurrent tier (multi-shard for high concurrency). TTL expiration is
required for correctness.
**Decision:** L1 is backed by the `lru` crate (small, bounded LRU). L2 is backed by `moka`
(sync feature, TTL-aware) or `dashmap` for a simpler concurrent map. On L1 miss, L2 is checked;
L2 hits are backfilled to L1. An optional `MetricsHook` trait object receives hit/miss events.
**Consequences:**
- Hot data stays in L1 for O(1) access without lock contention.
- `moka` provides native TTL eviction; `dashmap` requires manual TTL check on read.
- `MetricsHook` decouples cache observability from any specific metrics backend.
**Alternatives considered:** Redis-based cache (external I/O; overkill for in-process caching);
`cached` crate (too opinionated, limited TTL support); single-tier DashMap (no LRU eviction).
**Code location:** `crates/phenotype-cache-adapter/src/lib.rs`

---

## ADR-007: Hexagonal Ports in a Shared `phenotype-contracts` Crate

**Status:** Accepted
**Context:** Hexagonal architecture requires domain logic to depend on port traits (interfaces)
rather than concrete adapter implementations. Without a shared crate, each consumer would
re-declare the same port traits, creating drift.
**Decision:** `phenotype-contracts` defines all inbound and outbound port traits
(`Repository`, `CachePort`, `SecretPort`, `EventBus`) plus domain model base traits
(`Entity`, `ValueObject`, `AggregateRoot`). Adapter crates implement these traits.
Domain crates import `phenotype-contracts` only.
**Consequences:**
- Single source of truth for port contracts across the workspace.
- Domain logic is independent of adapter implementations; swap adapters at composition root.
- Adding a port requires a version bump to `phenotype-contracts`.
**Code location:** `crates/phenotype-contracts/src/ports/`,
`crates/phenotype-contracts/src/models/`
