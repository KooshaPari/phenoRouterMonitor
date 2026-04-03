# Code Decomposition & LOC Reduction Work Items

**Date:** 2026-03-29
**Total Potential Savings:** ~43-44K LOC
**Total Effort:** ~60-90 tool calls across 3 phases
**Timeline:** 6-8 weeks (Weeks 1-8 from start date)

---

## Phase 1: Quick Wins (Weeks 1-2)

### Effort: 15-20 tool calls | Savings: 15-20K LOC | Risk: Low

---

#### ✅ WI-1.1: Extract Test Fixtures → `agileplus-fixtures` Crate

**Status:** Pending
**Effort:** 4-5 tool calls
**Savings:** 541 LOC (seed.rs)
**Risk:** Low

**Description:**
Move `crates/agileplus-dashboard/src/seed.rs` (541 LOC) into a new shared crate for reuse across test suites.

**Tasks:**
1. [ ] Create `crates/agileplus-fixtures/Cargo.toml` with dependencies
2. [ ] Create `crates/agileplus-fixtures/src/lib.rs` with seed data + builders
3. [ ] Extract `FixtureSeed`, `FixtureBuilder` types
4. [ ] Update `agileplus-dashboard` to depend on fixtures crate
5. [ ] Update other test crates to use fixtures (if applicable)
6. [ ] Write integration tests for fixture API
7. [ ] Verify all tests still pass

**Acceptance Criteria:**
- [ ] Fixtures crate compiles without errors
- [ ] All dashboard tests pass using fixtures crate
- [ ] Original seed.rs deleted from dashboard crate
- [ ] No duplicate seed data across crates

**Blocked By:** None
**Blocks:** WI-1.4 (test consolidation)

---

#### ✅ WI-1.2: Consolidate Error Types → `agileplus-errors` Shared Crate

**Status:** Pending
**Effort:** 6-7 tool calls
**Savings:** 800-1,200 LOC
**Risk:** Low

**Description:**
Create a canonical error crate consolidating all error types from 8+ crates into a single, re-exported location. Reduces boilerplate and ensures consistency.

**Tasks:**
1. [ ] Audit error.rs in all crates (agileplus-*, phenotype-*)
2. [ ] Create `crates/agileplus-errors/Cargo.toml`
3. [ ] Create `crates/agileplus-errors/src/lib.rs` with all error variants
4. [ ] Define error hierarchy: `AgileError` (root) → category-specific errors
5. [ ] Implement `From<...>` conversions for common error types
6. [ ] Replace all local error enums with re-exports
7. [ ] Update Cargo.toml dependencies to use agileplus-errors
8. [ ] Write error classification tests
9. [ ] Verify no behavioral changes in error handling

**Acceptance Criteria:**
- [ ] Single canonical error type used across all crates
- [ ] Error classification tests pass (transient, permanent, rate-limit, etc.)
- [ ] No duplicate error variants across codebase
- [ ] Error messages remain clear and actionable

**Blocked By:** None
**Blocks:** WI-2.3 (config consolidation)

---

#### ✅ WI-1.3: Remove Archived Test Files

**Status:** Pending
**Effort:** 2-3 tool calls
**Savings:** ~40 KB
**Risk:** Low (if documented)

**Description:**
Audit `.archive/tests/` directory and remove obsolete test files. Keep only historically relevant examples.

**Tasks:**
1. [ ] Inventory all files in `.archive/tests/` with LOC count
2. [ ] For each file: determine if keep (for reference) or delete
3. [ ] Document keep/delete decisions in ADR (e.g., ADR-009: Archive Policy)
4. [ ] Move keepers to `docs/archived-tests/` with README
5. [ ] Delete unmarked files
6. [ ] Verify no tests reference deleted files
7. [ ] Update .gitignore if needed

**Acceptance Criteria:**
- [ ] All archive test decisions documented
- [ ] No orphaned references to deleted test files
- [ ] Archive directory only contains intentionally preserved examples
- [ ] Total LOC in archive reduced by ~40 KB

**Blocked By:** None
**Blocks:** None (independent)

---

#### ✅ WI-1.4: De-Duplicate CLI Tests via Git Symlinks

**Status:** Pending
**Effort:** 3-4 tool calls
**Savings:** ~15,000 LOC (test file copies)
**Risk:** Low (automation-friendly)

**Description:**
Replace duplicated test files in worktrees with symlinks to canonical test sources. Eliminates ~15K LOC of test duplication.

**Affected Files:**
- `test_phench_runtime.py` (5 copies → 1 + 4 symlinks)
- `test_unit_cli_coverage_*.py` (8 copies → 4 + 4 symlinks)
- `test_unit_cli_commands_*.py` (12 copies → 6 + 6 symlinks)
- `test_integration_cli_*.py` (8 copies → 4 + 4 symlinks)

**Tasks:**
1. [ ] Identify all duplicated test files across repos + worktrees
2. [ ] For each duplicate set: keep master in canonical location
3. [ ] Create symlinks in worktrees pointing to canonical
4. [ ] Update `.gitignore` if needed (to allow symlinks)
5. [ ] Test: Run full test suite, verify symlinks resolve
6. [ ] CI: Update CI to use single source (no path-based test discovery)
7. [ ] Verify worktree builds still work with symlinks

**Acceptance Criteria:**
- [ ] All duplicated test files replaced with symlinks
- [ ] Full test suite passes with symlinks
- [ ] No test duplication in git history (verify with `git ls-tree`)
- [ ] ~15K LOC "removed" (via symlinks)

**Blocked By:** WI-1.1 (fixtures extraction may affect test structure)
**Blocks:** Phase 2 testing

---

## Phase 2: High-Impact Refactors (Weeks 3-6)

### Effort: 30-40 tool calls | Savings: 25-35K LOC | Risk: Medium

---

#### ✅ WI-2.1: Split routes.rs (2,631 LOC) → Feature Modules

**Status:** Pending
**Effort:** 10-12 tool calls
**Savings:** ~45% complexity reduction
**Risk:** Medium (behavioral verification needed)

**Description:**
Decompose `crates/agileplus-dashboard/src/routes.rs` (2,631 LOC, 1,516 indent levels) into 5 focused feature modules.

**Target Structure:**
```
routes/
├── mod.rs          (~200 LOC: router setup, middleware, re-exports)
├── dashboard.rs    (~600 LOC: feature list, status, metrics, module tree)
├── api.rs          (~500 LOC: feature CRUD, WP ops, cycle management)
├── settings.rs     (~300 LOC: config page, auth, integrations)
├── health.rs       (~200 LOC: health checks, service status)
└── timeline.rs     (~150 LOC: event timeline, git links, CI/CD links)
```

**Tasks:**
1. [ ] Create `routes/` subdirectory structure
2. [ ] Analyze current routes.rs, group 53 handlers by feature
3. [ ] Extract dashboard handlers → `routes/dashboard.rs`
4. [ ] Extract CRUD handlers → `routes/api.rs`
5. [ ] Extract settings handlers → `routes/settings.rs`
6. [ ] Extract health handlers → `routes/health.rs`
7. [ ] Extract timeline handlers → `routes/timeline.rs`
8. [ ] Extract common utilities → `routes/common.rs` or parent module
9. [ ] Update `routes/mod.rs` to re-export and wire handlers
10. [ ] Consolidate template functions from `templates.rs` into respective modules
11. [ ] Add integration tests per route module
12. [ ] Verify middleware propagation (auth, telemetry, etc.)
13. [ ] Verify error handling in new structure
14. [ ] Performance test: Verify no regression in route dispatch latency

**Acceptance Criteria:**
- [ ] No route handler file >600 LOC
- [ ] All 53 handlers properly organized by feature
- [ ] Zero behavioral changes (same API responses)
- [ ] Integration tests pass for each route module
- [ ] Middleware (auth, telemetry, logging) still works correctly
- [ ] Error handling preserved and tested

**Blocked By:** WI-1.2 (error consolidation helpful but not required)
**Blocks:** WI-2.3 (config integration)

---

#### ✅ WI-2.2: Extract SQLite Adapter Logic (1,582 LOC) → Submodules

**Status:** Pending
**Effort:** 12-14 tool calls
**Savings:** ~40% complexity reduction
**Risk:** Medium-High (critical data access layer)

**Description:**
Decompose `crates/agileplus-sqlite/src/lib.rs` (1,582 LOC, 1,015 indent levels) into focused submodules by concern.

**Target Structure:**
```
agileplus-sqlite/src/
├── lib.rs              (~100 LOC: module exports, StoragePort impl dispatch)
├── connection.rs       (~150 LOC: connection pool, lifecycle management)
├── store/
│   ├── mod.rs          (~80 LOC: StoragePort impl, query dispatch)
│   ├── sync.rs         (~400 LOC: sync logic, state reconciliation, delta computation)
│   ├── query.rs        (~300 LOC: SQL query builder, parameterized queries, prepared statements)
│   ├── migrations.rs   (~250 LOC: schema management, version tracking, alter statements)
│   └── error.rs        (~100 LOC: SQLite-specific error types, conversion)
└── cache.rs            (~150 LOC: LRU cache, invalidation strategy)
```

**Tasks:**
1. [ ] Create `store/` subdirectory
2. [ ] Extract sync logic (~400 LOC) → `store/sync.rs`
   - [ ] State reconciliation
   - [ ] Delta computation
   - [ ] Conflict detection
3. [ ] Extract SQL query building (~300 LOC) → `store/query.rs`
   - [ ] Parameterized query construction
   - [ ] Prepared statement caching
   - [ ] Query result mapping
4. [ ] Extract schema migrations (~250 LOC) → `store/migrations.rs`
   - [ ] Schema version tracking
   - [ ] Migration execution
   - [ ] Rollback logic
5. [ ] Extract connection pooling (~150 LOC) → `connection.rs`
   - [ ] Pool initialization
   - [ ] Lifecycle management
   - [ ] Health checks
6. [ ] Update `lib.rs` to dispatch to submodules
7. [ ] Create `store/error.rs` for SQLite-specific errors
8. [ ] Add integration tests for each module (especially sync & migrations)
9. [ ] Add property-based tests for query builder
10. [ ] Performance test: Verify no regression in query latency or cache hit rate
11. [ ] Stress test: Run with concurrent transactions to verify safety

**Acceptance Criteria:**
- [ ] No module >400 LOC
- [ ] All logic properly separated by concern
- [ ] Zero behavioral changes in storage operations
- [ ] Integration tests pass (CRUD, sync, migrations)
- [ ] Property-based tests pass for query builder
- [ ] Performance: query latency within 5% of original
- [ ] Stress test: concurrent transactions still safe

**Blocked By:** WI-1.2 (error consolidation helpful; can use local errors if needed)
**Blocks:** None (can run in parallel with WI-2.1)

---

#### ✅ WI-2.3: Consolidate Config Loading Pattern

**Status:** Pending
**Effort:** 8-10 tool calls
**Savings:** ~300-500 LOC (dedup + abstraction)
**Risk:** Medium (affects all config consumers)

**Description:**
Unify fragmented config loading patterns across `phenotype-config-core` and `agileplus-*` crates into a single builder pattern.

**Current Fragmentation:**
- `phenotype-config-core/src/unified.rs` (423 LOC)
- `phenotype-config-core/src/loader.rs` (358 LOC)
- Scattered builder logic in `agileplus-api`, `agileplus-cli`, etc.

**Target:** Canonical builder at `phenotype-config-core/src/builder.rs`

**Tasks:**
1. [ ] Merge `unified.rs` + `loader.rs` logic into design
2. [ ] Create `ConfigBuilder` trait and implementation
3. [ ] Design builder API: `.with_env()`, `.with_file()`, `.with_defaults()`, `.build()`
4. [ ] Extract config schema definitions to `phenotype-config-core/src/schema.rs`
5. [ ] Implement validation framework using schema
6. [ ] Update `phenotype-config-core/lib.rs` to export builder + schema
7. [ ] Migrate `agileplus-api` config to builder pattern
8. [ ] Migrate `agileplus-cli` config to builder pattern
9. [ ] Migrate other crates using config
10. [ ] Add property-based tests for builder combinations
11. [ ] Verify no behavioral changes (same config output)

**Acceptance Criteria:**
- [ ] Single `ConfigBuilder` pattern used across all crates
- [ ] Config schema validated consistently
- [ ] All config files still load correctly
- [ ] Property-based tests pass for builder combinations
- [ ] No regression in startup time
- [ ] Error messages clear and actionable

**Blocked By:** WI-1.2 (error consolidation; config errors need canonical error type)
**Blocks:** None (can run after Phase 1)

---

## Phase 3: Long-Term Architecture (Weeks 7+)

### Effort: 30-50 tool calls | Ongoing | Risk: Medium-High

---

#### ✅ WI-3.1: Audit thegent's Go Monorepo (5.34M LOC)

**Status:** Pending
**Effort:** 5-7 tool calls
**Savings:** TBD (estimate 10-20% consolidation opportunity)
**Risk:** Low (audit only, no changes)

**Description:**
Analyze 16,745 Go files for package consolidation, circular dependencies, and duplicate implementations.

**Tasks:**
1. [ ] Map package dependency graph: `go mod graph` → visualization
2. [ ] Identify circular dependencies (if any)
3. [ ] Find duplicate package implementations (same logic in 2+ packages)
4. [ ] Estimate consolidation savings per package
5. [ ] Document package architecture in ADR-010 (thegent Go Architecture)
6. [ ] Recommend top 5 decomposition opportunities
7. [ ] Provide implementation roadmap for future phases

**Deliverable:** Audit report (20-30 pages) with:
- Package dependency graph
- Consolidation roadmap
- Estimated LOC savings (10-20% expected)
- Implementation timeline

**Blocked By:** None (independent)
**Blocks:** WI-3.2, WI-3.3

---

#### ✅ WI-3.2: Extract Shared Design System → `@phenotype/design` Package

**Status:** Pending
**Effort:** 10-15 tool calls
**Savings:** ~8,000 LOC (shared components)
**Risk:** Medium (coordination across repos)

**Description:**
Consolidate UI components, design tokens, and themes into a unified `@phenotype/design` monorepo package.

**Current Fragmentation:**
- Dashboard components in `agileplus-dashboard/`
- Byteport components in `platforms/thegent/apps/byteport/`
- Design tokens scattered across projects

**Target:** Unified `packages/@phenotype/design` with:
- Component library (React + Storybook)
- Design tokens (colors, spacing, typography)
- Theme system (light/dark mode)
- Icon library

**Tasks:**
1. [ ] Audit all component implementations across repos
2. [ ] Identify reusable components (buttons, forms, modals, etc.)
3. [ ] Extract design tokens (colors, spacing, typography)
4. [ ] Create `packages/@phenotype/design/` directory structure
5. [ ] Migrate components to design package
6. [ ] Set up Storybook for component documentation
7. [ ] Create theme system (light/dark mode)
8. [ ] Update `agileplus-dashboard` to use design package
9. [ ] Update `byteport` to use design package
10. [ ] Publish to npm or GitHub Packages
11. [ ] Write migration guide for other projects

**Deliverable:**
- `@phenotype/design` package on npm/GitHub Packages
- Storybook documentation
- Component API documentation
- Theme customization guide

**Blocked By:** WI-3.1 (optional; good to complete first for Go audit context)
**Blocks:** None

---

#### ✅ WI-3.3: Version Spec Docs Independently

**Status:** Pending
**Effort:** 8-12 tool calls
**Savings:** ~1-2K LOC (reduced duplication, cleaner structure)
**Risk:** Low-Medium (organizational change)

**Description:**
Create separate `phenotype-specs` monorepo with versioned, independently released specifications.

**Current:** Specs live in repo root (PRD.md, ADR.md, PLAN.md, etc.)

**Target:** Separate monorepo with:
- Version tags per spec (e.g., `thegent@2.0.0-rc1`)
- Changelog tracking per spec
- CI/CD release pipeline
- Cross-repo spec references

**Tasks:**
1. [ ] Create `phenotype-specs` repository
2. [ ] Structure: `specs/thegent/`, `specs/heliosCLI/`, `specs/agileplus/`, etc.
3. [ ] Migrate all specs from canonical repos to phenotype-specs
4. [ ] Set up versioning (SemVer per spec)
5. [ ] Create CHANGELOG.md per spec
6. [ ] Add CI/CD release pipeline (GitHub Actions)
7. [ ] Update canonical repos to reference external specs (git submodule or URL)
8. [ ] Document spec update workflow
9. [ ] Publish to GitHub Packages or doc site

**Deliverable:**
- `phenotype-specs` monorepo with full version history
- Release automation (tagging, changelogs, docs publishing)
- Cross-repo spec reference documentation

**Blocked By:** None (can run in parallel)
**Blocks:** None

---

## Work Item Dependencies

```
Phase 1 (Weeks 1-2):
  WI-1.1 (Fixtures)
    ├─ WI-1.2 (Errors) → independent, complete in parallel
    ├─ WI-1.3 (Archive) → independent
    └─ WI-1.4 (Symlinks) → depends on WI-1.1

Phase 2 (Weeks 3-6):
  WI-2.1 (Routes split)
    ├─ WI-2.2 (SQLite split) → independent, parallel
    └─ WI-2.3 (Config consolidation) → depends on WI-1.2 (errors)

Phase 3 (Weeks 7+):
  WI-3.1 (Go audit)
  WI-3.2 (Design system) → can run parallel with WI-3.1
  WI-3.3 (Specs monorepo) → can run parallel with WI-3.1, WI-3.2
```

---

## Effort & Resource Allocation

| Phase | Total Tool Calls | Agents | Duration | Parallel Capability |
|-------|---|---|---|---|
| **Phase 1** | 15-20 | 1-2 | 2 weeks | 80% parallelizable (T1.1, T1.2, T1.3 in parallel; T1.4 after) |
| **Phase 2** | 30-40 | 2-3 | 4 weeks | 50% parallelizable (T2.1 & T2.2 in parallel; T2.3 after) |
| **Phase 3** | 30-50 | 2-3 | 4+ weeks | 90% parallelizable (all independent) |
| **TOTAL** | ~75-110 | 2-3 sustained | 8-10 weeks | ~60% average parallelization |

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Phase 1 Completion | 15-20K LOC saved | `cloc` diff before/after |
| Routes.rs Split | <600 LOC per module | `wc -l routes/*.rs` |
| SQLite.lib Split | <400 LOC per module | `wc -l store/*.rs` |
| Test Duplication | <1K LOC duplicated | `git ls-tree` + manual count |
| Dead Code Suppressions | 0 undocumented | Grep `#[allow(dead_code)]` |
| All Tests Pass | 100% test suite green | CI run successful |
| No Behavioral Changes | 0 regressions | Integration test results |
| Code Quality | No new clippy warnings | `cargo clippy` clean |

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| **Behavioral changes in routes.rs split** | Write comprehensive integration tests before & after; run property-based tests on router logic |
| **Data loss in SQLite refactor** | Extensive migration tests; backup test database; verify round-trip CRUD operations |
| **Config loading breakage** | Maintain backward-compatibility layer; deprecate old patterns with warnings; extensive end-to-end tests |
| **Worktree symlink issues on Windows** | Test on macOS + Linux first; document Windows workaround if needed |
| **Merge conflicts during refactoring** | Use feature branches, avoid main changes during decomposition; coordinate timing |

---

## Approval & Tracking

- **Owner:** Architecture Team
- **Sponsor:** Engineering Lead
- **Start Date:** 2026-04-01 (estimated)
- **Target Completion:** 2026-05-31 (Phase 1-2 minimum viable, Phase 3 ongoing)

**Status Dashboard:** Update weekly with:
- [ ] Completed tasks
- [ ] In-progress work
- [ ] Blockers
- [ ] LOC savings achieved

---

**Document Status:** Ready for Implementation
**Last Updated:** 2026-03-29
