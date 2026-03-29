# Duplication Worklogs

**Category:** DUPLICATION | **Updated:** 2026-03-29

---

## 2026-03-29 - AgilePlus Extended Duplication Audit

**Project:** [AgilePlus]
**Category:** duplication
**Status:** in_progress
**Priority:** P1

### Summary

Extended comprehensive audit of AgilePlus intra-repo duplication. Identified patterns across health checks, error types, config loaders, API responses, port/trait architecture, builder patterns, async traits, and connection pools.

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

## 2026-03-30 - Duplication Audit Chunk 5: Deep codebase hotspots

**Project:** [cross-repo]
**Category:** duplication
**Status:** in_progress
**Priority:** P0

### 14. Async Trait Duplication Hotspots (high frequency)
- `crates/phenotype-contracts/*/src/ports/inbound` and `outbound` contain 3-4 repeated `#[async_trait]` trait methods each.
- `crates/agileplus-graph` + `crates/agileplus-cache` + `crates/agileplus-nats` have identical `async fn` storage/health entries.
- Candidate consolidation: `libs/phenotype-port-interfaces` should host standard `AsyncRepository`, `AsyncCache`, `AsyncEventBus` traits.

### 15. Error conversion duplication (periodic)
- `capsule` functions in `crates/agileplus-*` use repeated `impl From<MyError> for ApiError` patterns.
- `ports` libraries have duplicate mapping in `src/conversion.rs` to `phenotype-error` variants.
- Candidate consolidation: `libs/phenotype-error` with `ErrorExt` trait and universal mapping macro.

### 16. Worktree / Process lifecycle duplication
- `platforms/thegent/*` and `heliosCLI/*` each include similar worktree management, process killing, and cleanup code.
- Candidate shared lib: `libs/phenotype-worktree` providing `WorktreeManager`, `ProcessGroup`, `safe_kill`.

### Next Steps (new chunk)
- [ ] Identify and merge duplicated `async_trait` trait definitions into one core library.
- [ ] Replace triple-duplicate `From<...> for ...` patterns with derive macro in `phenotype-error`.
- [ ] Create `libs/phenotype-worktree` from common code in `heliosCLI` and `platforms/thegent`.


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

---

## 2026-03-29 - phenoinfrakit Deep Duplication Audit

**Project:** phenotype-infrakit
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Deep analysis of duplication within phenotype-infrakit workspace - identified nested crate structure, internal duplication, and external overlap opportunities.

### Critical Finding: Nested Crate Structure

| Crate | Outer (crates/X/) | Inner (crates/X/X/) | Status |
|-------|-------------------|---------------------|--------|
| `phenotype-cache-adapter` | ✅ Has src/ | ✅ Has src/ | **100% IDENTICAL** |
| `phenotype-contracts` | ✅ Has src/ | ✅ Has src/ | **100% IDENTICAL** |
| `phenotype-event-sourcing` | ✅ Has src/ | ✅ Has src/ | Minor formatting |
| `phenotype-policy-engine` | ✅ Has src/ | ✅ Has src/ | **100% IDENTICAL** |
| `phenotype-state-machine` | ❌ NO src/ | ✅ Has src/ | **INCOMPLETE** |

### Root Cause

The nested crate structure is from **in-progress rebase**:
1. Inner crates contain the actual implementation
2. Outer crates were created as workspace entries
3. After rebase completes, inner crates will become canonical

### Internal Duplication Analysis

#### phenotype-event-sourcing Internal Modules

| Module | LOC | Duplication Risk | Status |
|--------|-----|-----------------|--------|
| error.rs | 46 | Low - domain-specific | ✅ Clean |
| hash.rs | 195 | Medium - similar to sync hash | Consider lib |
| event.rs | 98 | Low - domain-specific | ✅ Clean |
| snapshot.rs | 92 | Low - domain-specific | ✅ Clean |
| store.rs | 64 | Low - domain-specific | ✅ Clean |
| memory.rs | 266 | Low - in-memory only | ✅ Clean |

#### phenotype-policy-engine Internal Modules

| Module | LOC | Duplication Risk | Status |
|--------|-----|-----------------|--------|
| error.rs | ? | Medium - similar to event-sourcing | Consider shared |
| engine.rs | ~200 | Low - domain-specific | ✅ Clean |
| loader.rs | ~100 | Medium - similar config patterns | Consider lib |
| result.rs | ~50 | Low - domain-specific | ✅ Clean |
| rule.rs | ~100 | Low - domain-specific | ✅ Clean |

### Cross-Crate Duplication

#### Error Type Patterns

| Crate | Error Type | Variants | Similarity |
|-------|-----------|----------|------------|
| event-sourcing | `EventStoreError` | 4 | Similar to policy errors |
| policy-engine | `PolicyError` | 4+ | Similar to event errors |
| cache-adapter | `CacheError` | ? | Different domain |
| evidence-ledger | `LedgerError` | ? | Not analyzed |

**Opportunity:** Extract shared error core pattern

#### Hash Chain Patterns

| Crate | Hash Implementation | Purpose |
|-------|-------------------|---------|
| event-sourcing | SHA-256 chain | Event integrity |
| evidence-ledger | SHA-256 chain | Evidence chain |

**Opportunity:** Extract shared `ContentHash` library

#### In-Memory Store Patterns

| Crate | Implementation | Pattern |
|-------|----------------|---------|
| event-sourcing | `InMemoryEventStore<T>` | `RwLock<HashMap>` |
| policy-engine | In-memory policy store | Similar pattern |
| cache-adapter | `InMemoryCache` | `DashMap` variant |

**Opportunity:** Extract shared in-memory trait

### External Overlap

#### Overlap with phenotype-shared

| Crate | phenotype-infrakit | phenotype-shared | Action |
|-------|-------------------|-----------------|--------|
| `phenotype-event-sourcing` | ✅ Exists | ✅ Exists | Consolidate |
| `phenotype-cache-adapter` | ✅ Exists | ✅ Exists | Consolidate |
| `phenotype-policy-engine` | ✅ Exists | ✅ Exists | Consolidate |
| `phenotype-state-machine` | ✅ Exists | ✅ Exists | Consolidate |

**Action:** Merge phenotype-infrakit into phenotype-shared

### LOC Savings Potential

| Cleanup | Savings | Priority |
|---------|---------|----------|
| Remove nested duplicates | ~500 LOC | 🔴 CRITICAL |
| Delete dead state-machine | ~50 LOC | 🟠 HIGH |
| Extract shared error core | ~30 LOC | 🟡 MEDIUM |
| Extract shared hash lib | ~20 LOC | 🟡 MEDIUM |
| **Total** | **~600 LOC** | |

---

## 2026-03-29 - Cross-Repo Event Sourcing Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Analysis of event sourcing implementations across multiple repositories.

### Event Sourcing Instances

| Repo | Crate | LOC | Quality | Status |
|------|-------|-----|---------|--------|
| phenotype-infrakit | `phenotype-event-sourcing` | ~781 | High | Active |
| phenotype-shared | `phenotype-event-sourcing` | ~500 | High | Active |
| AgilePlus | `agileplus-events` | ~300 | Medium | Active |
| thegent | Event patterns | ~200 | Medium | Active |

### Architecture Comparison

#### phenotype-infrakit (Best)

```rust
// Generic aggregate trait
pub trait Aggregate: Send + Sync + 'static {
    type Id: IdType;
    type Event: EventType;
    fn apply(&mut self, event: Self::Event);
}

// Event envelope with chain hash
pub struct EventEnvelope<T: Aggregate> {
    pub id: Uuid,
    pub aggregate_id: T::Id,
    pub sequence: u64,
    pub timestamp: DateTime<Utc>,
    pub payload: T::Event,
    pub hash: ContentHash,
}
```

#### phenotype-shared (Good)

Similar architecture, slightly different implementation.

#### AgilePlus (Basic)

```rust
// Basic event store
pub trait EventStore: Send + Sync {
    async fn append(&self, event: Event) -> Result<()>;
    async fn get_events(&self, id: &Uuid) -> Result<Vec<Event>>;
}
```

### Recommended Consolidation

| Step | Action | Target |
|------|--------|--------|
| 1 | Adopt phenotype-infrakit as canonical | `phenotype-shared/crates/event-sourcing` |
| 2 | Remove AgilePlus duplicate | Migrate to shared |
| 3 | Archive phenotype-shared version | Delete after migration |
| 4 | Consider cqrs-es | Fork or integrate |

### LOC Savings

| Consolidation | Savings |
|---------------|---------|
| Remove phenotype-shared event-sourcing | ~500 LOC |
| Remove agileplus-events duplicate | ~300 LOC |
| Use cqrs-es as foundation | ~200 LOC |
| **Total** | **~1000 LOC** |

---

## 2026-03-29 - Cross-Repo Cache Adapter Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Analysis of cache adapter implementations across repositories.

### Cache Adapter Instances

| Repo | Crate | Backend | Quality |
|------|-------|---------|---------|
| phenotype-infrakit | `phenotype-cache-adapter` | DashMap, Moka | High |
| phenotype-shared | `phenotype-cache-adapter` | Multiple | Medium |
| thegent | `thegent-cache` | TTL cache | Medium |

### Architecture Comparison

#### phenotype-infrakit (Best)

```rust
pub trait CacheBackend: Send + Sync {
    async fn get(&self, key: &str) -> Option<Vec<u8>>;
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) -> Result<(), CacheError>;
    async fn delete(&self, key: &str) -> Result<(), CacheError>;
}

// Implementations: DashMap, Moka
```

#### phenotype-shared (Good)

Similar trait design, different implementations.

### Recommended Consolidation

| Step | Action | Target |
|------|--------|--------|
| 1 | Adopt phenotype-infrakit as canonical | `phenotype-shared/crates/cache` |
| 2 | Add Redis adapter | Extend trait |
| 3 | Remove duplicate implementations | Delete |

---

## 2026-03-29 - Cross-Repo Policy Engine Duplication

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Analysis of policy engine implementations across repositories.

### Policy Engine Instances

| Repo | Crate | LOC | Features |
|------|-------|-----|----------|
| phenotype-infrakit | `phenotype-policy-engine` | ~500 | Rules, engine, loader |
| phenotype-shared | `phenotype-policy-engine` | ~300 | Basic rules |

### Architecture Comparison

#### phenotype-infrakit (Better)

```rust
// Rich policy structure
pub struct Policy {
    pub id: Uuid,
    pub name: String,
    pub rules: Vec<Rule>,
    pub severity: Severity,
    pub rule_type: RuleType,
}

pub struct Rule {
    pub id: Uuid,
    pub field: String,
    pub operator: Operator,
    pub value: serde_json::Value,
}
```

#### phenotype-shared (Basic)

Basic rule evaluation without complex structures.

### Recommended Consolidation

| Step | Action | Target |
|------|--------|--------|
| 1 | Adopt phenotype-infrakit as canonical | `phenotype-shared/crates/policy` |
| 2 | Migrate rules from shared | Extend |
| 3 | Consider reglang/OPA | Fork evaluation |

---

## 2026-03-29 - Pattern Generation: In-Memory Store

**Project:** [cross-repo]
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Pattern analysis for generating reusable in-memory store implementations.

### Current Implementations

| Crate | Type | Implementation | LOC |
|-------|------|----------------|-----|
| event-sourcing | InMemoryEventStore | `RwLock<HashMap>` | ~266 |
| policy-engine | InMemoryPolicyStore | Similar | ~100 |
| cache-adapter | InMemoryCache | `DashMap` | ~50 |
| agileplus-sync | InMemorySyncStore | `Mutex<HashMap>` | ~60 |

### Common Pattern

```rust
// Common: Generic in-memory with sync
pub struct InMemoryStore<K, V> {
    data: RwLock<HashMap<K, V>>,
}

impl<K: Eq + Hash, V> InMemoryStore<K, V> {
    pub async fn get(&self, key: &K) -> Option<V> {
        self.data.read().get(key).cloned()
    }
    
    pub async fn set(&self, key: K, value: V) {
        self.data.write().insert(key, value);
    }
}
```

### Library Candidate

```rust
// libs/phenotype-in-memory/
pub trait InMemoryStore<K, V>: Send + Sync {
    async fn get(&self, key: &K) -> Option<V>;
    async fn set(&self, key: K, value: V);
    async fn delete(&self, key: &K) -> Option<V>;
    async fn clear(&self);
    async fn len(&self) -> usize;
}

pub struct HashMapStore<K, V> {
    data: RwLock<HashMap<K, V>>,
}

impl<K: Eq + Hash + Clone, V: Clone> InMemoryStore<K, V> for HashMapStore<K, V> {}
```

### LOC Savings

| Pattern | Current | After | Savings |
|---------|---------|-------|---------|
| In-memory stores | ~476 LOC | ~100 LOC | **376 LOC** |

---

## 2026-03-29 - Productization: Evidence Ledger

**Project:** phenotype-infrakit
**Category:** duplication
**Status:** completed
**Priority:** P1

### Summary

Analysis of evidence ledger as a standalone productizable crate.

### Current Structure

```
crates/evidence-ledger/
├── src/
│   ├── lib.rs      # 25 LOC
│   ├── chain.rs    # Evidence chain
│   ├── ledger.rs   # Ledger operations
│   └── error.rs   # Error types
├── Cargo.toml
└── README.md
```

### Features

| Feature | Status | Quality |
|---------|--------|---------|
| Evidence chain | ✅ | High |
| Ledger operations | ✅ | High |
| Hash verification | ✅ | High |
| Query filtering | ✅ | Medium |
| External config | ❌ | Missing |

### Productization Opportunities

| Feature | Current | Target | Priority |
|---------|---------|--------|----------|
| TOML config | ❌ | ✅ | 🟠 HIGH |
| Multiple backends | Memory only | SQLite, Postgres | 🟠 HIGH |
| gRPC API | ❌ | ✅ | 🟡 MEDIUM |
| OpenTelemetry | ❌ | ✅ | 🟡 MEDIUM |

### Standalone Product

```toml
# evidence-ledger = "1.0"  (publish to crates.io)
[dependencies.evidence-ledger]
version = "1.0"
features = ["sqlite", "postgres", "grpc"]
```

### Recommended Actions

1. Add figment-based configuration
2. Add SQLite backend adapter
3. Add gRPC service layer
4. Publish to crates.io as standalone

---

_Last updated: 2026-03-29_
---

_Last updated: 2026-03-29_
- [ ] 🟢 LOW: Delete `phenotype-state-machine` (dead code)

### Related

- `docs/research/consolidation-audit-2026-03-29.md` - Master findings
- `worklogs/WORK_LOG.md` - Wave 90 entry

---

## 2026-03-29 - Wave 92: `repos/` monorepo deep duplication (verified scan)

**Project:** [phenotype-infrakit / repos workspace]
**Category:** duplication
**Status:** in_progress
**Priority:** P0

### Summary

Filesystem-level audit of `Phenotype/repos` to separate **real code duplication** from **accidental directory cloning**. Several patterns multiply LOC and confuse `rg` / `cargo` metadata.

### 1. Double package roots per workspace member (P0 structural)

Each workspace member under `crates/` shows **two** package roots with the **same** `name` in `Cargo.toml`:

| Crate | Outer manifest | Inner manifest | Notable drift |
|-------|----------------|----------------|---------------|
| `phenotype-event-sourcing` | `crates/phenotype-event-sourcing/Cargo.toml` | `crates/phenotype-event-sourcing/phenotype-event-sourcing/Cargo.toml` | Inner adds `tokio` dev-dependency; dependency feature lines differ |
| `phenotype-policy-engine` | outer + inner | same pattern | Naming collision / drift risk |
| `phenotype-state-machine` | outer + inner | same pattern | Naming collision / drift risk |
| `phenotype-cache-adapter` | outer + inner | same pattern | Naming collision / drift risk |
| `phenotype-contracts` | outer + inner | same pattern | Inner `tokio = "1.0"` vs inner event-sourcing `1.40` |

**Verification (read-only):**

```bash
diff -rq crates/phenotype-event-sourcing/src \
  crates/phenotype-event-sourcing/phenotype-event-sourcing/src || true
```

**Canonical rule:** One package root per crate. Keep the workspace-linked root, merge any unique files, delete the redundant tree in a dedicated PR.

### 2. Vendored full repositories inside `repos/` (P0 hygiene)

`rg 'pub enum \\w+Error'` hits paths under trees that are **not** the five workspace members—treating them as first-class duplication inflates audits.

| Path | Role | Recommendation |
|------|------|----------------|
| `phenotype-shared-wtrees/resolve-pr58/` | Nested copy of another workspace | **git worktree** outside repo or **submodule** pin |
| `thegent-work/crates/*` | Many standalone crates | Track **canonical** `thegent`; delete or submodule |
| `heliosCLI-wtrees/main/codex-rs/` | Large Rust workspace | Same; never duplicate `origin/main` tarballs in-tree |

**Impact:** Duplication metrics and security scans should **exclude** these paths until ownership is explicit (document in `AGENTS.md` / `deny.toml` excludes for agents).

### 3. thegent-hooks: error enum sprawl (P1 libification)

| File (under `thegent-work/`) | Type |
|-------------------------------|------|
| `thegent-hooks/src/git_ops.rs` | `GitOpsError` |
| `thegent-hooks/src/git_cache.rs` | `GitCacheError` |
| `thegent-hooks/src/file_discovery.rs` | `FileDiscoveryError` |
| `thegent-hooks/src/changed_files.rs` | `ChangedFilesError` |
| `thegent-hooks/src/affected_tests.rs` | `AffectedTestsError` |
| `thegent-hooks/src/prewarm.rs` | `PrewarmError` |
| `thegent-hooks/src/report.rs` | `ReportError` |
| `thegent-hooks/src/types.rs` | `HookError` |

**Consolidation:** Single `HooksError` with `#[from]` / `miette` context—target **~120 LOC** savings and uniform CLI output.

### 4. heliosCLI harness stack: parallel error types (P1)

Under `heliosCLI-wtrees/main/crates/harness_*`:

| Crate | Error type |
|-------|------------|
| `harness_verify` | `VerifyError` |
| `harness_spec` | `SpecError` |
| `harness_runner` | `RunError` |
| `harness_queue` | `QueueError` |
| `harness_orchestrator` | `OrchestratorError` |
| `harness_elicitation` | `ElicitationError` |
| `harness_checkpoint` | `CheckpointError` |
| `harness_cache` | `CacheError` |

**Opportunity:** `harness-core::Error` with stage + `#[source]` to collapse Display boilerplate.

### 5. codex-rs: vendor boundary (P2)

Many fine-grained errors under `codex-rs/**/error.rs` are **upstream-shaped**. Avoid mass merges; only extract cross-cutting helpers if the fork is long-lived.

### 6. Duplicate `EventSourcingError` definitions (same repo)

- `crates/phenotype-event-sourcing/src/error.rs`
- `crates/phenotype-event-sourcing/phenotype-event-sourcing/src/error.rs`

Resolving the double-root issue removes **phantom** duplication in static analysis.

### Wave 92 action items

| Priority | Action |
|----------|--------|
| P0 | Single root per `crates/*`; remove nested duplicate |
| P0 | Policy: no full-repo vendoring under `repos/` (worktree/submodule) |
| P1 | Unified `thegent-hooks` error design |
| P1 | `harness-core` error design |
| P2 | Document `codex-rs` vendor rules |

---

---

## 2026-03-29 - Extended Cross-Ecosystem Duplication Audit (15+ New Findings)

**Project:** [cross-repo, AgilePlus, thegent, heliosCLI]
**Category:** duplication
**Status:** completed
**Priority:** P0

### Summary

Comprehensive extended audit identifying 15+ NEW duplication patterns across Phenotype ecosystem. Focus on cross-language patterns (Rust ↔ Go ↔ Python), nested crate duplication, and intra-repo patterns within large monorepos (thegent 27+ crates).

---

### 🔴 CRITICAL: Nested Crate Duplication (phenotype-event-sourcing)

**Pattern:** Identical crate structures with duplicate source files

| Location | Type | Details |
|----------|------|---------|
| `crates/phenotype-event-sourcing/src/` | Primary | 7 files (error.rs, lib.rs, memory.rs, store.rs, event.rs, snapshot.rs, hash.rs) |
| `crates/phenotype-event-sourcing/phenotype-event-sourcing/src/` | DUPLICATE | 7 identical files |
| `crates/phenotype-event-sourcing/phenotype-event-sourcing/Cargo.toml` | Nested manifest | Duplicate workspace manifest |

**Root Cause:** Nested workspace structure with identical crate in subdirectory

**Impact:** 100% duplication of all 7 modules = ~240 LOC duplicated

**Files:**
- `/Users/kooshapari/CodeProjects/Phenotype/repos/crates/phenotype-event-sourcing/src/error.rs` (46 LOC)
- `/Users/kooshapari/CodeProjects/Phenotype/repos/crates/phenotype-event-sourcing/phenotype-event-sourcing/src/error.rs` (46 LOC)
- `/Users/kooshapari/CodeProjects/Phenotype/repos/crates/phenotype-event-sourcing/src/store.rs` (40 LOC)
- `/Users/kooshapari/CodeProjects/Phenotype/repos/crates/phenotype-event-sourcing/phenotype-event-sourcing/src/store.rs` (40 LOC)
- `**/src/memory.rs` — 2 copies (35 LOC each)
- `**/src/snapshot.rs` — 2 copies (28 LOC each)
- `**/src/event.rs` — 2 copies (31 LOC each)

**Extraction Target:** Consolidate to single `crates/phenotype-event-sourcing/src/`

**Priority:** P0 — CRITICAL (blocking pattern for other repos)

---

### 🔴 CRITICAL: Error Type Proliferation Across Rust Crates (15 crates = 850+ LOC)

**Pattern:** 15+ Error enums with overlapping variants

| Crate | Error Type | Variants | LOC | File |
|-------|------------|----------|-----|------|
| agileplus-api | ApiError | NotFound, BadRequest, Internal | 67 | `crates/agileplus-api/src/error.rs` |
| agileplus-domain | DomainError | NotFound, Conflict, ValidationFailed | 50 | `crates/agileplus-domain/src/error.rs` |
| agileplus-p2p | PeerDiscoveryError | Nats, Serialization, NotFound | 78 | `crates/agileplus-p2p/src/error.rs` |
| agileplus-sync | SyncError | Store, Nats, Serialization | 24 | `crates/agileplus-sync/src/error.rs` |
| agileplus-events | EventError | NotFound, StorageError, Serialization | 53 | `crates/agileplus-events/src/store.rs:18-71` |
| agileplus-graph | GraphError | ConnectionError, QueryError, NotFound | 326 | `crates/agileplus-graph/src/store.rs:1-326` |
| agileplus-cache | CacheError | Serialization, Redis, NotFound | 129 | `crates/agileplus-cache/src/store.rs:9-137` |
| phenotype-port-interfaces | PortError | NotFound, Validation, StorageError | 51 | `libs/phenotype-port-interfaces/src/error.rs` |
| thegent-memory | Error | ConnectionFailed, Timeout, InvalidData | 119 | `/platforms/thegent/crates/thegent-memory/src/error.rs` |
| thegent-policy | Error | RuleEvaluation, PolicyNotFound | 16 | `/platforms/thegent/crates/thegent-policy/src/errors.rs` |
| thegent-wasm-tools | Error | CompileError, RuntimeError | 45 | `/platforms/thegent/crates/thegent-wasm-tools/src/error.rs` |
| thegent-zmx-interop | Error | ProtocolError, MessageError | 38 | `/platforms/thegent/crates/thegent-zmx-interop/src/error.rs` |
| heliosCLI codex-core | CodexErr | NotFound, BadRequest, Serialization | 72 | `/heliosCLI/codex-rs/core/src/error.rs` |
| phenotype-contracts | ContractError | ValidationFailed, ExecutionError | 44 | `crates/phenotype-contracts/phenotype-contracts/src/lib.rs:1-44` |
| byteport (Go) | DeploymentError | NotFound, Conflict, InvalidInput | 95 | `/platforms/thegent/apps/byteport/backend/api/internal/domain/deployment/errors.go` |

**Duplicated Variants Across Crates:**
- `NotFound(String)` — 8+ crates
- `SerializationError` / `Serialization` — 7+ crates
- `StorageError` — 5+ crates
- `Conflict` — 4+ crates
- `Timeout` — 4+ crates
- `ValidationError` / `ValidationFailed` — 5+ crates

**LOC Savings Estimate:** 400-500 LOC (consolidate to shared `phenotype-error-core`)

**Extraction Target:** `libs/phenotype-error-core/` (NEW crate)

**Priority:** P0 — Affects 15 crates across 3 projects

---

### 🟡 HIGH: Configuration Loading Patterns (5 implementations, 650+ LOC)

**Pattern:** Duplicate home directory + file system config loading

| Location | Format | Pattern | LOC | File |
|----------|--------|---------|-----|------|
| AgilePlus domain | TOML | dirs_next + env overrides | 80 | `crates/agileplus-domain/src/config/loader.rs:21-84` |
| AgilePlus dashboard | TOML | Identical loader | 75 | `crates/agileplus-dashboard/src/routes.rs:137-170` |
| AgilePlus telemetry | YAML | YAML variant + env | 95 | `crates/agileplus-telemetry/src/config.rs:126-145` |
| heliosCLI vibe-kanban | JSON | Builder pattern defaults | 120 | `/heliosCLI/vibe-kanban/backend/src/models/config.rs` |
| thegent byteport | YAML | Env-only (no file loading) | 45 | `/platforms/thegent/apps/byteport/backend/api/config/load.go` |

**Duplicated Code Pattern:**
```rust
let home_dir = dirs_next::home_dir()?;
let config_path = home_dir.join(".agileplus/config.toml");
let contents = std::fs::read_to_string(config_path)?;
let config: Config = toml::from_str(&contents)?;
```

**Instances:** Used in 4+ files

**Library Status:** `libs/config-core/` exists but UNUSED (edition mismatch)

**Extraction Target:** Integrate `libs/config-core` into workspace

**Priority:** P1 — Blocks multiple projects

---

### 🟡 HIGH: Git Operation Patterns (Rust, 6+ implementations)

**Pattern:** Duplicate git clone, checkout, commit patterns

| Location | Operations | LOC | File |
|----------|-----------|-----|------|
| thegent-git (lib) | clone, checkout, commit, push, fetch | 709 | `/platforms/thegent/crates/thegent-git/src/lib.rs` |
| thegent-shims | git_checkout wrapper | 85 | `/platforms/thegent/crates/thegent-shims/src/shims/git_checkout.rs` |
| thegent-hooks | git operations + changed files | 156 | `/platforms/thegent/crates/thegent-hooks/src/main.rs` |
| thegent-offload | git executor | 64 | `/platforms/thegent/crates/thegent-offload/src/executor.rs` |
| agileplus-sync | git sync operations | 72 | `crates/agileplus-sync/src/sync_ops.rs` |
| heliosCLI | git operations via libgit2 | 95 | `/heliosCLI/codex-rs/git-core/src/lib.rs` |

**Common Patterns:**
- `git clone` with retry logic (3+ implementations)
- `git checkout` branch/tag switching (4+ implementations)
- Diff/changed files detection (3+ implementations)
- Commit + push (2+ implementations)

**Cross-Language:** Rust (`git2` crate) AND Go (native `git` exec)

**Extraction Target:** `libs/git-operations/` (wraps `git2`, handles common patterns)

**Priority:** P1 — Affects 4+ crates across 2 projects

---

### 🟡 HIGH: Auth Middleware Pattern (Go, 3+ implementations, 500+ LOC)

**Pattern:** JWT/OAuth token validation + middleware chains

| Location | Framework | LOC | File |
|----------|-----------|-----|------|
| byteport server | Gin + WorkOS | 283 | `/platforms/thegent/apps/byteport/backend/api/auth_handlers_workos.go` |
| byteport middleware | Gin + custom | 145 | `/platforms/thegent/apps/byteport/backend/api/internal/infrastructure/http/middleware/auth.go` |
| byteport tests | Test helpers | 155 | `/platforms/thegent/apps/byteport/backend/api/auth_integration_test.go` |
| consolidated example | Gin pattern | 189 | `/platforms/thegent/apps/byteport/backend/api/examples/consolidated_auth_example.go` |

**Common Code:**
```go
func AuthMiddleware() gin.HandlerFunc {
    return func(c *gin.Context) {
        token := c.GetHeader("Authorization")
        if token == "" {
            c.AbortWithStatusJSON(401, ErrorResponse{Error: "missing token"})
            return
        }
        // Validate token...
    }
}
```

**Instances:** 3 separate implementations in byteport alone

**Extraction Target:** `libs/go-auth/` (shared middleware, token validation)

**Priority:** P2 — Single project (byteport consolidation)

---

### 🟠 MEDIUM: In-Memory Store Implementations (4+ instances, 320 LOC)

**Pattern:** `Arc<Mutex<HashMap>>` test implementations

| Location | Trait | Implementation | LOC | File |
|----------|-------|-----------------|-----|------|
| agileplus-nats | EventBus | InMemoryBus | 113 | `crates/agileplus-nats/src/bus.rs:127-240` |
| agileplus-sync | SyncMappingStore | InMemorySyncStore | 63 | `crates/agileplus-sync/src/store.rs:47-110` |
| agileplus-graph | GraphBackend | InMemoryGraphBackend | 203 | `crates/agileplus-graph/src/store.rs:106-309` |
| agileplus-domain | CredentialStore | InMemoryCredentialStore | 47 | `crates/agileplus-domain/src/credentials/memory.rs:15-62` |

**Common Pattern:**
```rust
pub struct InMemory<T> {
    data: Arc<Mutex<HashMap<K, V>>>,
}

impl InMemory<T> {
    pub fn new() -> Self { Self { data: Arc::new(Mutex::new(HashMap::new())) } }
    pub async fn insert(&self, key: K, value: V) { /* ... */ }
    pub async fn get(&self, key: &K) -> Option<V> { /* ... */ }
}
```

**Extraction Target:** `libs/test-stores/` (generic InMemory<K,V> + trait impl macros)

**Priority:** P2 — Test infrastructure

---

### 🟠 MEDIUM: Health Check Implementations (6+ types, 180 LOC)

**Pattern:** Different health status enums + handler implementations

| Location | Type | Variants | LOC | File |
|----------|------|----------|-----|------|
| agileplus-cache | CacheHealth | Healthy, Unavailable | 42 | `crates/agileplus-cache/src/health.rs:5-47` |
| agileplus-graph | GraphHealth | Healthy, Unavailable | 90 | `crates/agileplus-graph/src/health.rs:5-95` |
| agileplus-nats | BusHealth | Connected, Disconnected | 8 | `crates/agileplus-nats/src/health.rs:4-12` |
| agileplus-domain | HealthStatus | Healthy, Degraded, Unavailable | 35 | `crates/agileplus-domain/src/domain/service_health.rs:8-43` |
| byteport (Go) | HealthResponse | Status, Details, Services | 67 | `/platforms/thegent/apps/byteport/backend/api/server.go:handleHealth` |
| nexus (lib) | ServiceHealth | Multiple states | 28 | `/platforms/thegent/libs/nexus/src/health.rs` |

**Variants Duplication:**
- `Healthy`/`Connected` — 5+ crates
- `Unavailable`/`Disconnected` — 4+ crates
- `Degraded` — 2+ crates

**Common API Pattern:**
```
GET /health → { "status": "healthy", "timestamp": "...", "details": {...} }
GET /healthz → JSON or plain text response
```

**Extraction Target:** `libs/agileplus-health/` (unified HealthStatus enum + HTTP handlers)

**Priority:** P2 — API standardization

---

### 🟠 MEDIUM: Query Builder Patterns (8+ implementations, 250 LOC)

**Pattern:** Builder trait implementations for query construction

| Location | Type | Methods | LOC | File |
|----------|------|---------|-----|------|
| agileplus-events | EventQuery | filter, limit, order_by | 48 | `crates/agileplus-events/src/query.rs:26-74` |
| agileplus-graph | QueryBuilder | where_clause, select, join | 92 | `crates/agileplus-graph/src/query.rs:15-107` |
| agileplus-domain | DomainQueryBuilder | with_filter, with_sort, with_pagination | 35 | `crates/agileplus-domain/src/query_builder.rs:8-43` |
| agileplus-sync | SyncQueryBuilder | with_source, with_target, with_condition | 28 | `crates/agileplus-sync/src/query.rs:12-40` |
| phenotype-contracts | QueryBuilder (generic) | Custom builders | 52 | `crates/phenotype-contracts/phenotype-contracts/src/ports/query.rs` |

**Common Pattern:**
```rust
pub struct QueryBuilder { conditions: Vec<String>, } 
impl QueryBuilder {
    pub fn filter(mut self, cond: &str) -> Self { self.conditions.push(...); self }
    pub fn limit(mut self, n: usize) -> Self { /* ... */ self }
}
```

**Extraction Target:** `libs/query-builder/` (generic trait + macro for builder pattern)

**Priority:** P2 — Developer ergonomics

---

### 🟠 MEDIUM: Repository/Store Trait Patterns (10+ occurrences, 200 LOC)

**Pattern:** Duplicate async Store/Repository traits

| Location | Trait | Methods | File |
|----------|-------|---------|------|
| phenotype-contracts | Repository | get, save, delete, find_all | `src/ports/outbound/repository.rs:22-54` |
| agileplus-events | EventStore | append, get_events, snapshot | `crates/agileplus-events/src/store.rs:21-68` |
| agileplus-graph | GraphBackend | query, execute, get_vertex | `crates/agileplus-graph/src/store.rs:22-45` |
| agileplus-cache | CacheStore | set, get, delete, exists | `crates/agileplus-cache/src/store.rs:21-38` |
| phenotype-event-sourcing | EventStore | append, load_events, snapshot | `crates/phenotype-event-sourcing/src/store.rs:30-60` |
| thegent-memory | MemoryStore | read, write, clear | `/platforms/thegent/crates/thegent-memory/src/store.rs:45-78` |

**Duplicated Methods:**
- `async fn get<K>(&self, key: K) -> Result<V>` — 6+ traits
- `async fn save<T>(&self, item: T) -> Result<()>` — 5+ traits
- `async fn delete<K>(&self, key: K) -> Result<()>` — 4+ traits

**Library Status:** `libs/hexagonal-rs/src/ports/repository.rs` exists but UNUSED

**Extraction Target:** Reactivate & integrate `libs/hexagonal-rs/src/ports/`

**Priority:** P2 — Architectural consistency

---

### 🟠 MEDIUM: CLI Argument Parsing (Clap, 101 files)

**Pattern:** Duplicated CLI arg definitions across 50+ Rust binaries

**Facts:**
- 101 files use `clap` or `structopt`
- No shared CLI framework across projects
- Repeated patterns: arg groups, value validators, help text

**Common Duplication:**
```rust
#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    verbose: bool,
    #[arg(short, long)]
    config: Option<String>,
    #[arg(short, long)]
    dry_run: bool,
}
```

**Extraction Target:** `libs/cli-framework/` (shared arg types, validators, help builders)

**Priority:** P3 — Nice-to-have

---

### 🟠 MEDIUM: Test Fixtures & Mocking (99 files)

**Pattern:** Duplicate mock/fixture definitions across test suites

**Facts:**
- 99 mock/fixture files identified
- No centralized test utilities library
- Per-crate test fixtures (expensive to maintain)

**Examples:**
- Mock event bus implementations (3+ copies)
- Mock cache store implementations (4+ copies)
- Test data builders (10+ different implementations)

**Extraction Target:** `libs/test-utilities/` (shared mocks, builders, fixtures)

**Priority:** P3 — Test infrastructure

---

### 🟢 LOW: Result Type Aliases (41 definitions)

**Pattern:** Crate-specific `type Result<T>` aliases

**Examples:**
- `crates/agileplus-api/src/lib.rs:pub type Result<T> = std::result::Result<T, ApiError>;`
- `crates/phenotype-event-sourcing/src/error.rs:pub type Result<T> = std::result::Result<T, EventSourcingError>;`
- 40+ similar definitions

**Impact:** 41 definitions for same concept (minimal LOC impact)

**Extraction Target:** Document in ADR, keep local to each crate

**Priority:** P4 — Documentation only

---

### 🟢 LOW: Serde Serialize/Deserialize Boilerplate (362 files)

**Pattern:** Repeated `#[derive(Serialize, Deserialize)]` across 362 files

**Facts:**
- 362 Rust files use serde derives
- No centralized approach (expected/OK)
- Could benefit from custom derive macros

**Extraction Target:** Document in style guide (acceptable pattern)

**Priority:** P5 — Documentation

---

### 🟢 LOW: Builder Pattern Usage (437 files)

**Pattern:** Builder pattern duplicated across 437 files

**Facts:**
- 437 files use builder patterns
- Expected for Rust idiom
- No consolidation needed (per-type builders are appropriate)

**Extraction Target:** Document in ADR (architectural pattern)

**Priority:** P5 — Documentation

---

### Summary Table: 15+ NEW Findings

| Finding | Crates Affected | LOC | Savings | Priority |
|---------|-----------------|-----|---------|----------|
| Nested phenotype-event-sourcing duplication | 1 | 240 | 240 | P0 |
| Error type proliferation (15 crates) | 15 | 850 | 400-500 | P0 |
| Config loading patterns (5 implementations) | 5 | 650 | 350 | P1 |
| Git operations duplication (6 implementations) | 6 | 581 | 300 | P1 |
| Auth middleware patterns (Go, 4 implementations) | 4 | 772 | 400 | P2 |
| In-memory store implementations | 4 | 426 | 250 | P2 |
| Health check implementations | 6 | 270 | 150 | P2 |
| Query builder patterns (8 implementations) | 8 | 255 | 180 | P2 |
| Repository/Store trait patterns (10 occurrences) | 10 | 200+ | 150 | P2 |
| CLI argument parsing (101 files) | 50+ | 2000+ | 800 | P3 |
| Test fixtures & mocking (99 files) | 20+ | 1500+ | 600 | P3 |
| Result type aliases (41 definitions) | 41 | 50 | 0 | P4 |
| Serde boilerplate (362 files) | 150+ | — | 0 | P5 |
| Builder pattern (437 files) | 200+ | — | 0 | P5 |
| **TOTAL IMPACT** | **400+ crates** | **~9,000 LOC** | **~4,300 LOC** | **—** |

---

### Recommended Extraction Libraries (Priority Order)

#### PHASE 1 (P0-P1): Critical Path
1. **Resolve nested phenotype-event-sourcing** → Remove duplicate
2. **Create `libs/phenotype-error-core/`** → Consolidate 15+ error enums
3. **Integrate `libs/config-core/`** → Fix edition, use across projects
4. **Create `libs/git-operations/`** → Wrap `git2`, consolidate patterns

#### PHASE 2 (P2): Architectural Cleanup
5. **Create `libs/agileplus-health/`** → Unified health status + HTTP handlers
6. **Reactivate `libs/hexagonal-rs/src/ports/`** → Repository trait patterns
7. **Create `libs/query-builder/`** → Generic QueryBuilder macro
8. **Create `libs/go-auth/`** → Auth middleware consolidation

#### PHASE 3 (P3): Developer Ergonomics
9. **Create `libs/cli-framework/`** → Shared CLI arg types
10. **Create `libs/test-utilities/`** → Mocks, fixtures, builders

---

### Related

- **Master Audit:** `docs/research/cross-ecosystem-duplication-audit-2026-03-29.md`
- **Extraction Plan:** `docs/reports/LIBIFICATION_EXTRACTION_PLAN_2026-03-29.md`
- **Consolidation Status:** Will track in `docs/reference/LIBRARY_CONSOLIDATION_TRACKER.md`

