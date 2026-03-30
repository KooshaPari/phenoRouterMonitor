# FR Tracker - phenotype-infrakit

**Last Updated:** 2026-03-30
**Source:** FUNCTIONAL_REQUIREMENTS.md (47 FRs total)

| FR ID | Description | Status | Test Location |
|-------|-------------|--------|---------------|
| **FR-EVT-001** | `EventEnvelope::new(payload, actor)` SHALL initialize `id` to a fresh UUIDv4, `timestamp` to `Utc::now()`, `sequence` to `0`, and `hash` to `""`. | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-EVT-002** | `prev_hash` SHALL be initialized to 64 zero hex characters as the chain genesis marker. | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-EVT-003** | `EventEnvelope<T>` SHALL round-trip through `serde_json` without data loss for any `T: Serialize + DeserializeOwned`. | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-EVT-004** | `compute_hash(id, timestamp, event_type, payload, actor, prev_hash)` SHALL produce a deterministic 64-character lowercase hex SHA-256 string. | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-EVT-005** | Hash input construction SHALL follow this exact order: UUID bytes (16), big-endian u32 length + ISO 8601 timestamp bytes, big-endian u32 length + event_type bytes, big-endian u32 length + JSON payload bytes, big-endian u32 length + actor bytes, 32-byte decoded prev_hash. | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-EVT-006** | `verify_chain(pairs: &[(hash, prev_hash)])` SHALL return `HashError::ChainBroken { sequence }` on the first broken link. | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-EVT-007** | `detect_gaps(sequences: &[i64])` SHALL return `Some(first_missing)` when the sequence is non-contiguous, `None` when contiguous. | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-EVT-008** | `EventStore::append<T>(&self, event, event_type) -> Result<i64>` SHALL assign the next sequence number and compute the SHA-256 hash before storing. | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-EVT-009** | `EventStore::get_events<T>(entity_type, entity_id)` SHALL return events in ascending sequence order. | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-EVT-010** | `EventStore::get_events_since<T>(entity_type, entity_id, sequence)` SHALL return events where `sequence > given` (exclusive lower bound). | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-EVT-011** | `EventStore::get_events_by_range<T>(entity_type, entity_id, from, to)` SHALL return events with `timestamp >= from AND timestamp <= to`. | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-EVT-012** | `EventStore::get_latest_sequence(entity_type, entity_id)` SHALL return `0` when no events exist for the entity. | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-EVT-013** | `EventStore::verify_chain(entity_type, entity_id)` SHALL validate the full hash chain and return an error on the first broken link. | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-EVT-014** | `InMemoryEventStore` SHALL permit concurrent reads and exclusive writes. | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-EVT-015** | `InMemoryEventStore::clear()` SHALL reset all state; `event_count()` SHALL return total events across all entities. | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-EVT-016** | `SnapshotConfig` SHALL default to 100 events or 300 seconds. `should_snapshot(config, current_seq, last_snap_seq, last_snap_time)` SHALL return `true` when either threshold is exceeded. | Missing | `crates/phenotype-event-sourcing/src/lib.rs` (empty) |
| **FR-CACHE-001** | Cache lookups SHALL check L1 (LRU, backed by `lru` crate) first; on L1 miss SHALL fall through to L2 (backed by `dashmap` or `moka` sync). | Missing | `crates/phenotype-cache-adapter/src/lib.rs` (empty) |
| **FR-CACHE-002** | On an L2 hit, the entry SHALL be backfilled into L1. | Missing | `crates/phenotype-cache-adapter/src/lib.rs` (empty) |
| **FR-CACHE-003** | Entries carrying a TTL SHALL not be returned after the TTL elapses. | Missing | `crates/phenotype-cache-adapter/src/lib.rs` (empty) |
| **FR-CACHE-004** | An optional `MetricsHook` trait object SHALL receive hit/miss events for observability integration. | Missing | `crates/phenotype-cache-adapter/src/lib.rs` (empty) |
| **FR-CACHE-005** | All public cache types SHALL implement `Send + Sync`. | Missing | `crates/phenotype-cache-adapter/src/lib.rs` (empty) |
| **FR-POL-001** | `RuleType` SHALL have variants `Allow`, `Deny`, `Require` and SHALL implement `Serialize + Deserialize + Display`. | Implemented | `crates/phenotype-policy-engine/src/rule.rs` |
| **FR-POL-002** | `Rule::evaluate(&self, context: &EvaluationContext)` SHALL return allow/deny/require logic as specified. | Implemented | `crates/phenotype-policy-engine/src/rule.rs` |
| **FR-POL-003** | An invalid regex pattern SHALL return `PolicyEngineError::RegexCompilationError { pattern, source }`. | Partial | `crates/phenotype-policy-engine/src/rule.rs` (handles regex errors, but loader.rs has unwrap) |
| **FR-POL-004** | `Rule::with_description(str)` SHALL attach a human-readable description. | Implemented | `crates/phenotype-policy-engine/src/rule.rs` |
| **FR-POL-005** | `Policy` SHALL be TOML-loadable via the `loader` module and SHALL have `name: String`, `enabled: bool`, and `rules: Vec<Rule>` fields. | Partial | `crates/phenotype-policy-engine/src/loader.rs` (unwrap on TOML parse) |
| **FR-POL-006** | `Policy::evaluate(context)` SHALL return `PolicyResult { passed: bool, violations: Vec<Violation> }`. | Implemented | `crates/phenotype-policy-engine/src/policy.rs` |
| **FR-POL-007** | `PolicyEngine` SHALL use `DashMap<String, Policy>` for thread-safe concurrent access. | Implemented | `crates/phenotype-policy-engine/src/engine.rs` |
| **FR-POL-008** | `PolicyEngine::evaluate_all(context)` SHALL merge violations from all enabled policies into one `PolicyResult`. | Implemented | `crates/phenotype-policy-engine/src/engine.rs` |
| **FR-POL-009** | `PolicyEngine::evaluate_subset(names, context)` SHALL evaluate only named policies. | Implemented | `crates/phenotype-policy-engine/src/engine.rs` |
| **FR-POL-010** | `enable_policy(name)` and `disable_policy(name)` SHALL return `PolicyEngineError::PolicyNotFound` for unknown names. | Implemented | `crates/phenotype-policy-engine/src/engine.rs` |
| **FR-POL-011** | `EvaluationContext` SHALL support `set_string`, `set_number`, `set_bool`, `set(key, serde_json::Value)` mutators. | Implemented | `crates/phenotype-policy-engine/src/context.rs` |
| **FR-POL-012** | `EvaluationContext` SHALL support `get_string`, `get_number`, `get_bool`, `get` accessors returning `Option`. | Implemented | `crates/phenotype-policy-engine/src/context.rs` |
| **FR-POL-013** | `EvaluationContext::merge(other)` SHALL absorb all facts from another context, overwriting on key conflict. | Implemented | `crates/phenotype-policy-engine/src/context.rs` |
| **FR-POL-014** | `EvaluationContext::from_json(Value)` SHALL construct from an object-shaped JSON value; non-object input yields an empty context. | Implemented | `crates/phenotype-policy-engine/src/context.rs` |
| **FR-CTR-001** | `outbound::CachePort` SHALL define get, set, and delete operations with optional TTL. | Missing | `crates/phenotype-contracts/src/lib.rs` (empty) |
| **FR-CTR-002** | `outbound::Repository<E, I>` SHALL define `find_by_id`, `save`, `delete`, `find_all`, and `find_by` operations. | Missing | `crates/phenotype-contracts/src/lib.rs` (empty) |
| **FR-CTR-003** | `outbound::SecretPort` SHALL define a `get_secret(key: &str)` operation. | Missing | `crates/phenotype-contracts/src/lib.rs` (empty) |
| **FR-CTR-004** | All outbound port traits SHALL be bound as `Send + Sync`. | Missing | `crates/phenotype-contracts/src/lib.rs` (empty) |
| **FR-CTR-005** | `models::Entity` trait SHALL provide an `id()` method returning a comparable, displayable identifier. | Missing | `crates/phenotype-contracts/src/lib.rs` (empty) |
| **FR-CTR-006** | `models::ValueObject` trait SHALL enforce value-based equality semantics. | Missing | `crates/phenotype-contracts/src/lib.rs` (empty) |
| **FR-CTR-007** | `models::AggregateRoot` trait SHALL extend `Entity` and expose uncommitted domain events for collection and flushing. | Missing | `crates/phenotype-contracts/src/lib.rs` (empty) |
| **FR-CTR-008** | The crate-level `Result<T>` alias SHALL be `std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>`. | Missing | `crates/phenotype-contracts/src/lib.rs` (empty) |
| **FR-SM-001** | The state machine SHALL reject transitions to states with lower ordinal values (forward-only enforcement). | Implemented | `crates/phenotype-state-machine/src/lib.rs` |
| **FR-SM-002** | The state machine SHALL support guard callbacks that can reject transitions. | Implemented | `crates/phenotype-state-machine/src/lib.rs` |
| **FR-SM-003** | The state machine SHALL maintain a full history of state transitions. | Missing | `crates/phenotype-state-machine/src/lib.rs` (history tracking not implemented) |
| **FR-SM-004** | The state machine SHALL support optional skip-state configuration for controlled non-sequential advancement. | Missing | `crates/phenotype-state-machine/src/lib.rs` (skip-state not implemented) |

## Summary

- **Total FRs:** 47
- **Implemented:** 14 (FR-POL-001,002,004,006,007,008,009,010,011,012,013,014, FR-SM-001,002)
- **Partial:** 2 (FR-POL-003, FR-POL-005)
- **Missing:** 31 (FR-EVT-001..016, FR-CACHE-001..005, FR-CTR-001..008, FR-SM-003,004)
- **FRs with no corresponding tests:** All FR-EVT, FR-CACHE, FR-CTR, FR-SM-003/004 (no test files exist). FR-POL-003/005 have tests but may contain unwrap issues.

**Note:** The test locations for missing FRs point to empty source files. Actual implementation and tests are required.