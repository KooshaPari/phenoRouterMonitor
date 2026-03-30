# phenotype-router-monitor Specification Tracker

Project: Router & API Monitoring Consolidation
Status: Design Phase (Ready for Phase 1 Extraction)
Last Updated: 2026-03-30

## Executive Summary

phenotype-router-monitor is a consolidated Rust workspace consolidating three infrastructure domains:
1. **Router** — Pareto-efficient task routing with audit trails (~3,500 LOC, from thegent-router)
2. **Metrics** — High-performance metrics collection (~700 LOC, from thegent-metrics + new modules)
3. **Metering** — API quotas, rate limiting, usage tracking (~1,500 LOC, new consolidation)

**Goal:** Single source of truth for router/monitoring infrastructure across Phenotype ecosystem.

## Deliverables

### Phase 1: Extract Router & Metrics (4 weeks)

| Deliverable | Type | Status | Owner | Due |
|-------------|------|--------|-------|-----|
| phenotype-router crate | Code | Planned | — | Week 2 |
| phenotype-metrics crate | Code | Planned | — | Week 3 |
| phenotype-meter crate | Code | Planned | — | Week 4 |
| Integration tests (router+metrics+meter) | Tests | Planned | — | Week 4 |
| CHANGELOG.md | Docs | Planned | — | Week 4 |
| v0.1.0 release tag | Release | Planned | — | Week 4 |

### Phase 2: CLI & API Tooling (1 week)

| Deliverable | Type | Status | Owner | Due |
|-------------|------|--------|-------|-----|
| phenotype-monitor-cli | Code | Planned | — | Week 5 |
| phenotype-monitor-api | Code | Planned | — | Week 5 |
| HTTP API tests | Tests | Planned | — | Week 5 |
| Observability docs | Docs | Planned | — | Week 5 |

### Phase 3: Integration & Deprecation (1 week)

| Deliverable | Type | Status | Owner | Due |
|-------------|------|--------|-------|-----|
| Update thegent imports | Code | Planned | — | Week 6 |
| Update AgilePlus imports | Code | Planned | — | Week 6 |
| Deprecation notices | Docs | Planned | — | Week 6 |
| Remove duplicate code from sources | Code | Planned | — | Week 6 |

## Work Packages

### WP-1: Project Setup

- [x] Create directory structure at `/Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-router-monitor`
- [x] Create workspace Cargo.toml
- [x] Create crate structure (contracts, router, metrics, meter, cli, api)
- [x] Write README.md with consolidation purpose
- [x] Write ARCHITECTURE.md with domain models
- [x] Write CONSOLIDATED_PROJECTS.md with source mappings
- [x] Write MIGRATION.md with extraction strategy

**Status:** COMPLETE

### WP-2: Router Extraction

- [ ] Extract thegent-router history via git subtree split
- [ ] Merge into crates/phenotype-router/
- [ ] Verify all 50+ router tests pass
- [ ] Update Cargo.toml dependencies
- [ ] Create PR: "feat: extract phenotype-router from thegent"
- [ ] Verify benchmarks <1% regression

**Status:** PENDING

**Effort:** 8 hours
**Owner:** TBD

### WP-3: Metrics Extraction

- [ ] Extract thegent-metrics history via git subtree split
- [ ] Merge into crates/phenotype-metrics/
- [ ] Add registry.rs (MetricsRegistry)
- [ ] Add snapshot.rs (JSON export)
- [ ] Add percentiles.rs (utility functions)
- [ ] Verify all tests pass (existing + new)
- [ ] Create PR: "feat: extract phenotype-metrics from thegent + add registry"

**Status:** PENDING

**Effort:** 6 hours
**Owner:** TBD

### WP-4: Metering Layer

- [ ] Create phenotype-meter crate
- [ ] Extract request classification from agileplus-telemetry
- [ ] Implement UsageMeter trait with quota/rate-limit
- [ ] Implement quota.rs module
- [ ] Implement rate_limit.rs module
- [ ] Implement analytics.rs module
- [ ] Implement cost.rs (cost models)
- [ ] Create integration tests (meter + metrics + router)
- [ ] Create PR: "feat: add phenotype-meter for API metering"

**Status:** PENDING

**Effort:** 12 hours
**Owner:** TBD

### WP-5: CLI Tooling

- [ ] Create phenotype-monitor-cli crate
- [ ] Implement real-time metrics display
- [ ] Implement audit log viewer
- [ ] Implement event streaming
- [ ] Create PR: "feat: add phenotype-monitor-cli"

**Status:** PENDING

**Effort:** 8 hours
**Owner:** TBD

### WP-6: HTTP API

- [ ] Create phenotype-monitor-api crate
- [ ] Implement /metrics endpoint (JSON format)
- [ ] Implement /audit endpoint (audit trail)
- [ ] Implement /usage endpoint (usage reports)
- [ ] Create integration tests
- [ ] Create PR: "feat: add phenotype-monitor-api"

**Status:** PENDING

**Effort:** 8 hours
**Owner:** TBD

### WP-7: Release & Integration

- [ ] Create CHANGELOG.md with extraction notes
- [ ] Tag v0.1.0 release
- [ ] Update thegent imports to use phenotype-router
- [ ] Update AgilePlus imports to use phenotype-meter
- [ ] Remove duplicate code from source projects
- [ ] Create integration PR

**Status:** PENDING

**Effort:** 6 hours
**Owner:** TBD

## Specification Coverage

### Architecture Principles

| Principle | Status | Coverage |
|-----------|--------|----------|
| **Hexagonal Architecture** | Documented | 100% (ARCHITECTURE.md §Ports) |
| **SOLID Principles** | Documented | 100% (ARCHITECTURE.md §Design Principles) |
| **No Inter-Crate Deps** | Planned | 100% (only contracts as dependency) |
| **Production-Grade** | Planned | 100% (audit chains, hysteresis, percentiles) |
| **Composable** | Planned | 100% (trait-based, feature flags) |

### Code Organization

| Item | Status | Target |
|------|--------|--------|
| Crate Count | Complete | 6 (contracts, router, metrics, meter, cli, api) |
| Module Count | Stub | 15+ (3 per domain + cross-cutting) |
| LOC Extracted | Planned | ~4,200 (from thegent-router, metrics) |
| LOC New | Planned | ~1,500 (meter, cli, api) |
| Test Count | Planned | 80+ |
| Benchmark Count | Planned | 5+ |

### Testing Strategy

| Test Type | Status | Count |
|-----------|--------|-------|
| Unit Tests (Router) | Planned | 20+ |
| Unit Tests (Metrics) | Planned | 15+ |
| Unit Tests (Meter) | Planned | 20+ |
| Integration Tests | Planned | 15+ |
| Benchmarks | Planned | 5+ |
| **Total** | | **75+** |

### Documentation

| Document | Status | Lines |
|----------|--------|-------|
| README.md | Complete | 180 |
| ARCHITECTURE.md | Complete | 390 |
| CONSOLIDATED_PROJECTS.md | Complete | 296 |
| MIGRATION.md | Complete | 330 |
| SPEC_TRACKER.md | Complete | (this doc) |
| API_METERING.md | Planned | 200+ |
| Rustdoc (src/) | Planned | 1000+ |
| **Total** | | **2,400+** |

## Dependency Alignment

### Workspace Dependencies

| Dependency | Version | Source | Status |
|-----------|---------|--------|--------|
| serde | 1.0+ | Latest | Defined |
| serde_json | 1.0+ | Latest | Defined |
| thiserror | 2.0+ | Latest | Defined |
| sha2 | 0.10+ | Latest | Defined |
| uuid | 1+ | Latest | Defined |
| tokio | 1.41+ | Latest | Defined |
| dashmap | 6+ | Latest | Defined |
| axum | 0.8+ | Latest | Defined |
| clap | 4+ | Latest | Defined |

**Status:** All workspace dependencies defined and aligned with latest stable versions.

## Success Metrics

### Code Quality

- [ ] All tests pass: `cargo test --workspace`
- [ ] All tests pass: `cargo test --all-features`
- [ ] Clippy clean: `cargo clippy -- -D warnings`
- [ ] Fmt check: `cargo fmt --check`
- [ ] Doc tests: `cargo test --doc`

### Performance

- [ ] Counter throughput: ≥1M ops/sec
- [ ] Gauge throughput: ≥1M ops/sec
- [ ] Histogram record: <50µs p99
- [ ] Router.route(): <100µs p99
- [ ] Regression: <1% vs. original

### Coverage

- [ ] Test coverage: ≥85% (all crates)
- [ ] All public APIs documented
- [ ] All examples runnable
- [ ] All modules have module-level docs

### Git History

- [ ] Original commits preserved for router
- [ ] Original commits preserved for metrics
- [ ] Authorship intact
- [ ] Commit messages unchanged
- [ ] No shallow clones

## Risk Register

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| git subtree split fails | Low | High | Use git filter-branch alternative |
| Tests fail after extraction | Low | High | Run full test suite before merge |
| Performance regression | Very Low | Medium | Compare benchmarks pre/post |
| Dependency version conflicts | Low | Medium | Use workspace-level pin strategy |
| Feature flag issues (pyo3) | Low | Low | Mark as optional, test separately |

## Blockers & Dependencies

### External Dependencies

- None — self-contained extraction

### Internal Dependencies (Sequential)

1. **WP-2 → WP-3** — Router must extract before metrics integration
2. **WP-2, WP-3 → WP-4** — Must extract both before meter integration
3. **WP-2, WP-3, WP-4 → WP-5, WP-6** — All three before CLI/API
4. **WP-5, WP-6 → WP-7** — All code done before release

## Next Steps

1. **Immediate (Today):** Review architecture and consolidation plan
2. **Week 1:** Begin WP-2 (router extraction) — use git subtree split
3. **Week 2:** Complete WP-2, begin WP-3
4. **Week 3:** Complete WP-3, begin WP-4
5. **Week 4:** Complete WP-4, begin WP-5 & WP-6
6. **Week 5:** Complete WP-5 & WP-6
7. **Week 6:** Complete WP-7 (release & integration)

## References

- README.md — Project overview and getting started
- ARCHITECTURE.md — Domain models and design principles
- CONSOLIDATED_PROJECTS.md — Source mappings and extraction order
- MIGRATION.md — Detailed extraction strategy and testing plan
