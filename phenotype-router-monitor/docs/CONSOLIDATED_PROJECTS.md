# Consolidated Projects

This document catalogs all projects being consolidated into phenotype-router-monitor with exact source locations and extraction strategy.

## Primary Sources

### 1. thegent-router

**Source:** `/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/crates/thegent-router/`

**Repository:** https://github.com/KooshaPari/phenotype-infrakit (under platforms/thegent)

**Current Version:** 0.1.0 (part of thegent monorepo)

#### Files to Extract

| File | LOC | Purpose | Reuse As |
|------|-----|---------|----------|
| `src/lib.rs` | ~150 | Module exports | `phenotype-router/src/lib.rs` |
| `src/router.rs` | ~400 | Core routing logic | `phenotype-router/src/router.rs` |
| `src/audit.rs` | ~250 | SHA-256 hash chains | `phenotype-router/src/audit.rs` |
| `src/hysteresis.rs` | ~300 | Hysteresis state machine | `phenotype-router/src/hysteresis.rs` |
| `src/executor.rs` | ~200 | Task execution interface | `phenotype-router/src/executor.rs` |
| `src/risk.rs` | ~180 | Risk assessment | `phenotype-router/src/risk.rs` |
| `src/orchestrator.rs` | ~220 | Multi-router coordination | `phenotype-router/src/orchestrator.rs` |
| `src/python.rs` | ~400 | PyO3 FFI bindings | `phenotype-router/src/python.rs` (optional feature) |
| `tests/*.rs` | ~1200 | Unit + integration tests | Migrate as-is to `tests/` |
| `benches/audit_bench.rs` | ~200 | Benchmark suite | `benches/audit_bench.rs` |

**Total LOC:** ~3,500

**Dependencies:**
```toml
serde = "1.0"
serde_json = "1.0"
thiserror = "2.0"
sha2 = "0.10"
hex = "0.4"
uuid = { version = "1", features = ["v4"] }
pyo3 = { version = "0.24", optional = true }
tokio = { version = "1", features = ["full"] }
```

#### Key Types to Preserve
- `Router` trait — main routing interface
- `Executor` trait — task execution interface
- `AuditChain` — hash-linked audit log
- `AuditEntry` — individual audit entry
- `HysteresisState` — decision state machine
- `RoutingContext` — decision factors
- `Target` — routing target
- `Task` — unit of work

#### Breaking Changes
- None expected; maintain API surface
- Remove thegent-specific dependencies (thegent-shm → Optional feature)

#### Testing Coverage
- 20+ unit tests for routing algorithm
- 10+ tests for hysteresis logic
- 8+ tests for audit chain integrity
- 12+ Python FFI tests
- 3+ phase 3 integration tests

---

### 2. thegent-metrics

**Source:** `/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/crates/thegent-metrics/`

**Repository:** https://github.com/KooshaPari/phenotype-infrakit (under platforms/thegent)

**Current Version:** 0.1.0 (part of thegent monorepo)

#### Files to Extract

| File | LOC | Purpose | Reuse As |
|------|-----|---------|----------|
| `src/lib.rs` | ~400 | Counter, Gauge, Histogram | `phenotype-metrics/src/lib.rs` |
| (NEW) | — | MetricsRegistry | `phenotype-metrics/src/registry.rs` |
| (NEW) | — | Metrics snapshot export | `phenotype-metrics/src/snapshot.rs` |
| (NEW) | — | Percentile utilities | `phenotype-metrics/src/percentiles.rs` |
| Tests | ~300 | Unit tests | Migrate to `tests/` |

**Total LOC:** ~700 (existing) + ~400 (new)

**Dependencies:**
```toml
serde = "1.0"
serde_json = "1.0"
dashmap = "6"
```

#### Key Types to Preserve
- `Counter` — concurrent counter
- `Gauge` — concurrent gauge
- `Histogram` — percentile-aware histogram
- `MetricsRegistry` — central storage (NEW)

#### Breaking Changes
- None expected; simple extension-only changes

#### New Modules
1. **registry.rs** — DashMap-backed central metric storage
   - `MetricsRegistry::counter(name)`
   - `MetricsRegistry::gauge(name)`
   - `MetricsRegistry::histogram(name, buckets)`
   - `MetricsRegistry::export()` → `MetricsSnapshot`

2. **snapshot.rs** — Serializable metrics export
   - `MetricsSnapshot` struct (serde Serialize)
   - Implement From<MetricsRegistry> for MetricsSnapshot

3. **percentiles.rs** — Utility functions
   - `percentile(values: &[u64], p: usize) -> f64`
   - `percentile_batch(values: &[u64], percentiles: &[usize]) -> Vec<f64>`

#### Testing Coverage
- 5+ unit tests per type (Counter, Gauge, Histogram)
- 10+ integration tests for registry
- 5+ export format tests

---

### 3. API Metering Tools (Scattered)

**Primary Sources:**

#### agileplus-telemetry
**Location:** `/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/AgilePlus/phenotype-docs/crates/agileplus-telemetry/`

- `src/adapter.rs` — Request/response tracking
- `src/lib.rs` — Telemetry trait
- Uses: `metric` counters, `user_id` tracking, `endpoint` classification

**Extract:** Request classification patterns → `phenotype-meter/src/classification.rs`

#### thegent-utils/bin/monitor.rs
**Location:** `/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/crates/thegent-utils/src/bin/monitor.rs`

- Real-time metric display
- Event streaming

**Extract:** CLI foundation → `phenotype-monitor-cli/src/display.rs`

#### Rate Limit Implementations (scattered)
- **AgilePlus routes.rs:** Rate limiting middleware
- **thegent API handlers:** Token bucket logic

**Extract:** Consolidate to `phenotype-meter/src/rate_limit.rs`

#### Cost Tracking
- **AgilePlus telemetry:** Token-based cost tracking
- **thegent resources:** CPU/memory quota tracking

**Extract:** Abstract cost model → `phenotype-meter/src/cost.rs`

#### Usage Analytics
- **agileplus-cli:** User action tracking
- **thegent telemetry:** Pipeline execution metrics

**Extract:** Analytics aggregation → `phenotype-meter/src/analytics.rs`

---

## Extraction Strategy

### Step 1: Initialize Workspace (Week 1)
1. Create directory structure
2. Copy Cargo.toml workspace manifest
3. Create crate stubs with placeholder src/lib.rs

### Step 2: Extract Router (Week 1-2)
1. Use `git subtree split` to preserve history from thegent-router
2. Copy all source files preserving paths
3. Update Cargo.toml to use workspace deps
4. Run tests — expect parity with source
5. Commit with git history preserved

### Step 3: Extract Metrics (Week 2)
1. Use `git subtree split` for thegent-metrics
2. Add new modules (registry, snapshot, percentiles)
3. Add integration tests with router
4. Run benchmarks — expect same throughput

### Step 4: Build Metering Layer (Week 2-3)
1. Create new crate `phenotype-meter`
2. Extract request classification patterns
3. Implement quota tracking
4. Implement rate limiting
5. Integration tests with router + metrics

### Step 5: CLI & API (Week 3-4)
1. Create `phenotype-monitor-cli` — CLI for real-time monitoring
2. Create `phenotype-monitor-api` — HTTP API for metric export
3. Full integration tests across all tiers

### Step 6: Documentation & Release (Week 4)
1. Complete architecture docs
2. Create migration guide for existing projects
3. Tag v0.1.0 release
4. Prepare v0.2.0 roadmap

---

## Git History Preservation

### Recommended Approach: git subtree split

For each source project:

```bash
# Extract thegent-router preserving history
cd /Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent
git subtree split -P crates/thegent-router -b extract-router

# Copy to new repo
cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-router-monitor
git remote add thegent-source /path/to/thegent
git merge -X subtree=crates/phenotype-router thegent-source/extract-router --allow-unrelated-histories

# Repeat for thegent-metrics, etc.
```

### Alternative: git filter-branch

If subtree split doesn't preserve structure as needed:

```bash
# Filter commits to only include thegent-router/
git filter-branch --subdirectory-filter crates/thegent-router -- --all
```

---

## Dependency Alignment

All extracted crates will use **workspace-level dependencies** to ensure consistent versions:

| Dependency | Version | Source |
|-----------|---------|--------|
| serde | 1.0+ | workspace |
| serde_json | 1.0+ | workspace |
| thiserror | 2.0+ | workspace |
| sha2 | 0.10+ | workspace |
| uuid | 1+ | workspace |
| tokio | 1.41+ | workspace |
| dashmap | 6+ | workspace |
| pyo3 | 0.24+ | workspace (optional) |

**No version pinning in individual crates** — all use `workspace = true`.

---

## Cross-Reference Map

### Code Locations After Consolidation

| Original Location | New Location | Crate |
|-------------------|--------------|-------|
| platforms/thegent/crates/thegent-router/src/router.rs | crates/phenotype-router/src/router.rs | phenotype-router |
| platforms/thegent/crates/thegent-router/src/audit.rs | crates/phenotype-router/src/audit.rs | phenotype-router |
| platforms/thegent/crates/thegent-metrics/src/lib.rs | crates/phenotype-metrics/src/lib.rs | phenotype-metrics |
| repos/.../agileplus-telemetry/src/adapter.rs | crates/phenotype-meter/src/classification.rs | phenotype-meter |
| platforms/thegent/.../monitor.rs | crates/phenotype-monitor-cli/src/display.rs | phenotype-monitor-cli |

---

## Integration Points (Post-Extraction)

### For AgilePlus
- Import `phenotype-meter` for API metering
- Import `phenotype-metrics` for observability
- Update telemetry adapters to use new abstractions

### For heliosCLI
- Import `phenotype-router` for task routing
- Import `phenotype-metrics` for CLI telemetry

### For thegent
- Become primary consumer
- Update import paths from `thegent-router` → `phenotype-router`
- Maintain FFI bindings via feature flags

---

## Success Criteria

1. **Code Duplication:** Eliminated (100% of thegent-router/metrics in new crate)
2. **Test Coverage:** Maintained (all existing tests pass)
3. **Dependencies:** Aligned (all workspace versions)
4. **History:** Preserved (git log shows original commits)
5. **Performance:** Regression < 1% (benchmarks match original)
6. **Documentation:** Complete (all modules documented)
7. **Integration:** Functional (all three tiers work together)
