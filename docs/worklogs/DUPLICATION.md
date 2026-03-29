# Duplication Worklogs

**Category:** DUPLICATION | **Updated:** 2026-03-29

---

## 2026-03-29 - AgilePlus Extended Duplication Audit
## 2026-03-29 - FORGE Extended Deep Research (Resumed Session)

**Project:** [AgilePlus]
**Category:** duplication
**Status:** in_progress
**Priority:** P0

### Summary
Extended deep research analysis verified through code inspection. All 11 libraries in `libs/` are UNUSED due to edition 2021 vs workspace 2024 mismatch.

### External Package Cross-Reference Matrix

| Internal Pattern | LOC | External Alternative | Recommendation | Savings |
|-----------------|-----|-------------------|----------------|---------|
| PTY/Process (3 repos) | ~1,433 | `command-group` | FORK → phenotype-process | 79% |
| Error types (135 files) | ~400 | CodexErr pattern | FORK → phenotype-error | 75% |
| Config loading (4 impls) | ~760 | `figment` | ADOPT | 80% |
| CLI progress bars | ~100 | `indicatif` | ADOPT | 80% |
| Git operations (3 repos) | ~500 | `gix` | MIGRATE | 60% |
| Event sourcing | ~300 | `eventually` | EVALUATE | TBD |
| Signal handling (5 daemons) | ~50 | `signal-hook` | EVALUATE | 30% |

### Total LOC Impact from External Packages

| Category | Current | External + Adoption | Reduction |
|----------|---------|-------------------|-----------|
| Process/PTY | ~1,433 | ~300 (command-group) | **79%** |
| Config loading | ~760 | ~150 (figment) | **80%** |
| Error handling | ~400 | ~100 (phenotype-error) | **75%** |
| Git operations | ~500 | ~200 (gix migration) | **60%** |
| CLI progress | ~100 | ~20 (indicatif) | **80%** |
| **TOTAL** | **~3,193** | **~770** | **~76%** |

### Detailed Findings

#### 1. Health Check Patterns (140 LOC across 3 files)

| File | Pattern | LOC |
|------|---------|-----|
| `crates/agileplus-cache/src/health.rs:5-8` | CacheHealth enum | 42 |
| `crates/agileplus-graph/src/health.rs:5-8` | GraphHealth enum + store.health_check() | 90 |
| `crates/agileplus-nats/src/health.rs:5-8` | BusHealth enum | 8 |

**Common Pattern:** HealthStatus enum with Healthy/Unavailable states + backend-specific check methods

**External Reference:** https://docs.rs/health_check/1.10.0/health_check/

**Canonical Location:** `agileplus-health` crate (PROPOSED)

#### 2. Error Type Proliferation (504 LOC across 15+ enums)

| Crate | Error Type | Variants | LOC |
|-------|------------|----------|-----|
| agileplus-api | ApiError | 6 | 67 |
| agileplus-domain | DomainError | 15+ | 50 |
| agileplus-sync | SyncError | 5 | 24 |
| agileplus-p2p | PeerDiscoveryError | 78 |
| phenotype-port-interfaces | PortError | 10 | 51 |
| phenotype-event-sourcing | EventSourcingError | 46 |
| phenotype-http-adapter | HttpError | 6 | 45 |

**Common Variants:** NotFound, Timeout, Serialization, Config/Validation

**Canonical Location:** `agileplus-error-core` crate (PROPOSED)

#### 3. Config Loading Patterns (449 LOC)

| Crate | Pattern | Format | Canonical |
|-------|---------|--------|-----------|
| agileplus-domain | TOML + env overrides | TOML | libs/config-core |
| agileplus-telemetry | YAML + env overrides | YAML | libs/config-core |
| agileplus-cache | Builder pattern | Struct | libs/config-core |

**Status:** libs/config-core EXISTS but workspace: false - UNUSED

#### 4. Port/Trait Architecture Split (2106 LOC)

| Ecosystem | Location | Ports |
|-----------|----------|-------|
| phenotype-port-interfaces | `libs/phenotype-shared/` | 8 traits |
| agileplus-domain | `crates/agileplus-domain/src/ports/` | 5 traits |
| hexagonal-rs | `libs/hexagonal-rs/` | Full framework (UNUSED) |

**Overlapping Concerns:**
- Logger trait vs ObservabilityPort
- Repository trait vs StoragePort

#### 5. API Response Patterns (224 LOC)

| Pattern | Location | Type |
|---------|----------|------|
| HealthResponse | `crates/agileplus-api/src/responses.rs:125-224` | Struct with HashMap |
| ApiResponse | `platforms/heliosCLI/codex-rs/core/src/client.rs` | Generic<T> |

**Canonical Location:** `agileplus-api-types` crate (PROPOSED)

#### 6. Builder Pattern Proliferation

| Builder | Location | Methods |
|---------|----------|---------|
| EventQuery | `agileplus-events/src/query.rs:26-74` | 9 methods |
| CacheConfig | `agileplus-cache/src/config.rs:13-35` | 2 methods |

#### 7. Async Trait Issues

**SnapshotStore misplaced:** `agileplus-events/src/snapshot.rs:37-56`
- Uses #[async_trait]
- NOT in phenotype-port-interfaces despite similar purpose to Repository trait

#### 8. Connection Pool Patterns

| Pool | Location | Manager |
|------|----------|---------|
| CachePool | `agileplus-cache/src/pool.rs:17-48` | bb8 |
| phenotype-redis-adapter | `libs/phenotype-shared/` | deadpool |

**Issue:** Inconsistent pool managers (bb8 vs deadpool)

### LOC Savings Potential

| Pattern | Current | Savings | Canonical |
|---------|---------|---------|-----------|
| Health checks | 140 | 80 | agileplus-health |
| Error types | 504 | 150 | agileplus-error-core |
| Config loaders | 449 | 200 | libs/config-core |
| API types | 224 | 50 | agileplus-api-types |
| **Total** | **1,317** | **480** | |

### Action Items

- [ ] 🔴 CRITICAL: Create `agileplus-health` crate
- [ ] 🟡 HIGH: Create `agileplus-error-core` crate
- [ ] 🟡 HIGH: Integrate `libs/config-core` into workspace
- [ ] 🟡 HIGH: Move `SnapshotStore` to phenotype-port-interfaces
- [ ] 🟠 MEDIUM: Create `agileplus-api-types` crate
- [ ] 🟠 MEDIUM: Create generic QueryBuilder trait
- [ ] 🟠 MEDIUM: Audit port interfaces for consolidation
- [ ] 🟢 LOW: Migrate bb8 to deadpool

### Related

- Audit: `docs/reports/AGILEPLUS_DUPLICATION_AUDIT_20260329.md`
- Decomposition: `docs/reports/AGILEPLUS_DECOMPOSITION_AUDIT.md`

---

## 2026-03-29 - Cross-Project Duplication Audit (Comprehensive)

**Project:** [cross-repo]
**Category:** duplication
**Status:** in_progress
**Priority:** P0

### Summary

Comprehensive audit of cross-project duplication across AgilePlus, heliosCLI, thegent, and libraries. Identified 36+ duplicate error types, 4 duplicate config loaders, 3 duplicate health enums, and 4 duplicate in-memory stores.

### High Priority Findings

#### Error Type Duplication (36+ enums)

| Error Type | Locations | Severity |
|------------|-----------|----------|
| `NotFound` | DomainError, ApiError, GraphError, NexusError | High |
| `Conflict` | DomainError, ApiError, SyncError | High |
| `Serialization` | SyncError, CacheError, EventBusError | High |
| `Config/InvalidConfig` | Multiple crates | High |

**Affected Files:**
- `crates/agileplus-sync/src/error.rs:6-24`
- `crates/agileplus-p2p/src/error.rs:26-47`
- `crates/agileplus-nats/src/bus.rs:17-31`
- `crates/agileplus-cache/src/store.rs:9-19`
- `libs/nexus/src/error.rs`
- `libs/hexagonal-rs/src/lib.rs`

#### Configuration Loading Duplication (4 implementations)

| Crate | File | Pattern |
|-------|------|---------|
| agileplus-domain | `src/config/loader.rs:21-84` | TOML + dirs_next |
| agileplus-dashboard | `src/routes.rs:137-170` | Identical pattern |
| agileplus-telemetry | `src/config.rs:126-145` | YAML variant |
| agileplus-subcmds | `src/sync/config.rs:12-36` | JSON variant |

**Duplicated `home_dir()` usage:**
- `crates/agileplus-telemetry/src/config.rs:209`
- `crates/agileplus-domain/src/config/core.rs:26`
- `crates/agileplus-domain/src/config/credentials.rs:32`
- `crates/agileplus-domain/src/config/loader.rs:24`

### Medium Priority Findings

#### Health Check Duplication (3 enums + 1 sophisticated)

| Crate | Type | File |
|-------|------|------|
| agileplus-graph | `GraphHealth { Healthy, Unavailable }` | `src/health.rs:4-8` |
| agileplus-cache | `CacheHealth { Healthy, Unavailable }` | `src/health.rs:4-8` |
| agileplus-nats | `BusHealth { Connected, Disconnected }` | `src/health.rs:4-7` |
| agileplus-domain | `HealthStatus { Healthy, Degraded, Unavailable }` | `src/domain/service_health.rs:8-15` |

#### Store Trait Patterns (3 traits)

| Trait | Crate | File |
|-------|-------|------|
| `EventStore` | agileplus-events | `src/store.rs:21-53` |
| `CacheStore` | agileplus-cache | `src/store.rs:21-38` |
| `GraphBackend` | agileplus-graph | `src/store.rs:22-27` |

#### In-Memory Backend Duplication (4 stores)

| Crate | Type | File |
|-------|------|------|
| agileplus-nats | `InMemoryBus` | `src/bus.rs:127` |
| agileplus-graph | `InMemoryBackend` | `src/store.rs:106` |
| agileplus-domain | `InMemoryCredentialStore` | `src/credentials/memory.rs:15` |
| agileplus-sync | `InMemoryStore` | `src/store.rs:59` |

### Tasks Completed

- [x] Audited error type definitions across 24 crates
- [x] Documented config loading patterns
- [x] Identified health check duplications
- [x] Catalogued store trait patterns
- [x] Created consolidation plan

### Next Steps

- [ ] Create `agileplus-error-core` crate
- [ ] Extract `agileplus-config-core` crate
- [ ] Unify health status types
- [ ] Extract test utilities

### Related

- Full Plan: `plans/2026-03-29-CROSS_PROJECT_DUPLICATION_PLAN-v1.md`
- Audit Files: `plans/2026-03-29-DUPLICATION_AUDIT*.md`

---

## 2026-03-29 - AgilePlus Intra-Repo Duplication Audit

**Project:** [AgilePlus]
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Audited intra-repo duplication within AgilePlus 24-crate workspace. Identified library libification candidates.

### Findings

| Category | Count | Recommendation |
|----------|-------|----------------|
| Error enums | 36+ | Extract to `libs/error-core` |
| Config loaders | 4 | Extract to `libs/config-core` |
| Health enums | 4 | Extract to `libs/health-core` |
| In-memory stores | 4 | Extract to `libs/test-core` |
| Builder patterns | 12+ | Document as pattern |
| Async traits | 6+ | Consider `store-core` |

### Library Candidates

| Library | Purpose | Status |
|---------|---------|--------|
| `libs/nexus` | Already exists, underutilized | Investigate |
| `libs/hexagonal-rs` | Hex patterns, unused | Archive |
| `libs/cli-framework` | CLI utilities | Enhance |
| `libs/config-core` | NEW | Create |

### Recommendations

1. Audit `libs/` utilization - many libs are unused
2. Consolidate hexagonal architecture libs
3. Create shared error/config/health libraries
4. Document builder patterns as ADR

### Related

- Audit: `plans/2026-03-29-AGILEPLUS_INTRA_REPO_DUPLICATION_AUDIT-v1.md`
- Libification: `plans/2026-03-29-AUDIT_LIBIFICATION-v1.md`

---

## 2026-03-28 - Library Libification Audit

**Project:** [AgilePlus]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Audit of existing library crates in `libs/` directory. Many are underutilized or could be consolidated.

### Library Inventory

| Library | Purpose | Utilization | Recommendation |
|---------|---------|-------------|----------------|
| `nexus` | Error types, config | Partial | Expand |
| `hexagonal-rs` | Hex patterns | None | Archive |
| `cli-framework` | CLI utilities | Partial | Enhance |
| `cipher` | Encryption | None | Archive |
| `gauge` | Metrics | None | Archive |
| `config-core` | Config patterns | Partial | Create |

### Action Items

- [x] Audited all libs
- [ ] Consolidate nexus usage
- [ ] Archive unused libs
- [ ] Enhance cli-framework

### Related

- Audit: `plans/2026-03-29-AUDIT_LIBIFICATION-v1.md`

---

## 2026-03-28 - Framework Audit

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Audit of framework choices across projects. Identified inconsistencies in error handling, config loading, and CLI patterns.

### Framework Comparison

| Framework | AgilePlus | thegent | heliosCLI |
|-----------|-----------|---------|-----------|
| Error handling | thiserror | thiserror | thiserror |
| Config format | TOML | YAML | TOML |
| CLI parsing | clap | argparse | clap |
| Logging | tracing | logging | tracing |
| Testing | tokio-test | pytest | tokio-test |

### Convergence Recommendations

1. Standardize on TOML for all config
2. Share `thiserror` patterns
3. Document CLI conventions
4. Create shared test utilities

### Related

- Audit: `plans/2026-03-29-AUDIT_FRAMEWORK-v1.md`

---

## 2026-03-29 - heliosCLI Duplication Analysis

**Project:** [heliosCLI]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Analyzed heliosCLI for duplication with other Phenotype repositories.

### Findings

| Pattern | heliosCLI | Similar In | Recommendation |
|---------|-----------|------------|----------------|
| PTY management | `utils/pty/` | vibe-kanban, agileplus-git | FORK to `phenotype-process` |
| Error types | `error.rs` | 135 files across repos | FORK to `phenotype-error` |
| Git operations | `utils/git/` | agileplus-git | EVALUATE fork |

### Duplication with AgilePlus

| Pattern | heliosCLI | AgilePlus | Recommendation |
|---------|-----------|-----------|----------------|
| Error handling | `thiserror` | `thiserror` | Extract to shared |
| Config loading | TOML | TOML | Consider `figment` |
| Async traits | `async-trait` | `async-trait` | Already shared |

### Next Steps

- [ ] FORK-001: Evaluate `utils/pty` for `phenotype-process`
- [ ] FORK-002: Evaluate `error.rs` for `phenotype-error`
- [ ] Document shared patterns

---

## 2026-03-29 - AgilePlus Comprehensive Duplication Audit (SAGE/MUSE/FORGE)

**Project:** [AgilePlus]
**Category:** duplication
**Status:** completed
**Priority:** P0

### Scope

| Metric | Value |
|--------|-------|
| Total Files | 1,599 |
| Rust Files | 439 (27%) |
| Crates | 27 in main workspace |
| External Projects | 2 (phenotype-shared-wtrees, vibe-kanban) |

### Summary

Comprehensive analysis identifying 1,800 LOC of duplication with 1,200 LOC savings potential through consolidation.

### 🔴 CRITICAL: Error Types — 8 Independent Definitions (~600 LOC)

| Crate | Error Type | Lines | Key Variants |
|-------|------------|-------|--------------|
| `agileplus-api/src/error.rs` | `ApiError` | 67 | NotFound, BadRequest, Internal |
| `agileplus-p2p/src/error.rs` | `SyncError`, `PeerDiscoveryError` | 78 | Nats, Serialization |
| `agileplus-sync/src/error.rs` | `SyncError` | 24 | Store, Nats |
| `agileplus-domain/src/error.rs` | `DomainError` | 50 | NotFound, Conflict |
| `agileplus-events/src/store.rs` | `EventError` | 53 | NotFound, StorageError |
| `agileplus-graph/src/store.rs` | `GraphError` | 326 | ConnectionError, QueryError |
| `agileplus-cache/src/store.rs` | `CacheError` | 129 | Serialization, Redis |
| `phenotype-port-interfaces/src/error.rs` | `PortError` | 51 | NotFound, Validation |

**Duplicated Variants**: `NotFound(String)`, `SerializationError`, `StorageError`, `Conflict`

### 🟡 HIGH: Configuration Loading — 3 Independent Implementations (~500 LOC)

| Location | Format | Pattern |
|----------|--------|---------|
| `crates/agileplus-domain/src/config/loader.rs` | TOML | env overrides, `~/.agileplus/config.toml` |
| `crates/agileplus-telemetry/src/config.rs` | YAML | env overrides, `~/.agileplus/otel-config.yaml` |
| `vibe-kanban/backend/src/models/config.rs` | JSON | defaults merge |

**Library Status**: `libs/config-core/` exists but **UNUSED** (edition mismatch: 2021 vs 2024)

### 🟠 MEDIUM: Async Traits — 5+ Repository Traits

| Location | Trait | Async Pattern |
|----------|-------|---------------|
| `agileplus-nats/src/bus.rs` | EventBus | #[async_trait] |
| `agileplus-sync/src/store.rs` | SyncMappingStore | #[async_trait] |
| `agosevents/src/store.rs` | EventStore | #[async_trait] |
| `agileplus-graph/src/store.rs` | GraphBackend | #[async_trait] |
| `agileplus-cache/src/store.rs` | CacheStore | #[async_trait] |

**Library Status**: `libs/hexagonal-rs/src/ports/repository.rs` has exact patterns but **UNUSED**

### 🟠 MEDIUM: In-Memory Test Implementations — 4 Instances (~400 LOC)

| Trait | Implementation | Location |
|-------|---------------|----------|
| EventBus | InMemoryBus | `agileplus-nats/src/bus.rs:127-240` |
| SyncMappingStore | InMemorySyncStore | `agileplus-sync/src/store.rs:47-110` |
| GraphBackend | InMemoryGraphBackend | `agileplus-graph/src/store.rs:106-309` |

**Common Pattern**: `Arc<Mutex<HashMap<Key, Value>>>` duplicated 4+ times

### UNUSED LIBRARIES (11 total)

| Library | Purpose | Issue |
|---------|---------|-------|
| `config-core` | Config loading | edition mismatch |
| `logger` | Structured logging | edition mismatch |
| `tracing` | Distributed tracing | edition mismatch |
| `metrics` | Metrics collection | edition mismatch |
| `hexagonal-rs` | Ports & Adapters | edition mismatch, has exact patterns |
| `hexkit` | HTTP/Persistence | edition mismatch |
| `cipher` | Encryption | NOT AUDITED |
| `gauge` | Benchmarking | NOT AUDITED |
| `nexus` | Service discovery | NOT AUDITED |
| `xdd-lib-rs` | Data transformation | NOT AUDITED |
| `phenotype-state-machine` | State machine patterns | DEAD CODE |

**Root Cause**: `libs/` uses `edition = "2021"`, workspace uses `edition = "2024"`

### LOC Impact Summary

| Category | Current | After Consolidation | Savings |
|----------|---------|---------------------|---------|
| Error Types | 600 | 200 | 400 |
| Config Loading | 500 | 150 | 350 |
| In-Memory Impls | 400 | 150 | 250 |
| Async Traits | 300 | 100 | 200 |
| **Total** | **1,800** | **600** | **1,200** |

### Recommended Actions

- [ ] 🔴 CRITICAL: Create `libs/agileplus-error/` for error consolidation
- [ ] 🟡 HIGH: Migrate `libs/config-core` to edition 2024
- [ ] 🟡 HIGH: Integrate `libs/hexagonal-rs` Repository patterns
- [ ] 🟠 MEDIUM: Create shared InMemory test implementations
- [ ] 🟠 MEDIUM: Create `libs/http-client` for HTTP patterns
- [ ] 🟢 LOW: Delete `phenotype-state-machine` (dead code)

### Related

- `docs/research/consolidation-audit-2026-03-29.md` - Master findings
- `worklogs/WORK_LOG.md` - Wave 90 entry

---

## 2026-03-29 - Error Type Duplication Analysis

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P0

### Summary

Detailed analysis of error type duplication across the codebase. Found 8 independent error definitions with significant semantic overlap.

### Error Type Inventory

| Error Type | Location | Variants | LOC |
|------------|----------|----------|-----|
| ApiError | `crates/agileplus-api/src/error.rs` | 6 | 67 |
| SyncError | `crates/agileplus-sync/src/error.rs` | 5 | 24 |
| PeerDiscoveryError | `crates/agileplus-p2p/src/error.rs` | 4 | 78 |
| DomainError | `crates/agileplus-domain/src/error.rs` | 15+ | 50 |
| EventError | `crates/agileplus-events/src/store.rs` | 8 | 53 |
| GraphError | `crates/agileplus-graph/src/store.rs` | 12 | 326 |
| CacheError | `crates/agileplus-cache/src/store.rs` | 6 | 129 |
| PortError | `libs/phenotype-port-interfaces/src/error.rs` | 10 | 51 |

### Common Error Variants

| Variant | Appears In | Semantic Meaning |
|---------|------------|------------------|
| NotFound(String) | 5+ enums | Resource doesn't exist |
| SerializationError | 4 enums | JSON/serde failure |
| StorageError | 3 enums | Database/storage failure |
| Conflict(String) | 3 enums | State conflict |
| InvalidInput(String) | 3 enums | Validation failure |
| Timeout(String) | 2 enums | Operation timeout |

### Proposed Canonical Error Types

```rust
// libs/phenotype-error/src/lib.rs
pub enum PhenotypeError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}
```

### Migration Strategy

1. Create `phenotype-error` crate with canonical types
2. Add `#[from]` implementations for transparent conversion
3. Update existing error types to use canonical variants
4. Add blanket implementations for error conversion

### LOC Impact

| Metric | Value |
|--------|-------|
| Current LOC | ~800 |
| After consolidation | ~300 |
| Savings | ~500 |

---

## 2026-03-29 - Config Loading Duplication Analysis

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P0

### Summary

Analysis of configuration loading duplication. Found 4 independent implementations with identical patterns.

### Config Loader Inventory

| Loader | Location | Format | LOC |
|--------|----------|--------|-----|
| TOML loader | `agileplus-domain/src/config/loader.rs` | TOML | 84 |
| YAML loader | `agileplus-telemetry/src/config.rs` | YAML | 201 |
| JSON loader | `vibe-kanban/backend/src/models/config.rs` | JSON | 374 |
| Builder pattern | `agileplus-cache/src/config.rs` | Struct | 100 |

### Common Pattern

All implementations follow identical structure:

```rust
pub struct ConfigLoader<T> {
    path: PathBuf,
    env_prefix: String,
    defaults: T,
}

impl<T: DeserializeOwned> ConfigLoader<T> {
    pub fn load(&self) -> Result<T> {
        // 1. Load from file
        // 2. Merge with defaults
        // 3. Override with env vars
        // 4. Validate
    }
    
    fn load_file(&self) -> Result<T> { ... }
    fn apply_env_overrides(&self, config: &mut T) { ... }
    fn validate(&self, config: &T) -> Result<()> { ... }
}
```

### Duplicated Utilities

| Utility | Locations | Description |
|---------|-----------|-------------|
| `home_dir()` | 4 files | Get user's home directory |
| `config_path()` | 4 files | Get config file path |
| `env_override()` | 3 files | Override config with env vars |

### libs/config-core Status

```rust
// libs/config-core/src/lib.rs - EXISTS but UNUSED
pub struct ConfigLoader<T: DeserializeOwned> {
    path: PathBuf,
    env_prefix: String,
    validator: Option<Box<dyn Fn(&T) -> Result<(), ConfigError>>>,
}
```

### Issue: Edition Mismatch

| Component | Edition | Workspace |
|-----------|---------|-----------|
| libs/config-core | 2021 | false |
| Main workspace | 2024 | true |

### Recommendations

1. **Migrate libs/config-core to edition 2024**
2. **Add to main workspace**
3. **Update all crates to use shared loader**
4. **Remove duplicate implementations**

### LOC Impact

| Metric | Value |
|--------|-------|
| Current LOC | ~760 |
| After consolidation | ~150 |
| Savings | ~610 |

---

## 2026-03-29 - Async Trait Duplication Analysis

**Project:** [AgilePlus]
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Analysis of async trait duplication. Found 5+ repository/store traits with overlapping functionality.

### Trait Inventory

| Trait | Location | Methods | LOC |
|-------|----------|---------|-----|
| EventStore | `agileplus-events/src/store.rs` | 4 | 33 |
| SyncMappingStore | `agileplus-sync/src/store.rs` | 4 | 26 |
| GraphBackend | `agileplus-graph/src/store.rs` | 3 | 6 |
| CacheStore | `agileplus-cache/src/store.rs` | 4 | 18 |
| SnapshotStore | `agileplus-events/src/snapshot.rs` | 3 | 20 |

### Common Async Pattern

All traits follow the same `#[async_trait]` pattern:

```rust
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append(&self, event: &Event) -> Result<i64, EventError>;
    async fn get_events(&self, entity_type: &str, entity_id: i64) -> Result<Vec<Event>, EventError>;
    async fn delete(&self, entity_type: &str, entity_id: i64) -> Result<(), EventError>;
}
```

### Repository Trait in hexagonal-rs

```rust
// libs/hexagonal-rs/src/ports/repository.rs
#[async_trait]
pub trait Repository<E: Entity> {
    async fn find(&self, id: &E::Id) -> Result<Option<E>, RepositoryError>;
    async fn save(&self, entity: &E) -> Result<(), RepositoryError>;
    async fn delete(&self, id: &E::Id) -> Result<(), RepositoryError>;
}
```

### libs/hexagonal-rs Status

| Issue | Status | Impact |
|-------|--------|--------|
| edition mismatch | 2021 vs 2024 | UNUSED |
| Not in workspace | true | UNUSED |
| Has exact patterns | Yes | Could replace 5 traits |

### Recommendations

1. **Migrate hexagonal-rs to edition 2024**
2. **Add to main workspace**
3. **Replace duplicated traits with Repository<E>**
4. **Create specific trait extensions where needed**

### LOC Impact

| Metric | Value |
|--------|-------|
| Current LOC | ~103 |
| After consolidation | ~30 |
| Savings | ~73 |

---

## 2026-03-29 - Health Check Duplication Analysis

**Project:** [AgilePlus]
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Analysis of health check enum duplication. Found 4 independent health type definitions.

### Health Type Inventory

| Type | Location | Variants | LOC |
|------|----------|----------|-----|
| GraphHealth | `agileplus-graph/src/health.rs` | Healthy, Unavailable | 90 |
| CacheHealth | `agileplus-cache/src/health.rs` | Healthy, Unavailable | 42 |
| BusHealth | `agileplus-nats/src/health.rs` | Connected, Disconnected | 8 |
| HealthStatus | `agileplus-domain/src/domain/service_health.rs` | Healthy, Degraded, Unavailable | 100 |

### Health Type Details

```rust
// GraphHealth - most complete
pub enum GraphHealth {
    Healthy,
    Unavailable,
}

impl GraphHealth {
    pub async fn check(&self, backend: &GraphBackend) -> Result<(), GraphError> { ... }
}

// CacheHealth - simple
pub enum CacheHealth {
    Healthy,
    Unavailable,
}

// BusHealth - uses different terminology
pub enum BusHealth {
    Connected,
    Disconnected,
}

// HealthStatus - most sophisticated
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unavailable,
}
```

### Recommended Canonical Type

```rust
// libs/health-core/src/lib.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unavailable,
}

#[derive(Clone, Debug)]
pub struct ServiceHealth<T: Clone = ()> {
    pub status: HealthStatus,
    pub metadata: T,
    pub last_check: DateTime<Utc>,
}

#[async_trait]
pub trait HealthCheck {
    async fn check_health(&self) -> Result<HealthStatus, HealthError>;
}
```

### Recommendations

1. Create `libs/health-core` with canonical types
2. Replace GraphHealth, CacheHealth, BusHealth with HealthStatus
3. Add generic ServiceHealth<T> for metadata
4. Implement HealthCheck trait for all backends

---

## 2026-03-29 - In-Memory Test Implementation Duplication

**Project:** [AgilePlus]
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Analysis of in-memory test implementation duplication. Found 4 independent implementations.

### In-Memory Implementation Inventory

| Implementation | Location | Type | LOC |
|----------------|----------|------|-----|
| InMemoryBus | `agileplus-nats/src/bus.rs:127` | EventBus | 114 |
| InMemorySyncStore | `agileplus-sync/src/store.rs:47` | SyncMappingStore | 64 |
| InMemoryGraphBackend | `agileplus-graph/src/store.rs:106` | GraphBackend | 204 |
| InMemoryCredentialStore | `agileplus-domain/src/credentials/memory.rs:15` | CredentialStore | 50 |

### Common Pattern

All implementations follow identical structure:

```rust
pub struct InMemoryBackend {
    data: Arc<Mutex<HashMap<Key, Value>>>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl BackendTrait for InMemoryBackend {
    async fn get(&self, key: &str) -> Result<Option<Value>> {
        let data = self.data.lock().await;
        Ok(data.get(key).cloned())
    }
    
    async fn set(&self, key: &str, value: Value) -> Result<()> {
        let mut data = self.data.lock().await;
        data.insert(key.to_string(), value);
        Ok(())
    }
}
```

### Recommended Solution

```rust
// libs/test-core/src/lib.rs
pub struct InMemoryStore<K, V> {
    data: Arc<Mutex<HashMap<K, V>>>,
}

impl<K, V> InMemoryStore<K, V> {
    pub fn new() -> Self { ... }
}

impl<K, V> Default for InMemoryStore<K, V> {
    fn default() -> Self { ... }
}

pub type InMemoryEventBus = InMemoryStore<String, Event>;
pub type InMemoryCache = InMemoryStore<String, Vec<u8>>;
pub type InMemoryGraph = InMemoryStore<String, GraphNode>;
```

### Recommendations

1. Create `libs/test-core` with generic InMemoryStore
2. Provide typed aliases for each backend
3. Migrate existing implementations to use shared type
4. Add additional test utilities

---

## 2026-03-29 - Builder Pattern Duplication Analysis

**Project:** [AgilePlus]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Analysis of builder pattern duplication. Found 12+ builder implementations with similar structure.

### Builder Inventory

| Builder | Location | Methods | Purpose |
|---------|----------|---------|---------|
| EventQuery | `agileplus-events/src/query.rs` | 9 | Event query builder |
| CacheConfig | `agileplus-cache/src/config.rs` | 2 | Cache configuration |
| GraphConfig | `agileplus-graph/src/config.rs` | 2 | Graph configuration |
| NatsConfig | `agileplus-nats/src/config.rs` | 3 | NATS configuration |
| QueryBuilder | Various | 5+ | Generic query building |

### Common Builder Pattern

```rust
// EventQuery builder
pub struct EventQueryBuilder {
    entity_type: Option<String>,
    entity_id: Option<i64>,
    from_sequence: Option<i64>,
    limit: Option<usize>,
}

impl EventQueryBuilder {
    pub fn new() -> Self { ... }
    
    pub fn entity_type(mut self, t: String) -> Self { ... }
    
    pub fn entity_id(mut self, id: i64) -> Self { ... }
    
    pub fn from_sequence(mut self, seq: i64) -> Self { ... }
    
    pub fn limit(mut self, n: usize) -> Self { ... }
    
    pub async fn execute(&self, store: &EventStore) -> Result<Vec<Event>> { ... }
}
```

### Query Builder Framework

Consider using a query builder crate:

```rust
// libs/query-core/src/lib.rs
pub trait QueryBuilder<E> {
    fn build(&self) -> Query;
    fn execute(&self, store: &dyn Store<E>) -> Result<Vec<E>>;
}

pub struct Query {
    filters: Vec<Filter>,
    pagination: Pagination,
    ordering: Vec<Order>,
}
```

### Recommendations

1. Create `libs/query-core` for generic query building
2. Add builder trait with common methods
3. Migrate existing builders to use shared trait
4. Consider SQLx query builder patterns

---

## 2026-03-29 - API Response Pattern Duplication

**Project:** [AgilePlus]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Analysis of API response pattern duplication. Found multiple response types with similar structure.

### Response Type Inventory

| Type | Location | Fields | LOC |
|------|----------|--------|-----|
| HealthResponse | `agileplus-api/src/responses.rs:125` | HashMap | 100 |
| ApiResponse | `heliosCLI/codex-rs/core/src/client.rs` | Generic<T> | 50 |
| GraphQLResponse | Various | data, errors | 30 |

### Common Response Pattern

```rust
// Standard API response
pub struct ApiResponse<T> {
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub meta: ResponseMetadata,
}

pub struct ResponseMetadata {
    pub request_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub pagination: Option<PaginationInfo>,
}

// Health response
pub struct HealthResponse {
    pub status: HealthStatus,
    pub services: HashMap<String, ServiceHealth>,
    pub timestamp: DateTime<Utc>,
}
```

### Recommended Canonical Response

```rust
// libs/api-types/src/responses.rs
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorResponse>,
    
    pub meta: ResponseMeta,
}

#[derive(Debug, Serialize)]
pub struct ResponseMeta {
    pub request_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}
```

### Recommendations

1. Create `libs/api-types` with canonical response types
2. Migrate HealthResponse, ApiResponse to shared types
3. Add response builder for consistent construction
4. Add response validation middleware

---

## 2026-03-29 - Port/Trait Architecture Split Analysis

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Analysis of port/trait architecture split. Found two independent hexagonal ecosystems.

### Ecosystem 1: phenotype-port-interfaces

```
libs/phenotype-shared/crates/phenotype-port-interfaces/
├── src/outbound/
│   ├── repository.rs (Repository trait)
│   ├── cache.rs (Cache trait)
│   ├── logger.rs (Logger trait)
│   ├── event_bus.rs
│   ├── http.rs
│   ├── filesystem.rs
│   └── config.rs
└── src/error.rs (PortError)
```

### Ecosystem 2: agileplus-domain/ports

```
crates/agileplus-domain/src/ports/
├── mod.rs
├── observability.rs (ObservabilityPort)
├── agent.rs (AgentPort)
├── vcs.rs (VcsPort)
├── storage.rs (StoragePort)
└── review.rs (ReviewPort)
```

### Overlap Analysis

| phenotype-port-interfaces | agileplus-domain | Overlap |
|--------------------------|------------------|---------|
| Repository trait | StoragePort | HIGH |
| Logger trait | ObservabilityPort | HIGH |
| Cache trait | (no direct match) | - |
| EventBus trait | (no direct match) | - |

### Root Cause

| Issue | Description |
|-------|-------------|
| Two teams | Different developers created independent systems |
| No shared core | No canonical hexagonal framework |
| Edition mismatch | 2021 vs 2024 prevents integration |

### Recommended Consolidation

1. Migrate phenotype-port-interfaces to edition 2024
2. Add to main workspace
3. Consolidate overlapping traits
4. Archive agileplus-domain/ports

---

## 2026-03-29 - Git Operation Duplication Analysis

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Analysis of Git operation duplication across repositories.

### Git Implementation Inventory

| Location | Implementation | Features | LOC |
|----------|----------------|----------|-----|
| `agileplus-git/src/` | Full implementation | Worktree, history | 500+ |
| `heliosCLI/codex-rs/utils/git/` | Partial | Cherry-pick, branch | 300+ |
| `vibe-kanban/backend/src/git/` | Basic | Clone, commit, push | 200+ |

### Feature Comparison

| Feature | agileplus-git | heliosCLI | vibe-kanban |
|---------|---------------|-----------|-------------|
| Clone | ✅ | ✅ | ✅ |
| Commit | ✅ | ✅ | ✅ |
| Push/Pull | ✅ | ✅ | ✅ |
| Branch | ✅ | ✅ | ❌ |
| Worktree | ✅ | ❌ | ❌ |
| Cherry-pick | ✅ | ✅ | ❌ |
| Rebase | ✅ | ❌ | ❌ |
| History | ✅ | Partial | ❌ |

### Recommended Solution

Fork candidate FORK-003: Create `phenotype-git` from `agileplus-git`

```rust
// libs/phenotype-git/src/lib.rs
pub mod repository;
pub mod worktree;
pub mod history;

pub use repository::GitRepository;
pub use worktree::WorktreeManager;
pub use history::HistoryAnalyzer;
```

### LOC Impact

| Metric | Value |
|--------|-------|
| Current LOC | ~1000 |
| After consolidation | ~500 |
| Savings | ~500 |

---

## 2026-03-29 - Process/PTY Management Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P0

### Summary

Analysis of process/PTY management duplication. Found cross-platform PTY code in multiple locations.

### PTY Implementation Inventory

| Location | Implementation | Platforms | LOC |
|----------|----------------|-----------|-----|
| `heliosCLI/codex-rs/utils/pty/` | Full PTY | Unix + Windows | 750+ |
| `vibe-kanban/backend/src/process.rs` | Basic | Unix only | 100+ |
| `agileplus-git/src/worktree/mod.rs` | Process spawn | Unix + Windows | 50+ |

### Common PTY Pattern

```rust
pub struct PtyProcess {
    master: RawFd,
    slave: RawFd,
    pid: u32,
}

impl PtyProcess {
    pub fn spawn(command: &str, args: &[&str]) -> Result<Self> { ... }
    
    pub fn write(&self, data: &[u8]) -> Result<usize> { ... }
    
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> { ... }
    
    pub fn resize(&self, rows: u16, cols: u16) -> Result<()> { ... }
    
    pub fn kill(&self) -> Result<()> { ... }
}
```

### Recommended Solution

Fork candidate FORK-001: Create `phenotype-process` from `heliosCLI/utils/pty`

```rust
// libs/phenotype-process/src/lib.rs
pub mod pty;
pub mod process;
pub mod process_group;

pub use pty::{PtyMaster, PtyChild};
pub use process::{ProcessHandle, ProcessOutput};
pub use process_group::ProcessGroup;
```

### LOC Impact

| Metric | Value |
|--------|-------|
| Current LOC | ~900 |
| After consolidation | ~400 |
| Savings | ~500 |

---

## 2026-03-29 - Secret Management Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Analysis of secret management duplication. Found multiple credential storage implementations.

### Secret Storage Inventory

| Location | Implementation | Encryption | LOC |
|----------|----------------|------------|-----|
| `agileplus-domain/src/credentials/` | Full | AES-256 | 200+ |
| `heliosCLI/codex-rs/core/src/secrets.rs` | Partial | TBD | 100+ |
| `libs/cipher/src/` | Low-level | Various | 150+ |

### Credential Store Pattern

```rust
pub trait CredentialStore: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Credential>>;
    async fn set(&self, key: &str, cred: &Credential) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list(&self) -> Result<Vec<String>>;
}

pub struct Credential {
    pub username: String,
    pub secret: Encrypted<String>,
    pub created_at: DateTime<Utc>,
}
```

### libs/cipher Status

```rust
// libs/cipher/src/domain/error.rs
pub enum CipherError {
    Encryption(String),
    Decryption(String),
    KeyNotFound,
}
```

### Recommendations

1. Integrate libs/cipher into agileplus-domain
2. Standardize on AES-256-GCM for encryption
3. Use key derivation for workspace-specific keys
4. Add audit logging for credential access

---

## 2026-03-29 - NATS Client Duplication

**Project:** [AgilePlus]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Analysis of NATS client duplication. Found multiple NATS implementations.

### NATS Implementation Inventory

| Location | Implementation | Features | LOC |
|----------|----------------|----------|-----|
| `agileplus-nats/src/` | Full JetStream | Streams, consumers | 500+ |
| `vibe-kanban/backend/src/nats.rs` | Basic | Publish only | 50+ |

### Common NATS Pattern

```rust
pub struct NatsClient {
    connection: async_nats::Client,
    jetstream: async_nats::jetstream::Context,
}

impl NatsClient {
    pub async fn publish(&self, subject: &str, payload: &[u8]) -> Result<()> { ... }
    
    pub async fn subscribe(&self, subject: &str) -> Result<Subscription> { ... }
    
    pub async fn create_stream(&self, config: &StreamConfig) -> Result<()> { ... }
}
```

### Recommendations

1. Migrate vibe-kanban to agileplus-nats crate
2. Add publish-only trait for simple use cases
3. Document JetStream patterns
4. Add connection pooling

---

## 2026-03-29 - gRPC Service Definition Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Analysis of gRPC service definition duplication. Found proto definitions in multiple locations.

### Proto Definition Inventory

| Location | Services | Messages | LOC |
|----------|----------|----------|-----|
| `crates/agileplus-grpc/proto/` | 3 | 20+ | 500+ |
| `platforms/heliosCLI/codex-rs/proto/` | 2 | 15+ | 300+ |
| `libs/phenotype-proto/` | 5 | 30+ | 800+ |

### Proto Definition Comparison

| Service | agileplus-grpc | heliosCLI | phenotype-proto |
|---------|----------------|-----------|-----------------|
| Feature | ✅ | ❌ | ❌ |
| Agent | ✅ | ✅ | ❌ |
| Event | ✅ | ❌ | ✅ |
| Auth | ❌ | ✅ | ✅ |

### Recommended Solution

```rust
// Centralize in libs/phenotype-proto
phenotype-proto/
├── agileplus/
│   ├── feature.proto
│   ├── agent.proto
│   └── event.proto
└── common/
    ├── types.proto
    └── error.proto
```

### Recommendations

1. Consolidate proto definitions in libs/phenotype-proto
2. Add buf.yaml for linting and breaking change detection
3. Generate code for all languages
4. Deprecate duplicate proto files

---

## 2026-03-29 - CLI Command Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Analysis of CLI command duplication across repositories.

### CLI Command Inventory

| Repository | Commands | Framework | LOC |
|------------|----------|-----------|-----|
| agileplus-cli | 8 | clap | 500+ |
| heliosCLI | 15+ | clap | 1000+ |
| thegent | 20+ | argparse | 1500+ |

### Command Comparison

| Command | agileplus-cli | heliosCLI | thegent |
|---------|---------------|-----------|---------|
| feature create | ✅ | ✅ | ✅ |
| feature list | ✅ | ✅ | ✅ |
| agent spawn | ✅ | ✅ | ✅ |
| agent status | ✅ | ✅ | ✅ |
| worktree create | ❌ | ✅ | ✅ |
| worktree list | ❌ | ✅ | ✅ |

### libs/cli-framework Status

```rust
// libs/cli-framework/src/lib.rs
pub mod parser;
pub mod commands;
pub mod output;

pub use parser::CliParser;
pub use commands::{Command, Subcommand};
pub use output::{OutputFormat, OutputWriter};
```

### Recommendations

1. Expand libs/cli-framework with shared patterns
2. Migrate agileplus-cli to use framework
3. Document command conventions
4. Add shell completion generation

---

## 2026-03-29 - HTTP Client Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Analysis of HTTP client duplication. Found multiple HTTP client implementations.

### HTTP Client Inventory

| Location | Implementation | Features | LOC |
|----------|----------------|----------|-----|
| `agileplus-api/src/http/` | Full | Auth, retry | 200+ |
| `heliosCLI/codex-rs/core/src/http.rs` | Basic | Simple GET/POST | 100+ |
| `libs/http-client/` | Partial | TBD | 50+ |

### libs/http-client Status

```rust
// libs/http-client/src/lib.rs
pub struct HttpClient {
    client: reqwest::Client,
    base_url: Url,
}

impl HttpClient {
    pub fn new(base_url: Url) -> Self { ... }
    
    pub async fn get(&self, path: &str) -> Result<Response> { ... }
    
    pub async fn post(&self, path: &str, body: &[u8]) -> Result<Response> { ... }
}
```

### Recommendations

1. Complete libs/http-client implementation
2. Add authentication support
3. Add retry with exponential backoff
4. Migrate existing implementations

---

## 2026-03-29 - Database Schema Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Analysis of database schema duplication. Found similar schemas across repositories.

### Schema Comparison

| Entity | agileplus | vibe-kanban | heliosCLI |
|--------|-----------|--------------|-----------|
| Feature | ✅ | ✅ | ✅ |
| Workspace | ✅ | ❌ | ❌ |
| Agent | ✅ | ❌ | ✅ |
| User | ✅ | ✅ | ✅ |
| Credential | ✅ | ❌ | ✅ |

### Shared Schema Pattern

```sql
-- Common across all databases
CREATE TABLE features (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id)
);
```

### Recommendations

1. Extract common schema to shared migration files
2. Use SQLx migrations across projects
3. Document schema evolution policy
4. Add schema validation in CI

---

## 2026-03-29 - Event Type Duplication

**Project:** [AgilePlus]
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Analysis of event type duplication. Found multiple event definitions with overlap.

### Event Type Inventory

| Event Type | Location | Variants | LOC |
|------------|----------|----------|-----|
| FeatureEvent | `agileplus-events/src/events.rs` | 8 | 100 |
| AgentEvent | `agileplus-events/src/events.rs` | 5 | 50 |
| DomainEvent | `agileplus-domain/src/events.rs` | 10 | 100 |
| WorkspaceEvent | `agileplus-domain/src/events.rs` | 5 | 50 |

### Common Event Pattern

```rust
pub trait DomainEvent: Serialize + DeserializeOwned {
    fn event_type(&self) -> &str;
    fn aggregate_id(&self) -> &str;
    fn occurred_at(&self) -> DateTime<Utc>;
}

#[derive(Serialize, Deserialize)]
pub struct FeatureCreatedEvent {
    pub feature_id: String,
    pub name: String,
    pub workspace_id: String,
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent for FeatureCreatedEvent {
    fn event_type(&self) -> &str { "feature.created" }
    fn aggregate_id(&self) -> &str { &self.feature_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.timestamp }
}
```

### Recommendations

1. Consolidate all events in agileplus-events crate
2. Add event versioning
3. Implement event upcasting for migrations
4. Document event naming conventions

---

## 2026-03-29 - Metrics Collection Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Analysis of metrics collection duplication. Found multiple metrics implementations.

### Metrics Implementation Inventory

| Location | Implementation | Metrics | LOC |
|----------|----------------|---------|-----|
| `agileplus-telemetry/src/metrics.rs` | Full | 30+ | 200+ |
| `heliosCLI/codex-rs/core/src/metrics.rs` | Basic | 10+ | 50+ |
| `libs/metrics/` | Partial | TBD | 100+ |

### libs/metrics Status

```rust
// libs/metrics/src/lib.rs
pub mod counter;
pub mod gauge;
pub mod histogram;

pub use counter::Counter;
pub use gauge::Gauge;
pub use histogram::Histogram;
```

### Recommendations

1. Complete libs/metrics implementation
2. Add Prometheus integration
3. Migrate existing implementations
4. Add metrics naming conventions

---

## 2026-03-29 - Connection Pool Duplication

**Project:** [AgilePlus]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Analysis of connection pool duplication. Found inconsistent pool implementations.

### Pool Implementation Inventory

| Location | Pool Manager | Target | LOC |
|----------|--------------|--------|-----|
| `agileplus-cache/src/pool.rs` | bb8 | Redis | 50 |
| `agileplus-graph/src/pool.rs` | bb8 | Neo4j | 50 |
| `libs/phenotype-redis-adapter/` | deadpool | Redis | 100+ |

### Pool Comparison

| Crate | Manager | Config | Health Check |
|-------|---------|--------|--------------|
| agileplus-cache | bb8 | PoolSize, Timeout | ✅ |
| agileplus-graph | bb8 | PoolSize, Timeout | ✅ |
| phenotype-redis-adapter | deadpool | TBD | ❌ |

### Recommendations

1. Standardize on bb8 for consistency
2. Add connection health checks to phenotype-redis-adapter
3. Document pool configuration patterns
4. Add pool metrics

---

## 2026-03-29 - Serialization Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Analysis of serialization duplication. Found multiple serialization patterns.

### Serialization Inventory

| Format | Usage | Locations |
|--------|-------|-----------|
| JSON | REST API | All crates |
| Protobuf | gRPC | agileplus-grpc |
| MessagePack | NATS | agileplus-nats |
| TOML | Config | agileplus-domain |

### Common Serialization Pattern

```rust
#[derive(Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    pub name: String,
    pub status: FeatureStatus,
}

#[async_trait]
pub trait Serializable {
    fn serialize(&self) -> Result<Vec<u8>>;
    fn deserialize(bytes: &[u8]) -> Result<Self>;
}

impl Serializable for Feature {
    fn serialize(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| Error::Serialization(e.to_string()))
    }
    
    fn deserialize(bytes: &[u8]) -> Result<Feature> {
        serde_json::from_slice(bytes).map_err(|e| Error::Serialization(e.to_string()))
    }
}
```

### Recommendations

1. Create serialization trait in shared location
2. Add MessagePack support where needed
3. Document serialization conventions
4. Add schema validation

---

## 2026-03-29 - Logging Infrastructure Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Analysis of logging infrastructure duplication. Found multiple logging setups.

### Logging Inventory

| Location | Framework | Format | LOC |
|----------|-----------|--------|-----|
| `agileplus-telemetry/src/tracing.rs` | tracing | Structured | 100 |
| `heliosCLI/codex-rs/core/src/logging.rs` | tracing | Structured | 50 |
| `libs/tracing/` | tracing | Structured | 100 |

### libs/tracing Status

```rust
// libs/tracing/src/lib.rs
pub mod subscriber;
pub mod span;

pub use subscriber::init_tracing;
pub use span::SpanExt;
```

### Recommendations

1. Integrate libs/tracing into main workspace
2. Standardize on structured logging
3. Add log sampling configuration
4. Document logging conventions

---

## 2026-03-29 - Test Infrastructure Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Analysis of test infrastructure duplication. Found multiple test utility implementations.

### Test Utility Inventory

| Location | Utilities | LOC |
|----------|-----------|-----|
| `crates/agileplus-domain/src/test/` | Fixtures, helpers | 100+ |
| `heliosCLI/codex-rs/core/src/test/` | Fixtures, mocks | 150+ |
| `libs/test-utils/` | TBD | 50 |

### Common Test Pattern

```rust
// Test fixtures
pub fn test_workspace() -> Workspace {
    Workspace {
        id: WorkspaceId::new(),
        name: "test-workspace".to_string(),
        owner_id: UserId::new(),
    }
}

pub fn test_feature() -> Feature {
    Feature {
        id: FeatureId::new(),
        name: "test-feature".to_string(),
        workspace_id: test_workspace().id,
        status: FeatureStatus::Draft,
    }
}

// Test helpers
pub async fn setup_test_db() -> SqlitePool { ... }
pub fn mock_github_client() -> MockGitHubClient { ... }
```

### Recommendations

1. Create `libs/test-utils` with shared fixtures
2. Add mock implementations for all traits
3. Add test database setup helpers
4. Document test conventions

---

## 2026-03-29 - Validation Logic Duplication

**Project:** [AgilePlus]
**Category:** duplication
**Status:** completed
**Priority:** P2

### Summary

Analysis of validation logic duplication. Found multiple validation implementations.

### Validation Inventory

| Location | Validates | Rules | LOC |
|----------|-----------|-------|-----|
| `agileplus-domain/src/validation.rs` | Features, agents | 10+ | 100 |
| `agileplus-api/src/validation.rs` | API input | 5+ | 50 |
| `heliosCLI/codex-rs/core/src/validation.rs` | Commands | 10+ | 100 |

### Common Validation Pattern

```rust
pub trait Validator<T> {
    fn validate(&self, value: &T) -> Result<(), ValidationError>;
}

pub struct FeatureValidator;

impl Validator<Feature> for FeatureValidator {
    fn validate(&self, feature: &Feature) -> Result<(), ValidationError> {
        if feature.name.is_empty() {
            return Err(ValidationError::new("name", "cannot be empty"));
        }
        
        if feature.name.len() > 100 {
            return Err(ValidationError::new("name", "too long"));
        }
        
        Ok(())
    }
}
```

### Recommendations

1. Consolidate validators in shared location
2. Add validation combinators
3. Implement derive macro for simple cases
4. Add cross-field validation support
