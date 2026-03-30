# phenotype-router-monitor: Consolidation Roadmap

**Project:** Router & API Monitoring Infrastructure Consolidation
**Status:** Design Phase Complete — Ready for Phase 1 Extraction
**Location:** `/Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-router-monitor`
**Created:** 2026-03-30

---

## Executive Summary

phenotype-router-monitor consolidates routing, metrics, and metering infrastructure across the Phenotype ecosystem into a single, reusable Rust workspace.

### What We're Consolidating

| Component | Source | LOC | Purpose |
|-----------|--------|-----|---------|
| **Router** | thegent-router | ~3,500 | Pareto-efficient task routing with audit trails |
| **Metrics** | thegent-metrics | ~700 | High-performance counters, gauges, histograms |
| **Metering** | AgilePlus + thegent | ~1,500 (new) | API quotas, rate limiting, usage tracking |
| **CLI** | thegent-utils | ~800 (new) | Real-time monitoring and audit log viewing |
| **HTTP API** | New | ~600 (new) | Metrics export and observability endpoints |

**Total Code:** ~7,100 LOC (4,200 extracted + 2,900 new)

### Key Benefits

1. **Eliminate Code Duplication** — Single source of truth for router/metrics
2. **Unified API** — Consistent interfaces across router, metrics, metering domains
3. **Production-Grade** — Audit trails, hysteresis control, percentile metrics
4. **Composable** — No inter-crate dependencies; trait-based contracts
5. **Observable** — Metrics registry, HTTP API, real-time CLI tooling
6. **Reusable** — Publish to crates.io for external use

---

## Directory Structure

```
phenotype-router-monitor/
├── Cargo.toml                          # Workspace root (6 member crates)
├── README.md                           # Project overview (180 lines)
├── CONSOLIDATION_ROADMAP.md            # This document
│
├── crates/                             # Implementation crates
│   ├── phenotype-monitor-contracts/    # Shared traits & types (no deps)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs                # MonitorError enum
│   │       ├── router.rs               # Router domain contracts
│   │       ├── metrics.rs              # Metrics domain contracts
│   │       └── meter.rs                # Metering domain contracts
│   │
│   ├── phenotype-router/               # Routing engine (3,500 LOC extracted)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── router.rs               # Core routing algorithm
│   │   │   ├── audit.rs                # SHA-256 hash chains
│   │   │   ├── hysteresis.rs           # Hysteresis state machine
│   │   │   ├── executor.rs             # Task execution interface
│   │   │   ├── risk.rs                 # Risk assessment
│   │   │   ├── orchestrator.rs         # Multi-router coordination
│   │   │   └── python.rs               # PyO3 FFI (optional feature)
│   │   ├── tests/                      # 50+ tests from source
│   │   └── benches/                    # Routing benchmarks
│   │
│   ├── phenotype-metrics/              # Metrics collection (700 LOC + 400 new)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── counter.rs              # Arc<Mutex<u64>> counter
│   │   │   ├── gauge.rs                # Arc<Mutex<f64>> gauge
│   │   │   ├── histogram.rs            # Percentile-aware histogram
│   │   │   ├── registry.rs             # DashMap-backed registry (NEW)
│   │   │   ├── snapshot.rs             # JSON export (NEW)
│   │   │   └── percentiles.rs          # Utility functions (NEW)
│   │   ├── tests/                      # 30+ tests
│   │   └── benches/                    # Metrics benchmarks
│   │
│   ├── phenotype-meter/                # API metering (1,500 LOC new)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── meter.rs                # UsageMeter implementation
│   │   │   ├── quota.rs                # Quota tracking & enforcement
│   │   │   ├── rate_limit.rs           # Token bucket + sliding window
│   │   │   ├── analytics.rs            # Usage aggregation
│   │   │   ├── cost.rs                 # Cost model abstractions
│   │   │   └── classification.rs       # Request classification
│   │   ├── tests/                      # 20+ tests
│   │   └── benches/                    # Performance tests
│   │
│   ├── phenotype-monitor-cli/          # Real-time monitoring CLI (800 LOC new)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs                 # CLI entry point
│   │   │   ├── display.rs              # Metrics visualization
│   │   │   ├── audit_viewer.rs         # Audit log viewer
│   │   │   └── streaming.rs            # Event streaming
│   │   └── tests/
│   │
│   └── phenotype-monitor-api/          # HTTP API server (600 LOC new)
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs                 # Server entry point
│       │   ├── handlers.rs             # Route handlers
│       │   ├── routes.rs               # /metrics, /audit, /usage
│       │   └── models.rs               # Response types
│       └── tests/                      # Integration tests
│
├── docs/                               # Documentation
│   ├── ARCHITECTURE.md                 # Domain models & design (390 lines)
│   ├── CONSOLIDATED_PROJECTS.md        # Source mappings (296 lines)
│   ├── MIGRATION.md                    # Extraction strategy (330 lines)
│   └── API_METERING.md                 # Usage metering spec (PLANNED)
│
├── .agileplus/                         # Work tracking
│   └── SPEC_TRACKER.md                 # WP breakdown and status
│
└── tests/                              # Integration tests (PLANNED)
    └── integration/                    # Cross-crate integration tests
```

---

## Project Status

### Complete (Design Phase)

✅ **WP-1: Project Setup**
- Directory structure created
- Workspace Cargo.toml with all dependencies
- All 6 crates with structure
- Architecture documented (390 lines)
- Consolidated projects mapped (296 lines)
- Migration strategy written (330 lines)
- Specification tracker created

### Pending (Implementation Phase)

⏳ **WP-2: Router Extraction (Week 1-2)**
- Extract thegent-router via git subtree split
- Merge into crates/phenotype-router/
- Verify all tests pass
- Create PR

⏳ **WP-3: Metrics Extraction (Week 2-3)**
- Extract thegent-metrics via git subtree split
- Add new modules (registry, snapshot, percentiles)
- Create PR

⏳ **WP-4: Metering Layer (Week 3-4)**
- Create phenotype-meter crate
- Extract patterns from AgilePlus, thegent
- Full integration tests
- Create PR

⏳ **WP-5: CLI Tooling (Week 4-5)**
- Real-time metrics display
- Audit log viewer
- Event streaming
- Create PR

⏳ **WP-6: HTTP API (Week 4-5)**
- /metrics endpoint
- /audit endpoint
- /usage endpoint
- Integration tests

⏳ **WP-7: Release & Integration (Week 5-6)**
- v0.1.0 release tag
- Update source projects (thegent, AgilePlus)
- Remove duplicate code

---

## Consolidation Strategy

### Phase 1: Extract Core (Weeks 1-3)

**Goal:** Move 4,200 LOC from thegent into reusable crates

```bash
# Week 1-2: Router
git subtree split -P platforms/thegent/crates/thegent-router -b extract-router
# → phenotype-router crate with 50+ tests

# Week 2-3: Metrics
git subtree split -P platforms/thegent/crates/thegent-metrics -b extract-metrics
# → phenotype-metrics crate with registry, snapshot, percentiles

# Success Criteria:
# - All 50+ router tests pass
# - All 30+ metrics tests pass
# - Benchmarks show <1% regression
# - Clippy clean, fmt check passes
```

### Phase 2: Build New Layer (Weeks 3-4)

**Goal:** Create metering abstraction consolidating 1,500 LOC of new code

```rust
// Define UsageMeter trait
pub trait UsageMeter {
    fn record_request(&self, req: RequestMetadata) -> Result<()>;
    fn check_quota(&self, user_id: &str, endpoint: &str) -> Result<QuotaStatus>;
}

// Implement quota, rate limiting, cost tracking
// Integration with phenotype-metrics for observability
```

### Phase 3: Add Tooling (Weeks 4-5)

**Goal:** Create CLI and HTTP API for observability

```bash
# CLI: Real-time metrics, audit log viewer
phenotype-monitor --refresh-interval 5

# HTTP API: Metrics export and usage reports
GET /metrics → Prometheus-format metrics
GET /audit   → Audit trail JSON
GET /usage   → Usage by user/endpoint
```

### Phase 4: Integrate & Deprecate (Week 6)

**Goal:** Update source projects, remove duplicates

```bash
# In thegent/Cargo.toml
phenotype-router = { version = "0.1", path = "../../phenotype-router-monitor/..." }

# Remove source duplicates
rm -rf platforms/thegent/crates/thegent-router
rm -rf platforms/thegent/crates/thegent-metrics
```

---

## Git History Preservation

### Strategy: git subtree split (Recommended)

For each source crate, extract commits while preserving history:

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit

# Extract router commits to new branch
git subtree split -P platforms/thegent/crates/thegent-router -b extract-router

# Expected: 30-40 commits with full history
git log extract-router --oneline | wc -l  # → 35+

# Merge into new repo
cd ../phenotype-router-monitor
git remote add phenotype-infrakit ../phenotype-infrakit
git merge -X subtree=crates/phenotype-router phenotype-infrakit/extract-router \
    --allow-unrelated-histories -m "feat: extract thegent-router with git history"
```

### Result

- ✅ All commits preserved
- ✅ Original authorship intact
- ✅ Commit messages unchanged
- ✅ References to original files maintained (in commit messages)
- ✅ Easy to trace changes back to source

---

## Dependency Management

### Workspace Strategy

All crates use **workspace-level dependencies** to ensure consistency:

```toml
# Cargo.toml (root)
[workspace.dependencies]
serde = "1.0"
tokio = "1.41"
dashmap = "6"
```

```toml
# Cargo.toml (each crate)
[dependencies]
serde = { workspace = true }
tokio = { workspace = true }
```

### Benefits

- Single version for entire workspace
- Easy to upgrade all at once
- No version conflicts
- Faster builds (fewer duplicate compilations)

### Dependency Graph

```
phenotype-monitor-contracts          (no deps)
    ↓
phenotype-router                     (contracts)
phenotype-metrics                    (contracts)
    ↓
phenotype-meter                      (contracts + metrics)
    ↓
phenotype-monitor-cli                (router + metrics + meter)
phenotype-monitor-api                (router + metrics + meter)
```

**Key:** No circular dependencies, flat import graph.

---

## Testing Strategy

### Unit Tests

Each crate maintains inline tests:

```bash
cargo test -p phenotype-router       # 50+ tests
cargo test -p phenotype-metrics      # 30+ tests
cargo test -p phenotype-meter        # 20+ tests
```

### Integration Tests

Cross-crate validation:

```bash
cargo test --test integration_router_metrics
# Validates: routing decisions emit metrics correctly
```

### Benchmarks

Performance validation:

```bash
cargo bench -p phenotype-metrics     # Counter, Gauge, Histogram throughput
cargo bench -p phenotype-router      # Router decision latency
```

**Expected Results:**
- Counter throughput: ≥1M ops/sec
- Router latency: <100µs p99
- Regression: <1% vs. source

---

## Success Criteria

### Code Quality

- [x] Directory structure created
- [ ] All tests pass: `cargo test --workspace`
- [ ] All tests pass: `cargo test --all-features` (with pyo3)
- [ ] Clippy clean: `cargo clippy -- -D warnings`
- [ ] Fmt check: `cargo fmt --check`

### Performance

- [ ] Counter throughput: ≥1M ops/sec
- [ ] Histogram record: <50µs p99
- [ ] Router.route(): <100µs p99
- [ ] Regression: <1% vs. original

### Documentation

- [x] README.md (project overview)
- [x] ARCHITECTURE.md (domain models)
- [x] CONSOLIDATED_PROJECTS.md (source mapping)
- [x] MIGRATION.md (extraction strategy)
- [ ] API_METERING.md (metering spec)
- [ ] Rustdoc (1000+ lines of code comments)

### Git History

- [ ] Original commits preserved (router 30+, metrics 20+)
- [ ] Authorship intact
- [ ] Commit messages unchanged
- [ ] No shallow clones

### Integration

- [ ] thegent imports phenotype-router
- [ ] AgilePlus imports phenotype-meter
- [ ] Duplicate code removed from sources
- [ ] All dependent tests pass

---

## Timeline & Effort

| Phase | Task | Effort | Duration | Dates |
|-------|------|--------|----------|-------|
| Design | Setup + docs | 12h | 1 day | Mar 30 |
| 1 | Extract router | 8h | 2 days | Apr 1-2 |
| 1 | Extract metrics | 6h | 1.5 days | Apr 3-4 |
| 2 | Build meter | 12h | 2.5 days | Apr 5-7 |
| 2 | Create CLI | 8h | 1.5 days | Apr 8-9 |
| 3 | Create API | 8h | 1.5 days | Apr 10-11 |
| 3 | Integrate & deprecate | 6h | 1 day | Apr 12 |
| **Total** | | **60h** | **4 weeks** | **Mar 30-Apr 26** |

---

## Risk Register & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| git subtree split fails | Low | High | Use git filter-branch; plan B in MIGRATION.md |
| Tests fail post-extraction | Low | High | Run full suite before merge; manual verification |
| Performance regression | Very Low | Medium | Compare benchmarks pre/post; rollback if >1% |
| Dependency conflicts | Low | Medium | Use workspace pin strategy; test all-features |
| pyo3 feature issues | Low | Low | Mark optional; test separately on Linux only |

**Contingency:** Can always pause extraction and continue using original code in thegent.

---

## Files Created (Today)

### Configuration
- ✅ `/Cargo.toml` — Workspace root with 6 member crates
- ✅ `/crates/*/Cargo.toml` — 6 individual crate manifests

### Documentation
- ✅ `/README.md` — Project overview (180 lines, already existed)
- ✅ `/docs/ARCHITECTURE.md` — Domain models (390 lines, already existed)
- ✅ `/docs/CONSOLIDATED_PROJECTS.md` — Source mapping (296 lines, already existed)
- ✅ `/docs/MIGRATION.md` — Extraction strategy (330 lines, newly created)
- ✅ `/CONSOLIDATION_ROADMAP.md` — This document

### Specification
- ✅ `/.agileplus/SPEC_TRACKER.md` — Work packages and status (7 WPs)

### Crate Stubs (Structure Only)
- ✅ `phenotype-monitor-contracts/src/lib.rs` — Shared trait exports
- ✅ `phenotype-monitor-contracts/src/{error,router,metrics,meter}.rs` — Domain contracts
- ✅ `phenotype-router/src/lib.rs` — Module exports
- ✅ `phenotype-router/src/{router,audit,hysteresis,executor,risk,orchestrator}.rs` — Stubs
- ✅ `phenotype-metrics/src/lib.rs` — Module exports
- ✅ `phenotype-metrics/src/{counter,gauge,histogram,registry,snapshot,percentiles}.rs` — Stubs
- ✅ `phenotype-meter/src/lib.rs` — Module exports
- ✅ `phenotype-meter/src/{meter,quota,rate_limit,analytics,cost,classification}.rs` — Stubs
- ✅ `phenotype-monitor-cli/src/main.rs` — Placeholder binary
- ✅ `phenotype-monitor-api/src/main.rs` — Placeholder binary

**Total new files:** 22 (config + docs + code stubs)

---

## Next Steps (For Implementation)

### Immediate (Week 1)

1. Review ARCHITECTURE.md for domain model clarity
2. Review MIGRATION.md for extraction procedure
3. Prepare git environment:
   ```bash
   cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit
   git status  # Ensure clean state
   ```

### Week 1-2 (WP-2: Router Extraction)

1. Extract router history:
   ```bash
   git subtree split -P platforms/thegent/crates/thegent-router -b extract-router
   git log extract-router --oneline | head -20  # Verify
   ```

2. Merge into phenotype-router-monitor:
   ```bash
   cd ../phenotype-router-monitor
   git merge -X subtree=crates/phenotype-router ... --allow-unrelated-histories
   ```

3. Verify tests:
   ```bash
   cargo test -p phenotype-router
   cargo clippy -p phenotype-router -- -D warnings
   ```

4. Create PR: "feat: extract phenotype-router from thegent"

### Week 2-3 (WP-3: Metrics Extraction)

Similar process for thegent-metrics, with new modules (registry, snapshot, percentiles).

### Week 3-4 (WP-4: Metering Layer)

Build new phenotype-meter crate consolidating patterns from AgilePlus and thegent.

### Week 4-5 (WP-5 & WP-6: CLI & API)

Create tooling crates for observability.

### Week 6 (WP-7: Integration)

Update source projects, tag v0.1.0, announce consolidation.

---

## References & Resources

### In This Repository

- **README.md** — Quick start and overview
- **docs/ARCHITECTURE.md** — Detailed domain models and design
- **docs/CONSOLIDATED_PROJECTS.md** — Exact source locations and extraction mapping
- **docs/MIGRATION.md** — Step-by-step extraction procedure with git examples
- **.agileplus/SPEC_TRACKER.md** — Work packages, dependencies, risk register

### External

- [git subtree documentation](https://git-scm.com/docs/git-subtree)
- [Cargo workspace documentation](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Hexagonal Architecture](https://en.wikipedia.org/wiki/Hexagonal_architecture)

### Related Projects

- **phenotype-infrakit** (source) — `/Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit`
- **thegent** (router source) — `phenotype-infrakit/platforms/thegent/crates/thegent-router`
- **thegent-metrics** (metrics source) — `phenotype-infrakit/platforms/thegent/crates/thegent-metrics`
- **AgilePlus** (metering patterns) — `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus`
- **heliosCLI** (consumer) — `/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI`

---

## Contact & Questions

For questions about extraction strategy or implementation details, refer to:
- **MIGRATION.md** for git procedures
- **ARCHITECTURE.md** for design rationale
- **SPEC_TRACKER.md** for work breakdown

---

**Created:** 2026-03-30
**Status:** Design Complete — Ready for Phase 1 Extraction
**Effort Remaining:** 48 hours over 4 weeks
**Owner:** TBD (Implementation team)
