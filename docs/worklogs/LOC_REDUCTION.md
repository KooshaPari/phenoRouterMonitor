# LOC Reduction & Code Optimization

**Category:** LOC_REDUCTION | **Updated:** 2026-03-29

---

## 2026-03-29 - LOC Reduction Opportunities (Expanded)

**Project:** [cross-repo]
**Category:** loc_reduction
**Status:** in_progress
**Priority:** P0

### Summary

Comprehensive LOC reduction opportunities across the Phenotype ecosystem. Target: **Quality >= Same, LOC--**

### LOC Reduction Matrix

| Pattern | Current LOC | Target LOC | Savings | Priority | Complexity |
|---------|------------|-----------|---------|----------|------------|
| Nested duplicate crates | ~1,710 | 0 | **1,710** | P0 | Low |
| Error types consolidation | ~600 | 200 | **400** | P0 | Medium |
| Config loaders | ~500 | 150 | **350** | P1 | Medium |
| Health check enums | ~140 | 60 | **80** | P1 | Low |
| In-memory test impls | ~400 | 150 | **250** | P1 | Medium |
| Async traits duplication | ~300 | 100 | **200** | P2 | High |
| HTTP client patterns | ~200 | 100 | **100** | P2 | Medium |
| Serialization boilerplate | ~150 | 50 | **100** | P2 | Low |
| **TOTAL** | **4,000** | **810** | **3,190** | | |

---

## 2026-03-29 - Code Optimization Patterns

**Project:** [cross-repo]
**Category:** optimization
**Status:** in_progress
**Priority:** P1

### Dead Code Elimination

#### Unused Dependencies (Rust)

| Crate | Downloads | Status | Action |
|-------|----------|--------|--------|
| `lru` | 500k/wk | UNUSED | Remove or use `moka` |
| `parking_lot` | 2M/wk | UNUSED | Remove if `std::sync` sufficient |
| `moka` | 500k/wk | IN USE | Keep |
| `dashmap` | 100k/wk | PARTIAL | Evaluate if needed |
| `uuid` | 3M/wk | PARTIAL | Use `cargo tree -u` to audit |

#### Unused Libraries (Edition Mismatch)

All `libs/` use `edition = "2021"` while workspace uses `edition = "2024"`:

| Library | Purpose | LOC | Action |
|---------|---------|-----|--------|
| `hexagonal-rs` | Ports & Adapters | ~300 | Migrate to edition 2024 |
| `config-core` | Config patterns | ~200 | Migrate to edition 2024 |
| `logger` | Structured logging | ~100 | Migrate to edition 2024 |
| `tracing` | Distributed tracing | ~150 | Migrate to edition 2024 |
| `metrics` | Metrics collection | ~100 | Migrate to edition 2024 |
| `hexkit` | HTTP/Persistence | ~200 | Migrate to edition 2024 |
| `cipher` | Encryption | ~150 | Archive |
| `gauge` | Benchmarking | ~50 | Archive |
| `nexus` | Service discovery | ~100 | Investigate |
| `xdd-lib-rs` | Data transformation | ~80 | Archive |
| `phenotype-state-machine` | State machine | ~100 | Archive |

### Boilerplate Reduction

#### Derive Macro Opportunities

| Pattern | Manual | With Derive | Savings |
|---------|--------|-------------|---------|
| Error types | 30-50 LOC | 5 LOC | **~40 LOC/error** |
| Builder patterns | 40-60 LOC | 10 LOC | **~45 LOC/builder** |
| JSON serialization | 20-30 LOC | 2 LOC | **~25 LOC/type** |

#### Common Trait Implementations

| Trait | Locations | Boilerplate LOC | Derive Solution |
|-------|-----------|-----------------|-----------------|
| `Default` | 50+ impls | ~1 LOC each | `#[derive(Default)]` |
| `Clone` | 40+ impls | ~1 LOC each | `#[derive(Clone)]` |
| `Debug` | 30+ impls | ~3 LOC each | `#[derive(Debug)]` |
| `Serialize` | 25+ impls | ~5 LOC each | `#[derive(Serialize)]` |
| `Deserialize` | 25+ impls | ~5 LOC each | `#[derive(Deserialize)]` |

### Macro Opportunities

| Pattern | Current | With Macro | Savings |
|---------|---------|------------|---------|
| `impl From<X> for Error` | 15 LOC | 1 LOC | **14 LOC** |
| `async fn with Error` | 20 LOC | 5 LOC | **15 LOC** |
| `Port trait impl` | 30 LOC | 5 LOC | **25 LOC** |

---

## 2026-03-29 - Decomposition Opportunities

**Project:** [cross-repo]
**Category:** decomposition
**Status:** in_progress
**Priority:** P0

### Crate Extraction Candidates

#### High Priority (Large, Independent)

| Crate | Current LOC | Extracted LOC | Dependencies |
|-------|-------------|---------------|--------------|
| `agileplus-api` | ~2,000 | ~500 | domain, cache |
| `agileplus-graph` | ~3,000 | ~800 | domain |
| `agileplus-events` | ~2,500 | ~600 | domain, cache |
| `agileplus-cache` | ~1,500 | ~400 | domain |
| `agileplus-sync` | ~2,000 | ~500 | domain |

#### Medium Priority (Related Functionality)

| Crate | Current LOC | Extracted LOC | Dependencies |
|-------|-------------|---------------|--------------|
| `agileplus-domain` | ~5,000 | ~1,500 | - |
| `agileplus-p2p` | ~1,800 | ~400 | domain |
| `agileplus-nats` | ~2,200 | ~600 | domain |

### Trait/Port Extraction

| Port | Locations | Extract To |
|------|-----------|-----------|
| `Repository<E>` | 5+ crates | `phenotype-ports::repository` |
| `CachePort` | 3+ crates | `phenotype-ports::cache` |
| `LoggerPort` | 2+ crates | `phenotype-ports::logging` |
| `EventBus` | 3+ crates | `phenotype-ports::event_bus` |

### Module Extraction

| Module | Type | Extract To |
|--------|------|-----------|
| `config/loader.rs` | TOML loader | `libs/config-core` |
| `error.rs` (multiple) | Error types | `libs/error-core` |
| `health.rs` (multiple) | Health checks | `libs/health-core` |
| `metrics.rs` (multiple) | Metrics | `libs/metrics-core` |

---

## 2026-03-29 - Code Quality Improvements

**Project:** [cross-repo]
**Category:** quality
**Status:** in_progress
**Priority:** P1

### Complexity Reduction

#### Cyclomatic Complexity Hotspots

| File | Current | Target | Reduction |
|------|---------|--------|-----------|
| `agileplus-graph/src/store.rs` | 45 | 20 | **55%** |
| `agileplus-domain/src/service.rs` | 38 | 15 | **60%** |
| `phenotype-event-sourcing/src/memory.rs` | 32 | 12 | **62%** |

#### Nesting Depth Reduction

| Pattern | Current | Target | Method |
|---------|---------|--------|--------|
| `if` chains | 8+ levels | 3 max | Early returns |
| `match` nesting | 5+ levels | 2 max | `?` operator |
| `Option` unwraps | 12+ | 0 | `map`, `and_then` |

### Type Safety Improvements

| Pattern | Current | Improved |
|---------|---------|----------|
| `String` for IDs | 50+ | `UserId`, `FeatureId` newtypes |
| Raw `bool` params | 20+ | Builder patterns |
| `HashMap<String, _>` | 15+ | Strongly typed maps |

### Async/Await Patterns

| Pattern | Current | Improved |
|---------|---------|----------|
| Blocking in async | 10+ | `tokio::task::spawn_blocking` |
| Unbounded channels | 5+ | `mpsc::bounded` |
| No cancellation | 8+ | `CancellationToken` |
| Missing timeouts | 12+ | `tokio::time::timeout` |

---

## 2026-03-29 - Test Reduction Opportunities

**Project:** [cross-repo]
**Category:** testing
**Status:** in_progress
**Priority:** P2

### Test Duplication

| Pattern | Count | LOC | Action |
|---------|-------|-----|--------|
| Identical unit tests | 30+ | ~900 | Extract to test utilities |
| Copy-paste integration tests | 15+ | ~450 | Parametrize tests |
| Redundant mock implementations | 10+ | ~300 | Share mock traits |

### Property-Based Testing Opportunities

| Module | Manual Cases | Property Tests | Savings |
|--------|--------------|--------------|---------|
| Config parsing | 50 | 5 | **90%** |
| Serialization | 30 | 3 | **90%** |
| ID generation | 20 | 2 | **90%** |
| Hash computation | 25 | 3 | **88%** |

### Fuzz Testing Candidates

| Module | Target | Coverage |
|--------|--------|----------|
| Config parsing | Malformed TOML | High |
| Event serialization | Corrupt bytes | High |
| Graph queries | Deep recursion | Medium |
| ID generation | Collision | Low |

---

## 2026-03-29 - External Package LOC Impact

**Project:** [cross-repo]
**Category:** loc_reduction
**Status:** in_progress
**Priority:** P1

### Fork/Replace Opportunities

| Internal Pattern | LOC | External Alternative | Savings | Effort |
|------------------|-----|---------------------|---------|--------|
| PTY handling | ~750 | `portable-pty` | **700** | 1 week |
| Health checks | ~140 | `health-check` fork | **120** | 3 days |
| Config loading | ~500 | `figment` | **400** | 1 week |
| Error context | ~100 | `anyhow` patterns | **80** | 2 days |
| Process groups | ~200 | `command-group` | **180** | 3 days |
| CLI progress | ~100 | `indicatif` | **90** | 2 days |

### Consolidate to External

| Internal | External | LOC Saved | Risk |
|----------|----------|-----------|------|
| Custom logging | `tracing` | ~200 | Low |
| Custom JSON | `serde_json` | ~150 | Low |
| Custom UUID | `uuid` | ~50 | Low |
| Custom hash | `blake3` | ~30 | Low |

---

## 2026-03-29 - Bundle Size Optimization

**Project:** [cross-repo]
**Category:** optimization
**Status:** pending
**Priority:** P2

### Rust Binary Size

| Technique | Current | Target | Savings |
|-----------|---------|--------|---------|
| LTO (link-time optimization) | OFF | ON | **20-30%** |
| Codegen units = 1 | 16 | 1 | **10-15%** |
| opt-level = "z" | "2" | "z" | **5-10%** |
| strip = true | false | true | **2-5%** |
| panic = "abort" | "unwind" | "abort" | **5-8%** |

### Dependency Optimization

| Pattern | Current | Target |
|---------|---------|--------|
| Dev dependencies in prod | 5+ | 0 |
| Unused features | 20+ | 0 |
| Duplicate deps | 10+ | 0 |
| Large transitive deps | 5+ | Replace |

---

## 2026-03-29 - Implementation Plan

### Phase 1: Quick Wins (This Week)

1. **Remove nested duplicate crates** (~1,710 LOC)
   - `phenotype-event-sourcing/phenotype-event-sourcing/`
   - `phenotype-contracts/phenotype-contracts/`
   - `phenotype-policy-engine/phenotype-policy-engine/`

2. **Add derive macros** (~300 LOC)
   - Error types → `thiserror`
   - Serialization → `#[derive(Serialize, Deserialize)]`

3. **Delete unused deps** (~50 LOC direct)
   - `lru`, `parking_lot`, dead code

### Phase 2: Medium Effort (This Month)

4. **Extract error core** (~400 LOC)
   - Create `libs/error-core`
   - Migrate 8 error types

5. **Extract config core** (~350 LOC)
   - Migrate `libs/config-core` to edition 2024
   - Replace 4 config loaders

6. **Extract health core** (~80 LOC)
   - Create `libs/health-core`
   - Unify 4 health enums

### Phase 3: Major Refactors (This Quarter)

7. **Extract port traits** (~500 LOC)
   - Consolidate hexagonal ecosystems
   - Create `phenotype-ports` crate

8. **External package adoption** (~1,500 LOC)
   - Fork `health-check` → `agileplus-health`
   - Fork `figment` → `phenotype-config`
   - Wrap `casbin` → `phenotype-policy`

---

## Related

- Duplication: `worklogs/DUPLICATION.md`
- Dependencies: `worklogs/DEPENDENCIES.md`
- Architecture: `worklogs/ARCHITECTURE.md`
- Research: `worklogs/RESEARCH.md`

---

_Last updated: 2026-03-29_
