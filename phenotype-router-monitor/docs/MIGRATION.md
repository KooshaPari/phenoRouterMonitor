# Git History Preservation & Migration Strategy

This document details how to consolidate code from source projects while preserving git history and maintaining clean extraction boundaries.

## Overview

phenotype-router-monitor consolidates three primary domains:
1. **Router** (from thegent-router)
2. **Metrics** (from thegent-metrics)
3. **Metering** (new crate, extracting patterns from AgilePlus, thegent)

Total code to extract: ~4,200 LOC across 30+ files.

## Git History Preservation Strategies

### Strategy 1: git subtree split (RECOMMENDED)

Preserves full commit history for each extracted project as a branch.

#### Step 1: Extract Router History

From the phenotype-infrakit repo (where thegent is a subdirectory):

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit

# Extract thegent-router commits to a new branch
git subtree split -P platforms/thegent/crates/thegent-router -b extract-router

# View extracted history
git log extract-router --oneline | head -20
```

#### Step 2: Merge into phenotype-router-monitor

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-router-monitor

# Add source as remote (if not already)
git remote add phenotype-infrakit /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit

# Fetch the extracted branch
git fetch phenotype-infrakit extract-router

# Merge with subtree strategy to place under crates/phenotype-router/
git merge -X subtree=crates/phenotype-router phenotype-infrakit/extract-router --allow-unrelated-histories -m "feat: extract thegent-router with git history"
```

#### Step 3: Clean Up

```bash
# Delete the temporary branch from source
git branch -d extract-router

# Delete remote reference
git push phenotype-infrakit --delete extract-router
```

### Strategy 2: git filter-branch (Alternative)

If subtree split doesn't preserve directory structure correctly:

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit

# Create filtered history (only thegent-router commits)
git filter-branch --subdirectory-filter platforms/thegent/crates/thegent-router -- --all

# View filtered history
git log --oneline | head -20
```

Then merge into phenotype-router-monitor:

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-router-monitor
git remote add thegent-filtered /tmp/thegent-router-filtered
git merge thegent-filtered/main --allow-unrelated-histories -m "feat: extract thegent-router"
```

## Extraction Order & Phasing

### Phase 1: Router Extraction (Week 1-2)

**Goal:** Extract ~3,500 LOC from thegent-router with full history

**Steps:**
1. Extract router history via git subtree split (Strategy 1 recommended)
2. Merge into `crates/phenotype-router/` directory
3. Update Cargo.toml dependencies (use workspace versions)
4. Verify all tests pass: `cargo test -p phenotype-router`
5. Create PR: "feat: extract phenotype-router from thegent"

**Expected Changes:**
- `/crates/phenotype-router/src/*.rs` — 3,500 LOC
- `/crates/phenotype-router/tests/` — 1,200 LOC
- `/crates/phenotype-router/benches/` — 200 LOC
- 50+ commits with original authorship preserved

**Success Criteria:**
- All tests pass
- `cargo clippy` clean
- No new warnings
- Benchmarks show <1% regression from original

### Phase 2: Metrics Extraction (Week 2-3)

**Goal:** Extract ~700 LOC from thegent-metrics + add new modules (~400 LOC)

**Steps:**
1. Extract metrics history via git subtree split
2. Merge into `crates/phenotype-metrics/` directory
3. Add new modules:
   - `registry.rs` — DashMap-backed MetricsRegistry
   - `snapshot.rs` — MetricsSnapshot serialization
   - `percentiles.rs` — Percentile utilities
4. Integration test with router: metrics emitted during routing
5. Create PR: "feat: extract phenotype-metrics from thegent + add registry"

**Expected Changes:**
- `/crates/phenotype-metrics/src/*.rs` — 700 LOC (extracted) + 400 LOC (new)
- `/crates/phenotype-metrics/tests/` — 300 LOC
- 30+ commits with original authorship preserved
- 10+ new tests for registry/snapshot

**Success Criteria:**
- All tests pass (existing + new)
- Registry API supports all three metric types
- Snapshot exports valid JSON
- Integration test: router metrics observable

### Phase 3: New Metering Layer (Week 3-4)

**Goal:** Create new crate consolidating metering patterns (~1,500 LOC)

**Steps:**
1. Create `crates/phenotype-meter/` crate
2. Extract patterns from:
   - agileplus-telemetry (request classification)
   - thegent API handlers (rate limiting, quota)
   - AgilePlus routes.rs (cost tracking)
3. Implement modules:
   - `meter.rs` — UsageMeter trait implementation
   - `quota.rs` — Quota tracking and enforcement
   - `rate_limit.rs` — Token bucket and sliding window
   - `analytics.rs` — Usage aggregation and reporting
   - `cost.rs` — Cost model abstractions
   - `classification.rs` — Request classification
4. Integration test with router + metrics
5. Create PR: "feat: add phenotype-meter for API metering"

**Expected Changes:**
- `/crates/phenotype-meter/src/*.rs` — 1,500 LOC (new)
- `/crates/phenotype-meter/tests/` — 400 LOC
- 15+ commits (new work)
- 20+ tests covering quota, rate limiting, cost

**Success Criteria:**
- Quota enforcement tested under load
- Rate limiter handles concurrent requests
- Cost calculator correct for multiple models
- Integration: meter visible in metrics registry

### Phase 4: CLI & HTTP API (Week 4-5)

**Goal:** Create tooling crates for observability and operations

#### phenotype-monitor-cli

- Real-time metrics display (TUI or term-based)
- Audit log viewer with git-log-style output
- Event streaming for live updates

**Expected Changes:**
- `/crates/phenotype-monitor-cli/src/*.rs` — 800 LOC
- Depends on: router, metrics, meter
- Clap for CLI parsing
- Integration tests

#### phenotype-monitor-api

- Axum HTTP server
- `/metrics` endpoint (Prometheus format)
- `/audit` endpoint (JSON audit trail)
- `/usage` endpoint (usage reports)

**Expected Changes:**
- `/crates/phenotype-monitor-api/src/*.rs` — 600 LOC
- Depends on: router, metrics, meter, axum
- Integration tests with HTTP client

## Dependency Management During Migration

### Before Extraction

In **phenotype-infrakit** (canonical source):
```toml
[workspace.dependencies]
serde = "1.0"
tokio = "1.41"
# ... etc
```

In **thegent-router** (source crate):
```toml
[dependencies]
serde.workspace = true
tokio.workspace = true
```

### After Extraction to phenotype-router-monitor

In **phenotype-router-monitor** (new workspace):
```toml
[workspace.dependencies]
serde = "1.0"  # Same version
tokio = "1.41" # Same version
```

In **phenotype-router** (extracted crate):
```toml
[dependencies]
serde.workspace = true  # Uses new workspace versions
tokio.workspace = true
```

**Key Point:** All workspace dependencies remain aligned; only the workspace root changes.

## Testing Strategy During Migration

### Unit Tests (In-Crate)

Every extracted crate maintains its original tests:

```bash
# Test individual crate
cargo test -p phenotype-router

# Test all crates in workspace
cargo test --workspace

# Test with all features (including optional pyo3)
cargo test --all-features
```

### Integration Tests (Cross-Crate)

New integration tests validate boundaries:

```bash
# tests/integration_router_metrics.rs
#[test]
fn test_routing_emits_metrics() {
    let router = StandardRouter::new();
    let metrics = MetricsRegistry::new();

    // Route a task
    let task = Task { /* ... */ };
    let _result = router.route(task, context);

    // Verify metrics were recorded
    let snapshot = metrics.export();
    assert!(snapshot.counters.contains_key("router.decisions_total"));
}
```

### Regression Tests

Benchmarks ensure extraction doesn't degrade performance:

```bash
# Run benchmarks before and after extraction
cargo bench -p phenotype-router
cargo bench -p phenotype-metrics
```

**Expected Results:**
- Counter inc(): 1-2 million ops/sec (no change)
- Router.route(): <100µs p99 latency (no change)
- Histogram.record(): <50µs p99 latency (no change)

## Handling Breaking Changes

### Minor API Adjustments

If source code requires updates for phenotype-router-monitor constraints:

1. **Update trait bounds** (e.g., add Clone for serialization)
   ```rust
   // Before: pub struct Task { ... }
   // After:
   #[derive(Clone, Debug, Serialize, Deserialize)]
   pub struct Task { ... }
   ```

2. **Remove thegent-specific dependencies**
   - Replace `thegent-shm` with optional feature
   - Replace thegent logging with standard tracing/log

3. **Document in CHANGELOG**
   ```
   ## v0.1.0 (Extraction)
   - Breaking: Removed thegent-shm dependency (use feature flag for custom IPC)
   - Enhancement: Added MetricsRegistry for centralized metric storage
   - Fix: Updated AuditChain to use SHA-256 strictly (was variable hash)
   ```

### No Incompatibilities Expected

Given the careful crate design in phenotype-infrakit, we expect:
- 0 breaking changes to public API
- 100% test compatibility
- <1% performance regression

## Post-Migration Integration

### Step 1: Update Source Projects

Once extraction is complete, update source projects to import from phenotype-router-monitor:

**In thegent/Cargo.toml:**
```toml
[dependencies]
phenotype-router = { path = "../../phenotype-router-monitor/crates/phenotype-router" }
```

Or from published crate:
```toml
[dependencies]
phenotype-router = "0.1"  # From crates.io
```

**Remove duplicated code:**
```bash
rm -rf /path/to/thegent/crates/thegent-router
rm -rf /path/to/thegent/crates/thegent-metrics
```

### Step 2: Update AgilePlus

Import metering abstractions:

```toml
[dependencies]
phenotype-meter = { path = "../../phenotype-router-monitor/crates/phenotype-meter" }
```

Replace inline telemetry with meter trait.

### Step 3: Update heliosCLI

Import router for task dispatch:

```toml
[dependencies]
phenotype-router = { path = "../../phenotype-router-monitor/crates/phenotype-router" }
```

## Validation Checklist

Before declaring migration complete:

- [ ] All commits in new crates have original authorship preserved
- [ ] git log shows 50+ extracted commits for router, 30+ for metrics
- [ ] All tests pass: `cargo test --workspace`
- [ ] All tests pass: `cargo test --all-features`
- [ ] Clippy clean: `cargo clippy --all-targets -- -D warnings`
- [ ] Benchmarks show <1% regression
- [ ] README.md updated with extraction status
- [ ] CHANGELOG.md documents extraction
- [ ] Integration PR merged to main
- [ ] Source projects updated to import from new location
- [ ] Duplicate code removed from source projects
- [ ] Published to crates.io (optional)

## Timeline & Effort Estimate

| Phase | Task | Effort | Start | Duration |
|-------|------|--------|-------|----------|
| 1 | Extract router | 8h | Day 1 | 2 days |
| 1 | Extract metrics | 6h | Day 3 | 1.5 days |
| 2 | Build meter | 12h | Day 5 | 2.5 days |
| 2 | Create CLI | 8h | Day 8 | 1.5 days |
| 3 | Create API | 8h | Day 10 | 1.5 days |
| 3 | Docs & Release | 6h | Day 12 | 1 day |
| **Total** | | **48h** | | **4 weeks** |

## Rollback Plan

If extraction encounters issues:

1. **In phenotype-router-monitor:**
   ```bash
   git reset --hard HEAD~N  # Before extraction merge
   git branch -D extract-router
   ```

2. **Continue using original locations:**
   - thegent-router stays in phenotype-infrakit
   - thegent-metrics stays in phenotype-infrakit
   - No changes required in dependent projects

3. **Retry extraction** with alternative strategy (git filter-branch)

## References

- [git subtree documentation](https://git-scm.com/docs/git-subtree)
- [Preserving history with git filter-branch](https://git-scm.com/docs/git-filter-branch)
- [Cargo workspace documentation](https://doc.rust-lang.org/cargo/reference/workspaces.html)
