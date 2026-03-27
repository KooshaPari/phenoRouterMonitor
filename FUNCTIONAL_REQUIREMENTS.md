# Functional Requirements — phenotype-infrakit

**Version:** 1.1.0
**Traces to:** `PRD.md`
**Stack:** Rust 2021, Cargo workspace

---

## FR-EVT: Event Sourcing (`phenotype-event-sourcing`)

- **FR-EVT-001:** `EventEnvelope::new(payload, actor)` SHALL initialize `id` to a fresh UUIDv4,
  `timestamp` to `Utc::now()`, `sequence` to `0`, and `hash` to `""`.
  Traces to: E1.1
- **FR-EVT-002:** `prev_hash` SHALL be initialized to 64 zero hex characters as the chain genesis
  marker.
  Traces to: E1.1
- **FR-EVT-003:** `EventEnvelope<T>` SHALL round-trip through `serde_json` without data loss for
  any `T: Serialize + DeserializeOwned`.
  Traces to: E1.1
- **FR-EVT-004:** `compute_hash(id, timestamp, event_type, payload, actor, prev_hash)` SHALL
  produce a deterministic 64-character lowercase hex SHA-256 string.
  Traces to: E1.2
- **FR-EVT-005:** Hash input construction SHALL follow this exact order: UUID bytes (16),
  big-endian u32 length + ISO 8601 timestamp bytes, big-endian u32 length + event\_type bytes,
  big-endian u32 length + JSON payload bytes, big-endian u32 length + actor bytes,
  32-byte decoded prev\_hash.
  Traces to: E1.2
- **FR-EVT-006:** `verify_chain(pairs: &[(hash, prev_hash)])` SHALL return
  `HashError::ChainBroken { sequence }` on the first broken link.
  Traces to: E1.2
- **FR-EVT-007:** `detect_gaps(sequences: &[i64])` SHALL return `Some(first_missing)` when
  the sequence is non-contiguous, `None` when contiguous.
  Traces to: E1.2
- **FR-EVT-008:** `EventStore::append<T>(&self, event, event_type) -> Result<i64>` SHALL assign
  the next sequence number and compute the SHA-256 hash before storing.
  Traces to: E1.3
- **FR-EVT-009:** `EventStore::get_events<T>(entity_type, entity_id)` SHALL return events in
  ascending sequence order.
  Traces to: E1.3
- **FR-EVT-010:** `EventStore::get_events_since<T>(entity_type, entity_id, sequence)` SHALL
  return events where `sequence > given` (exclusive lower bound).
  Traces to: E1.3
- **FR-EVT-011:** `EventStore::get_events_by_range<T>(entity_type, entity_id, from, to)` SHALL
  return events with `timestamp >= from AND timestamp <= to`.
  Traces to: E1.3
- **FR-EVT-012:** `EventStore::get_latest_sequence(entity_type, entity_id)` SHALL return `0`
  when no events exist for the entity.
  Traces to: E1.3
- **FR-EVT-013:** `EventStore::verify_chain(entity_type, entity_id)` SHALL validate the full
  hash chain and return an error on the first broken link.
  Traces to: E1.3
- **FR-EVT-014:** `InMemoryEventStore` SHALL permit concurrent reads and exclusive writes.
  Traces to: E1.4
- **FR-EVT-015:** `InMemoryEventStore::clear()` SHALL reset all state;
  `event_count()` SHALL return total events across all entities.
  Traces to: E1.4
- **FR-EVT-016:** `SnapshotConfig` SHALL default to 100 events or 300 seconds.
  `should_snapshot(config, current_seq, last_snap_seq, last_snap_time)` SHALL return `true`
  when either threshold is exceeded.
  Traces to: E1.5

---

## FR-CACHE: Two-Tier Cache (`phenotype-cache-adapter`)

- **FR-CACHE-001:** Cache lookups SHALL check L1 (LRU, backed by `lru` crate) first; on L1 miss
  SHALL fall through to L2 (backed by `dashmap` or `moka` sync).
  Traces to: E2.1
- **FR-CACHE-002:** On an L2 hit, the entry SHALL be backfilled into L1.
  Traces to: E2.1
- **FR-CACHE-003:** Entries carrying a TTL SHALL not be returned after the TTL elapses.
  Traces to: E2.1
- **FR-CACHE-004:** An optional `MetricsHook` trait object SHALL receive hit/miss events for
  observability integration.
  Traces to: E2.1
- **FR-CACHE-005:** All public cache types SHALL implement `Send + Sync`.
  Traces to: E2.1

---

## FR-POL: Policy Engine (`phenotype-policy-engine`)

- **FR-POL-001:** `RuleType` SHALL have variants `Allow`, `Deny`, `Require` and SHALL implement
  `Serialize + Deserialize + Display`.
  Traces to: E3.1
- **FR-POL-002:** `Rule::evaluate(&self, context: &EvaluationContext)` SHALL return:
  - `Allow`: absent fact passes; present fact must match pattern.
  - `Deny`: absent fact passes; present fact must NOT match pattern.
  - `Require`: absent fact fails; present fact must match pattern.
  Traces to: E3.1
- **FR-POL-003:** An invalid regex pattern SHALL return
  `PolicyEngineError::RegexCompilationError { pattern, source }`.
  Traces to: E3.1
- **FR-POL-004:** `Rule::with_description(str)` SHALL attach a human-readable description.
  Traces to: E3.1
- **FR-POL-005:** `Policy` SHALL be TOML-loadable via the `loader` module and SHALL have
  `name: String`, `enabled: bool`, and `rules: Vec<Rule>` fields.
  Traces to: E3.2
- **FR-POL-006:** `Policy::evaluate(context)` SHALL return
  `PolicyResult { passed: bool, violations: Vec<Violation> }`.
  Traces to: E3.2
- **FR-POL-007:** `PolicyEngine` SHALL use `DashMap<String, Policy>` for thread-safe concurrent
  access.
  Traces to: E3.2
- **FR-POL-008:** `PolicyEngine::evaluate_all(context)` SHALL merge violations from all enabled
  policies into one `PolicyResult`.
  Traces to: E3.2
- **FR-POL-009:** `PolicyEngine::evaluate_subset(names, context)` SHALL evaluate only named
  policies.
  Traces to: E3.2
- **FR-POL-010:** `enable_policy(name)` and `disable_policy(name)` SHALL return
  `PolicyEngineError::PolicyNotFound` for unknown names.
  Traces to: E3.2
- **FR-POL-011:** `EvaluationContext` SHALL support `set_string`, `set_number`, `set_bool`,
  `set(key, serde_json::Value)` mutators.
  Traces to: E3.3
- **FR-POL-012:** `EvaluationContext` SHALL support `get_string`, `get_number`, `get_bool`,
  `get` accessors returning `Option`.
  Traces to: E3.3
- **FR-POL-013:** `EvaluationContext::merge(other)` SHALL absorb all facts from another context,
  overwriting on key conflict.
  Traces to: E3.3
- **FR-POL-014:** `EvaluationContext::from_json(Value)` SHALL construct from an object-shaped
  JSON value; non-object input yields an empty context.
  Traces to: E3.3

---

## FR-CTR: Hexagonal Contracts (`phenotype-contracts`)

- **FR-CTR-001:** `outbound::CachePort` SHALL define get, set, and delete operations with
  optional TTL.
  Traces to: E4.1
- **FR-CTR-002:** `outbound::Repository<E, I>` SHALL define `find_by_id`, `save`, `delete`,
  `find_all`, and `find_by` operations.
  Traces to: E4.1
- **FR-CTR-003:** `outbound::SecretPort` SHALL define a `get_secret(key: &str)` operation.
  Traces to: E4.1
- **FR-CTR-004:** All outbound port traits SHALL be bound as `Send + Sync`.
  Traces to: E4.1
- **FR-CTR-005:** `models::Entity` trait SHALL provide an `id()` method returning a comparable,
  displayable identifier.
  Traces to: E4.2
- **FR-CTR-006:** `models::ValueObject` trait SHALL enforce value-based equality semantics.
  Traces to: E4.2
- **FR-CTR-007:** `models::AggregateRoot` trait SHALL extend `Entity` and expose uncommitted
  domain events for collection and flushing.
  Traces to: E4.2
- **FR-CTR-008:** The crate-level `Result<T>` alias SHALL be
  `std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>`.
  Traces to: E4.2

---

## FR-SM: State Machine (`phenotype-state-machine`)

- **FR-SM-001:** The state machine SHALL reject transitions to states with lower ordinal values
  (forward-only enforcement).
  Traces to: E4.3
- **FR-SM-002:** The state machine SHALL support guard callbacks that can reject transitions.
  Traces to: E4.3
- **FR-SM-003:** The state machine SHALL maintain a full history of state transitions.
  Traces to: E4.3
- **FR-SM-004:** The state machine SHALL support optional skip-state configuration for controlled
  non-sequential advancement.
  Traces to: E4.3

---

## Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| NFR-INDEP | Each crate SHALL compile independently with zero cross-crate source-level dependencies. |
| NFR-THREADSAFE | All public adapter types SHALL implement `Send + Sync`. |
| NFR-SERDE | All public data types SHALL implement `Serialize` and `Deserialize`. |
| NFR-ERROR | All errors SHALL use `thiserror`-derived `Display` and `Error` impls; `unwrap()` is banned in library code outside tests. |
| NFR-TESTS | Each crate SHALL include `#[cfg(test)]` in-module unit tests covering all public API methods. |
| NFR-MSRV | Minimum Supported Rust Version: current stable, edition 2021. |
| NFR-DEPS | `[workspace.dependencies]` SHALL pin all transitive library crates to specific semver ranges. |
