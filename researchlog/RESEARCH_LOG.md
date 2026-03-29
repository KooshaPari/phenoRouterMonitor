# Research Log

## Audit Research Sessions

### 2026-03-29 - Code Duplication Research

**Session:** Initial codebase analysis for duplication patterns
**Agents:** Sage (research) + Forge (implementation)

---

## Research Finding #1: Health Check Patterns

### Files Investigated

| File | Path | Lines | Pattern |
|------|------|-------|---------|
| CacheHealth | `crates/agileplus-cache/src/health.rs:5-8` | 42 | Healthy, Unavailable enum |
| GraphHealth | `crates/agileplus-graph/src/health.rs:5-8` | 90 | with store.health_check() |
| BusHealth | `crates/agileplus-nats/src/health.rs:5-8` | 8 | Connected, Disconnected enum |

### Summary
- 3 independent implementations of health check pattern
- All use similar enum-based status (Healthy/Unavailable or Connected/Disconnected)
- Different backend-specific check methods (Redis PING, Neo4j health, NATS connection)
- Total: 140 LOC duplicated

### Opportunity
- Create unified `agileplus-health` crate
- Generic `HealthChecker` trait with `async fn check() -> HealthStatus`
- Extension traits for common backends

### External Reference
- `health_check` crate on crates.io (1.4+ versions)
- docs.rs: https://docs.rs/health_check/1.10.0/health_check/
- Recommendation: Fork and extend for async_trait support

---

## Research Finding #2: Error Type Proliferation

### Error Enums Identified

| Crate | Error Type | Variants | LOC |
|-------|------------|----------|-----|
| agileplus-api | ApiError | 6 | 67 |
| agileplus-domain | DomainError | 15+ | 50 |
| agileplus-sync | SyncError | 5 | 24 |
| agileplus-p2p | PeerDiscoveryError, SyncError, ConnectionError | 78 |
| phenotype-port-interfaces | PortError | 10 | 51 |
| phenotype-event-sourcing | EventSourcingError, EventStoreError, HashError | 46 |
| phenotype-http-adapter | HttpError | 6 | 45 |

### Common Variants Across Enums

| Variant | Appears In |
|---------|------------|
| NotFound(String) | ApiError, DomainError, SyncError, PortError, EventSourcingError |
| Timeout(String) | DomainError, PortError, HttpError |
| Serialization(String) | SyncError, PortError, EventStoreError, HttpError |
| Config/Validation(String) | PortError |

### Summary
- 15+ error enums with similar variants
- 504 total LOC in error.rs files
- No shared base error type
- Inconsistent HTTP status mapping

### Opportunity
- Create `agileplus-error-core` crate
- Common `AppErrorKind` enum with shared variants
- `AppError` trait for HTTP status mapping
- `From` implementations for conversion

---

## Research Finding #3: Configuration Loading Patterns

### Config Patterns Identified

| Crate | Pattern | Format | Path |
|-------|---------|--------|------|
| agileplus-domain | TOML + env overrides | TOML | `~/.agileplus/config.toml` |
| agileplus-telemetry | YAML + env overrides | YAML | `~/.agileplus/otel-config.yaml` |
| agileplus-cache | Builder pattern | Struct | Simple |

### libs/config-core (UNDERSUSED)

```
libs/config-core/
├── src/
│   ├── adapters/     # TOML, YAML adapters
│   ├── domain/      # Config primitives
│   ├── ports/       # ConfigLoader trait
│   └── lib.rs
├── Cargo.toml
│   ├── dependencies: anyhow, serde, serde_json, toml
│   └── workspace: false (not integrated!)
└── tests/
```

### Summary
- Manual config loaders in 3+ crates
- libs/config-core exists but NOT integrated into workspace
- Different formats (TOML, YAML, struct)

### Opportunity
- Integrate libs/config-core into workspace
- Add `FromEnv` derive macro for automatic env var mapping
- Deprecate custom loaders in favor of config-core

---

## Research Finding #4: Port/Trait Architecture Split

### Two Hexagonal Ecosystems

#### Ecosystem 1: phenotype-port-interfaces
```
libs/phenotype-shared/crates/phenotype-port-interfaces/
├── src/outbound/
│   ├── repository.rs (Repository trait, 78 LOC)
│   ├── cache.rs (Cache trait)
│   ├── logger.rs (Logger trait, 101 LOC)
│   ├── event_bus.rs
│   ├── http.rs
│   ├── filesystem.rs
│   └── config.rs
└── src/error.rs (PortError, 51 LOC)
```

#### Ecosystem 2: agileplus-domain
```
crates/agileplus-domain/src/ports/
├── mod.rs
├── observability.rs (ObservabilityPort, 850 LOC)
├── agent.rs (AgentPort)
├── vcs.rs (VcsPort)
├── storage.rs (StoragePort)
└── review.rs (ReviewPort)
```

### libs/hexagonal-rs (ALSO UNDERSUSED)
```
libs/hexagonal-rs/
├── src/
│   ├── domain/
│   ├── ports/
│   ├── application/
│   └── adapters/
├── Cargo.toml
└── README.md (1.6KB - full hexagonal framework)
```

### Summary
- Two separate hexagonal ecosystems
- Overlapping concerns:
  - Logger trait vs ObservabilityPort
  - Repository trait vs StoragePort
- Framework libraries not integrated

### Opportunity
- Long-term: Consolidate port interfaces
- Short-term: Audit for overlap
- Framework: Leverage libs/hexagonal-rs

---

## Research Finding #5: API Response Patterns

### Patterns Identified

| Pattern | Location | Type |
|---------|----------|------|
| HealthResponse | `crates/agileplus-api/src/responses.rs:125-224` | Struct with HashMap |
| ApiHealth | `crates/agileplus-api/src/responses.rs:100-124` | Simple struct |
| ApiResponse | `platforms/heliosCLI/codex-rs/core/src/client.rs` | Generic<T> |

### agileplus-api Responses
```rust
pub struct HealthResponse { status: &'static str, version: &'static str }
pub struct ServiceHealth { status: String, latency_ms: Option<u64>, error: Option<String> }
pub struct ApiHealth { status: String, uptime_seconds: u64 }
pub struct DetailedHealthResponse { status: String, services: HashMap<String, ServiceHealth> }
```

### heliosCLI Response
```rust
pub struct ApiResponse<T> { success: bool, data: Option<T>, message: Option<String> }
```

### Summary
- Different response conventions between projects
- Health-specific types not shared
- No TypeScript generation support in agileplus

### Opportunity
- Create `agileplus-api-types` crate
- Unified `ApiResponse<T>` with success/data/error
- TypeScript code generation (ts_rs already in use)

---

## Research Finding #6: Builder Pattern Proliferation

### EventQuery Builder
```rust
// agileplus-events/src/query.rs:26-74
impl EventQuery {
    pub fn entity_type(self, entity_type: EntityType) -> Self { ... }
    pub fn entity_id(self, entity_id: Uuid) -> Self { ... }
    pub fn event_type(self, event_type: String) -> Self { ... }
    pub fn actor(self, actor: UserId) -> Self { ... }
    pub fn start_time(self, start_time: DateTime<Utc>) -> Self { ... }
    pub fn end_time(self, end_time: DateTime<Utc>) -> Self { ... }
    pub fn after_sequence(self, seq: u64) -> Self { ... }
    pub fn end_sequence(self, seq: u64) -> Self { ... }
    pub fn limit(self, limit: u32) -> Self { ... }
}
```

### CacheConfig Builder
```rust
// agileplus-cache/src/config.rs:13-35
impl CacheConfig {
    pub fn with_pool_size(self, pool_size: u32) -> Self { ... }
    pub fn with_default_ttl(self, ttl_secs: u64) -> Self { ... }
}
```

### Summary
- Multiple builder patterns across crates
- No generic QueryBuilder trait
- Different conventions

### Opportunity
- Create generic `QueryBuilder` trait
- Standardize builder conventions

---

## Research Finding #7: Async Trait Pattern Issues

### SnapshotStore - Misplaced
```rust
// agileplus-events/src/snapshot.rs:37-56
#[async_trait]
pub trait SnapshotStore {
    async fn save_snapshot(&self, snapshot: &Snapshot) -> Result<()>;
    async fn load_snapshot(&self, entity_id: &Uuid) -> Result<Option<Snapshot>>;
}
```

### Repository (phenotype-port-interfaces)
```rust
// libs/phenotype-shared/crates/phenotype-port-interfaces/src/outbound/repository.rs:18-40
#[async_trait]
pub trait Repository {
    async fn save(&self, entity: &dyn StorableEntity) -> Result<(), PortError>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<Box<dyn StorableEntity>>, PortError>;
    async fn delete(&self, id: &Uuid) -> Result<(), PortError>;
}
```

### Summary
- SnapshotStore has similar purpose to Repository
- SnapshotStore NOT in phenotype-port-interfaces despite pattern match
- Inconsistent trait organization

### Opportunity
- Move SnapshotStore to phenotype-port-interfaces
- Create unified entity persistence trait

---

## Research Finding #8: Hash/Snapshot Patterns

### Hash Chain (agileplus-events)
```rust
// agileplus-events/src/hash.rs:251 LOC
pub struct HashChain { ... }
impl HashChain {
    pub fn new(entity_id: Uuid) -> Self { ... }
    pub fn append(&mut self, content: &[u8]) -> Result<Hash, HashError> { ... }
    pub fn verify(&self) -> Result<bool, HashError> { ... }
}
```

### Content Hash (agileplus-plane)
```rust
// agileplus-plane/src/content_hash.rs
// Similar SHA-256 chain pattern
```

### P2P Vector Clock
```rust
// agileplus-p2p/src/vector_clock.rs
// CRDT-based clock, different purpose
```

### Summary
- SHA-256 chain well-consolidated in agileplus-events
- Content hash in plane may overlap
- Vector clock is P2P-specific (no consolidation)

---

## Research Finding #9: Connection Pool Patterns

### CachePool (bb8 + Redis)
```rust
// agileplus-cache/src/pool.rs:17-48
pub struct CachePool {
    manager: bb8::Pool<redis::aio::MultiplexedConnection>,
}
impl CachePool {
    pub async fn new(config: &CacheConfig) -> Result<Self> { ... }
    pub async fn get(&self) -> Result<PooledConnection<CachePool>> { ... }
}
```

### phenotype-redis-adapter (deadpool)
```rust
// libs/phenotype-shared/crates/phenotype-redis-adapter/Cargo.toml
deadpool-redis = { version = "0.14", features = ["serde"] }
```

### Summary
- Inconsistent pool managers: bb8 vs deadpool
- phenotype-redis-adapter uses deadpool
- agileplus-cache uses bb8

### Opportunity
- Migrate to deadpool everywhere
- deadpool has better async support

---

## Research Finding #10: Cross-Language Pattern Gap

### hexagonal-ts (TypeScript)
```
libs/hexagonal-ts/
├── src/
├── package.json (122KB package-lock)
└── node_modules/ (active project)
```

### hexagonal-rs (Rust) - UNDERSUSED
```
libs/hexagonal-rs/
├── src/ (full hexagonal framework)
└── Cargo.toml (workspace: false)
```

### phenotype-config (TypeScript)
```
libs/phenotype-config/ (43 directories)
```

### config-core (Rust) - UNDERSUSED
```
libs/config-core/
├── src/ (TOML/YAML adapters)
└── Cargo.toml (workspace: false)
```

### Summary
- TypeScript hexagonal-ts active but not integrated with Rust
- Rust hexagonal-rs framework not used
- TypeScript config (phenotype-config) active, Rust config-core unused

### Opportunity
- Integrate hexagonal-rs into Rust projects
- Share patterns between TypeScript and Rust
- Use phenotype-config patterns in config-core

---

*Research log maintained by Sage/Forge agents*
*Last updated: 2026-03-29T07:37:00Z*
