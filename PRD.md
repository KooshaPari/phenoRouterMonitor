# PRD — phenotype-infrakit

## Overview

`phenotype-infrakit` is a Rust workspace of five infrastructure-layer crates implementing the driven ports (adapters) for the Phenotype hexagonal architecture. It provides event sourcing with SHA-256 hash-chain integrity, two-tier caching, rule-based policy evaluation, finite state machines, and shared hexagonal contracts (ports and domain models). All crates are independently consumable with no cross-crate workspace dependencies.

**Stack**: Rust 2021 edition, Cargo workspace resolver v2.
**Key dependencies**: `serde`, `serde_json`, `thiserror`, `chrono`, `sha2`, `hex`, `dashmap`, `lru`, `moka`, `toml`, `regex`, `uuid`.
**Consumers**: Any Phenotype service or library implementing hexagonal architecture ports.
**Repository**: `KooshaPari/phenotype-infrakit`

---

## E1: Event Sourcing (`phenotype-event-sourcing`)

### E1.1: Generic Event Envelope

As a backend developer, I want to wrap any serializable domain event in an `EventEnvelope<T>` that carries a UUIDv4 identifier, UTC timestamp, named actor, and JSON-serialized payload, so domain events are uniformly structured regardless of their type.

**Acceptance criteria**:
- `EventEnvelope::new(payload, actor)` initializes `id` to a fresh UUIDv4 and `timestamp` to `Utc::now()`.
- `sequence` is initialized to `0` and `hash` to `""` as sentinels; the store fills both on append.
- `prev_hash` is initialized to 64 zero hex characters (the chain genesis marker).
- The envelope round-trips through `serde_json` without data loss for any `T: Serialize + DeserializeOwned`.

### E1.2: SHA-256 Hash Chain

As a platform auditor, I want each event to cryptographically link to its predecessor via SHA-256 so that any post-hoc modification to the event log is detectable.

**Acceptance criteria**:
- `compute_hash(id, timestamp, event_type, payload, actor, prev_hash)` produces a deterministic 64-character lowercase hex string.
- Hash input is constructed in this exact order: UUID bytes (16 bytes), big-endian u32 length + ISO 8601 timestamp bytes, big-endian u32 length + event_type bytes, big-endian u32 length + JSON payload bytes, big-endian u32 length + actor bytes, 32-byte decoded prev_hash.
- The first event uses `"0".repeat(64)` as prev_hash.
- `verify_chain(&[(hash, prev_hash)])` returns `HashError::ChainBroken { sequence }` on the first broken link.
- `detect_gaps(&[i64])` returns `Some(first_missing)` or `None` when the sequence is contiguous.
- `compute_hash` is deterministic: identical inputs produce identical output across invocations.

### E1.3: EventStore Trait

As a service developer, I want a `Send + Sync` `EventStore` trait with a stable contract so I can swap storage backends (in-memory for tests, persistent for production) without changing domain code.

**Acceptance criteria**:
- `append<T: Serialize + DeserializeOwned>(&self, event, event_type) -> Result<i64>` assigns next sequence and computes the SHA-256 hash before storing.
- `get_events<T>(entity_type, entity_id) -> Result<Vec<EventEnvelope<T>>>` returns events in ascending sequence order.
- `get_events_since<T>(entity_type, entity_id, sequence) -> Result<Vec<...>>` returns events where `sequence > given` (exclusive lower bound).
- `get_events_by_range<T>(entity_type, entity_id, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<...>>` returns events with `timestamp >= from && timestamp <= to`.
- `get_latest_sequence(entity_type, entity_id) -> Result<i64>` returns `0` when no events exist for the entity.
- `verify_chain(entity_type, entity_id) -> Result<()>` validates the full hash chain and returns an error on any broken link.

### E1.4: In-Memory EventStore

As a test author, I want an `InMemoryEventStore` that implements `EventStore` so I can write deterministic unit tests without external I/O.

**Acceptance criteria**:
- Backed by `RwLock<BTreeMap<entity_type, BTreeMap<entity_id, Vec<StoredEvent>>>>`.
- Concurrent reads are permitted; writes exclusively lock.
- `clear()` resets all state; `event_count()` returns the total number of stored events across all entities.
- Appending two events for the same entity (same UUID) produces sequences 1 and 2 with a valid hash chain.
- `get_events` returns `EventStoreError::NotFound` for an unknown entity/type combination.

### E1.5: Snapshot Support

As a service developer, I want configurable snapshot policies so I can cap aggregate-rebuild time by avoiding full event replay for long-lived entities.

**Acceptance criteria**:
- `SnapshotConfig { event_threshold: i64, time_threshold_secs: u64 }` defaults to 100 events / 300 seconds.
- `Snapshot<T> { entity_type, entity_id, state: T, event_sequence: i64, created_at: DateTime<Utc> }` is serializable.
- `should_snapshot(config, current_seq, last_snapshot_seq, last_snapshot_time: Option<DateTime<Utc>>)` returns `true` when `current_seq - last_snapshot_seq >= event_threshold` OR when `elapsed_since_last_snapshot > time_threshold`.

---

## E2: Two-Tier Cache (`phenotype-cache-adapter`)

### E2.1: Two-Tier Cache with TTL and Metrics

As a service developer, I want a two-tier cache (L1 in-process LRU, L2 concurrent DashMap/moka) with TTL expiration and optional `MetricsHook` so I can reduce latency on hot data while supporting concurrent access.

**Acceptance criteria**:
- L1 is backed by the `lru` crate; L2 is backed by `dashmap` or `moka` (sync feature enabled).
- A lookup checks L1 first; on L1 miss it falls through to L2; on L2 hit it backfills L1.
- Entries carry a TTL; the cache does not return expired entries.
- An optional `MetricsHook` trait object receives hit/miss events for observability integration.
- All public types implement `Send + Sync`.

---

## E3: Policy Engine (`phenotype-policy-engine`)

### E3.1: Rule Types and Pattern Matching

As a platform operator, I want three rule types — `Allow`, `Deny`, and `Require` — that evaluate a named fact from an `EvaluationContext` against a regex pattern, so I can express fine-grained access and compliance policies.

**Acceptance criteria**:
- `RuleType` is `Allow | Deny | Require`, implements `Serialize + Deserialize + Display`.
- `Rule::evaluate(&self, context: &EvaluationContext) -> Result<bool, PolicyEngineError>`:
  - `Allow`: absent fact passes; present fact must match pattern to pass.
  - `Deny`: absent fact passes; present fact must NOT match pattern to pass.
  - `Require`: absent fact fails; present fact must match pattern to pass.
- Invalid regex pattern returns `PolicyEngineError::RegexCompilationError { pattern, source }`.
- `Rule::with_description(str)` attaches a human-readable description field.

### E3.2: Policy Composition and Engine

As a platform operator, I want to group rules into named `Policy` objects and evaluate contexts against a single policy, a subset, or all loaded policies, so enforcement logic can be composed from independent rules.

**Acceptance criteria**:
- `Policy { name: String, enabled: bool, rules: Vec<Rule> }` is TOML-loadable via the `loader` module.
- `Policy::evaluate(context)` returns `PolicyResult { passed: bool, violations: Vec<Violation> }`.
- `PolicyEngine` uses `DashMap<String, Policy>` for thread-safe concurrent access.
- `PolicyEngine::evaluate_all(context)` merges violations from all enabled policies into a single `PolicyResult`.
- `PolicyEngine::evaluate_subset(names, context)` evaluates only the named policies.
- `enable_policy(name)` and `disable_policy(name)` return `PolicyEngineError::PolicyNotFound` for unknown names.
- `evaluate_policy(nonexistent_name, context)` returns `PolicyEngineError::PolicyNotFound`.

### E3.3: EvaluationContext

As a developer, I want an `EvaluationContext` that holds typed facts (string, number, bool, arbitrary JSON) so I can construct policy evaluation inputs without manually constructing JSON maps.

**Acceptance criteria**:
- `set_string`, `set_number`, `set_bool`, `set(key, serde_json::Value)` mutators.
- `get_string`, `get_number`, `get_bool`, `get` accessors returning `Option`.
- `contains(key) -> bool` membership test.
- `merge(other: EvaluationContext)` absorbs all facts from another context, overwriting on key conflict.
- `from_json(Value)` constructs from an object-shaped JSON value; non-object input yields empty context.
- `from_map(HashMap<String, Value>)` constructs directly from a pre-built map.

---

## E4: Hexagonal Contracts (`phenotype-contracts`)

### E4.1: Outbound Driven Ports

As a domain service author, I want typed traits for all driven ports so domain logic depends only on abstractions and adapters are injected at the composition root.

**Acceptance criteria**:
- `outbound::CachePort` — get/set/delete with optional TTL.
- `outbound::Repository` — CRUD operations for domain entities (find, save, delete).
- `outbound::SecretPort` — retrieve a secret value by string key.
- `outbound::EventBus` (or equivalent) — publish domain events to a downstream bus.
- All traits bound as `Send + Sync + 'static` for compatibility with async runtimes.

### E4.2: Domain Model Traits

As a domain modeler, I want base traits for `Entity`, `ValueObject`, and `AggregateRoot` so domain objects have a consistent identity and equality contract across the workspace.

**Acceptance criteria**:
- `Entity` — provides `id()` method returning a comparable, displayable identifier.
- `ValueObject` — equality is structural (value semantics); no mutable identity.
- `AggregateRoot` — extends `Entity`; exposes uncommitted domain events for collection and flushing.
- Crate-level `Result<T>` alias: `std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>`.

---

## E5: State Machine (`phenotype-state-machine`)

**Goal**: Generic forward-only finite state machine with guard callbacks, action hooks, full transition history, and skip-state configuration.

### E5.1: Typed Forward-Only FSM

As a service developer, I want a `StateMachine<S, C>` where `S` is the state enum and `C` is the context type so workflow state is enforced with forward-only transitions and domain-specific guard callbacks operating over typed context.

**Acceptance criteria**:
- `StateMachine::new(initial_state: S, initial_context: C)` constructs a machine in the given state with an owned context.
- `Transition::new(from, to)` creates a transition registration between two states.
- `add_transition(transition)` registers a `Transition<S, C>` with the machine.
- `transition_to(target_state)` returns `Ok(())` or `Err(StateMachineError::InvalidTransition)`.
- Transitions are matched by `(from == current, to == target)` equality; no matching transition returns `InvalidTransition { from, to }`.
- All public types (`S`, `C`) are bounded by `Clone + PartialEq + Debug + Serialize + DeserializeOwned`.
- All internal state is behind `Arc<RwLock<_>>` for `Send + Sync` compatibility.

### E5.2: Guard Callbacks and Action Hooks

As a service developer, I want guard conditions that gate transitions and action hooks that run on successful transitions so domain logic is decoupled from state-machine plumbing.

**Acceptance criteria**:
- `Transition::with_guard(Fn(&C) -> bool + Send + Sync + 'static)` attaches a guard closure; evaluated before the transition is applied.
- `Transition::with_action(Fn(&mut C) + Send + Sync + 'static)` attaches an action closure; executed after guard passes but before the state is updated.
- A failing guard returns `StateMachineError::GuardConditionFailed { reason }` and the machine state is unchanged.
- A transition with no guard always succeeds (permissive by default).
- `can_transition_to(&S)` returns `Ok(true)` if a matching transition exists and its guard (if any) returns `true`; `Ok(false)` otherwise.

### E5.3: Transition History

As an auditor, I want an immutable record of every state the machine has visited so transitions can be replayed and inspected post-hoc.

**Acceptance criteria**:
- `history()` returns `Result<Vec<S>>` containing every state in visitation order, starting with the initial state.
- History is append-only; each successful `transition_to` appends the new state.
- History is persisted behind `Arc<RwLock<Vec<S>>>` for concurrent read access.
- A machine with N successful transitions has `history().len() == N + 1`.

### E5.4: Skip-State Configuration

As a platform operator, I want to declare specific non-sequential state advances so emergency or out-of-band transitions bypass the normal forward path.

**Acceptance criteria**:
- `StateMachineConfig` (or equivalent) holds a `skip_states: Vec<(S, S)>` list of allowed non-sequential transitions.
- A transition that jumps forward (target ordinal > current + 1) is rejected unless explicitly listed in `skip_states`.
- A skip-state entry `(from, to)` is validated at registration: `to` ordinal must be greater than `from` ordinal.
- Skip-state transitions still require guard evaluation and trigger action hooks identically to sequential transitions.

---

## Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| NFR-INDEP | Each crate MUST compile independently with zero cross-crate workspace dependencies at the source level. |
| NFR-THREADSAFE | All public types exposed in adapter crates MUST implement `Send + Sync`. |
| NFR-SERDE | All public data types MUST implement `Serialize` and `Deserialize`. |
| NFR-ERROR | All errors MUST use `thiserror`-derived `Display` and `Error` impls; `unwrap()` is banned in library code outside tests. |
| NFR-TESTING | Each crate MUST include in-module unit tests (`#[cfg(test)]`) covering all public API methods. |
| NFR-MSRV | Minimum Supported Rust Version: current stable (edition 2021). |
| NFR-NODEPS | Workspace `[workspace.dependencies]` MUST pin all transitive library crates to specific semver ranges. |
