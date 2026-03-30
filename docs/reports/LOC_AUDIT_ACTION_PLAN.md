# LOC Audit - Action Plan

**Based on:** `/docs/reports/LOC_AUDIT_REPORT.md`
**Status:** Ready for implementation
**Last Updated:** 2026-03-29

---

## Phase 1: Immediate Cleanup (2-4 hours)

### Task 1.1: Delete Empty Stub Crates
**Scope:** Remove 3 crates with 0-2 LOC of actual implementation
**Files to Delete:**
- `/crates/phenotype-state-machine/` (0 LOC)
- `/crates/phenotype-git-core/` (1 LOC)
- `/crates/phenotype-cache-adapter/` (1 LOC)

**Steps:**
1. [ ] Update `/Cargo.toml` - remove from workspace
2. [ ] Search for references to `phenotype-state-machine` in:
   - [ ] agileplus-domain (likely uses agileplus-domain models instead)
   - [ ] Any other crate with dependency
3. [ ] Search for references to `phenotype-git-core` in:
   - [ ] agileplus-git
4. [ ] Search for references to `phenotype-cache-adapter` in:
   - [ ] agileplus-cache
   - [ ] agileplus-api
5. [ ] Verify no crates depend on these
6. [ ] Delete directories
7. [ ] Commit: "chore: remove empty stub crates"

**Estimated Time:** 30 minutes
**Risk:** LOW - no code depends on stubs
**Savings:** 2 LOC (cleanup only)

---

### Task 1.2: Remove/Complete Phench Stubs
**Scope:** Incomplete models.py and store.py in `/phench/`
**Files:**
- `/phench/models.py` (66 LOC - imports only)
- `/phench/store.py` (66 LOC - imports only)

**Decision Required:**
- [ ] **Option A:** Complete implementation (estimate: 1-2 days per file)
- [ ] **Option B:** Delete both files (quick cleanup)

**If Option B (Delete):**
1. [ ] Identify what models.py was meant to provide
2. [ ] Identify what store.py was meant to provide
3. [ ] Verify no code imports from these modules
4. [ ] Delete files
5. [ ] Commit: "chore: remove incomplete phench stub modules"

**Estimated Time:** 1-2 hours (if deleting)
**Risk:** LOW - check for imports first
**Savings:** 132 LOC

---

### Task 1.3: Audit and Remove TODO-Marked Test Files (Thegent)
**Scope:** 425 Python files with TODO/FIXME comments
**Approach:** Sampling-based audit

**Steps:**
1. [ ] Generate list of TODO files:
   ```bash
   find /platforms/thegent -name "*.py" ! -path "*/__pycache__/*" -exec grep -l "TODO\|FIXME" {} \; | sort > /tmp/todo_files.txt
   ```
2. [ ] Manually audit first 20 files (highest priority):
   - [ ] Check if TODO is test scenario only
   - [ ] Check if test actually runs (not skipped)
   - [ ] Check if used by CI/CD
3. [ ] For files with only TODO comments:
   - [ ] Mark with `@pytest.mark.skip(reason="TODO: implement")`
4. [ ] For orphaned test files:
   - [ ] Move to `tests/archive/` or delete
5. [ ] Create issue to track completion of TODO tests

**Estimated Time:** 1-2 hours (sampling)
**Risk:** MEDIUM - need to verify test coverage
**Savings:** 100-500 LOC (removing dead tests)

---

## Phase 2: Consolidation (1-2 weeks)

### Task 2.1: Merge phenotype-config-core into Thegent Config
**Scope:** Consolidate 2,463 LOC of config management
**Current Locations:**
- `crates/phenotype-config-core/src/` (1,429 LOC)
- `platforms/thegent/src/thegent/config/` (1,034 LOC)

**Analysis First:**
1. [ ] Compare both implementations:
   ```bash
   # List modules in phenotype-config-core
   find crates/phenotype-config-core/src -name "*.rs" -exec wc -l {} +

   # List modules in thegent config
   find platforms/thegent/src/thegent/config -name "*.py" -exec wc -l {} +
   ```
2. [ ] Identify overlap in features:
   - [ ] Loading strategy (file, env, defaults)
   - [ ] Validation
   - [ ] Schema definition
   - [ ] Type safety/coercion
3. [ ] Document feature differences

**If Merging Into Thegent Config:**
1. [ ] Create new `phenotype-config` crate (Rust) to be shared
2. [ ] Implement Rust version based on thegent config
3. [ ] Extract shared traits into `phenotype-contracts`
4. [ ] Add `phenotype-config` as dependency in AgilePlus workspace
5. [ ] Update AgilePlus to use `phenotype-config` instead of local version
6. [ ] Update thegent to use `phenotype-config`
7. [ ] Delete old implementations

**Subtasks:**
- [ ] Design config trait interface
- [ ] Implement Rust version with validation
- [ ] Add tests for Rust version
- [ ] Migrate AgilePlus dependencies
- [ ] Migrate Thegent dependencies
- [ ] Document configuration patterns

**Estimated Time:** 1 week (analysis, design, implementation, migration)
**Risk:** MEDIUM - ensure feature parity
**Savings:** 600-800 LOC (unified impl)

---

### Task 2.2: Create Unified phenotype-errors Crate
**Scope:** Consolidate error handling across projects
**Current Implementations:**
- `crates/phenotype-error-core/` (exists but minimal)
- `platforms/thegent/src/thegent/utils/errors.py` (~100 LOC)
- Scattered error enums in both projects

**Steps:**
1. [ ] Audit all error types:
   ```bash
   # Find error definitions in Rust
   grep -r "pub enum.*Error" crates/ | head -20
   grep -r "pub struct.*Error" crates/ | head -20

   # Find in Python
   grep -r "class.*Error\|class.*Exception" platforms/thegent/src | head -20
   ```
2. [ ] Create unified error types:
   - [ ] I/O errors
   - [ ] Git errors
   - [ ] Configuration errors
   - [ ] Validation errors
   - [ ] Sync/merge errors
   - [ ] API errors
3. [ ] Implement From/Into traits for conversions
4. [ ] Add error traits (Display, Debug, Error)
5. [ ] Create Python wrapper if needed
6. [ ] Update dependencies to use unified crate

**Estimated Time:** 3-4 days
**Risk:** LOW - backward compatible via adapters
**Savings:** 100-150 LOC (removed duplication)

---

### Task 2.3: Consolidate Policy Engine (2,664 LOC)
**Scope:** Merge phenotype-policy-engine + thegent-policy
**Current Implementations:**
- `crates/phenotype-policy-engine/` (1,398 LOC) - Rust
- `platforms/thegent/crates/thegent-policy/` (1,266 LOC) - Rust

**Analysis First:**
1. [ ] Compare implementations:
   - [ ] Rule evaluation mechanism
   - [ ] Policy definition format
   - [ ] Execution context
   - [ ] Decision output format
2. [ ] Identify feature union
3. [ ] Design unified API

**If Merging:**
1. [ ] Create `phenotype-policy` crate combining both
2. [ ] Implement unified policy evaluator
3. [ ] Add comprehensive tests
4. [ ] Migrate AgilePlus to use unified crate
5. [ ] Migrate Thegent to use unified crate
6. [ ] Delete old implementations

**Estimated Time:** 1 week
**Risk:** MEDIUM - reconcile API differences
**Savings:** 400-600 LOC

---

### Task 2.4: Unify Cache Implementations (461 LOC)
**Scope:** Consolidate phenotype-cache-adapter + agileplus-cache
**Current:**
- `crates/phenotype-cache-adapter/` (1 LOC stub!)
- `crates/agileplus-cache/` (460 LOC)
- Scattered caching in Thegent

**Steps:**
1. [ ] Assess agileplus-cache implementation
2. [ ] Design cache trait abstraction
3. [ ] Create `phenotype-cache` crate with:
   - [ ] Trait-based interface
   - [ ] Multiple backend implementations (in-memory, disk, Redis)
   - [ ] Serialization support
4. [ ] Update dependencies
5. [ ] Delete old implementations

**Estimated Time:** 4-5 days
**Risk:** LOW - abstraction-based consolidation
**Savings:** 150-200 LOC

---

## Phase 3: Code Refactoring (3-4 weeks)

### Task 3.1: Refactor agileplus-dashboard/routes.rs (2,631 LOC)
**File:** `/crates/agileplus-dashboard/src/routes.rs`
**Issue:** Monolithic route handler file
**Target:** Split into logical modules

**Refactoring Plan:**
1. [ ] Analyze route groups:
   - [ ] HomePage, DashboardPage
   - [ ] FeaturesPage, FeatureDetailPage
   - [ ] SettingsPage (Agent, Services, Plane)
   - [ ] API endpoints (agents, health, events)
   - [ ] Evidence/artifacts
2. [ ] Extract into modules:
   - [ ] `routes/pages/mod.rs` - page handlers
   - [ ] `routes/api/agents.rs` - agent detection
   - [ ] `routes/api/health.rs` - health checks
   - [ ] `routes/api/evidence.rs` - evidence generation
   - [ ] `routes/api/settings.rs` - configuration
   - [ ] `templates/mod.rs` - template utilities
3. [ ] Update imports and references
4. [ ] Add tests for extracted modules
5. [ ] Benchmark to ensure no performance regression

**Estimated Time:** 1 week
**Risk:** MEDIUM - refactoring with router dependencies
**Savings:** 400-600 LOC (structural reduction)

---

### Task 3.2: Modularize agileplus-sqlite/lib.rs (1,582 LOC)
**File:** `/crates/agileplus-sqlite/src/lib.rs`
**Issue:** Mixed concerns (driver, queries, schema)
**Target:** Separate layers

**Refactoring Plan:**
1. [ ] Extract modules:
   - [ ] `schema.rs` - table definitions
   - [ ] `queries/mod.rs` - query builders
   - [ ] `queries/features.rs` - feature queries
   - [ ] `queries/workpackages.rs` - WP queries
   - [ ] `migrations.rs` - schema migrations
   - [ ] `connection.rs` - connection pooling
2. [ ] Create query builder traits
3. [ ] Add type-safe query construction
4. [ ] Update tests

**Estimated Time:** 3-4 days
**Risk:** MEDIUM - database logic is critical
**Savings:** 300-400 LOC

---

### Task 3.3: Split Thegent phench/service.py (2,398 LOC)
**File:** `/platforms/thegent/src/thegent/phench/service.py`
**Issue:** Business logic + HTTP handlers mixed
**Target:** Separate layers

**Refactoring Plan:**
1. [ ] Extract into:
   - [ ] `phench/service.py` - pure business logic
   - [ ] `phench/handlers.py` - FastAPI handlers
   - [ ] `phench/models.py` - data models
   - [ ] `phench/validators.py` - input validation
2. [ ] Add service interface
3. [ ] Decouple handlers from service logic
4. [ ] Update tests

**Estimated Time:** 3-4 days
**Risk:** LOW - isolated to Python module
**Savings:** 200-300 LOC

---

### Task 3.4: Refactor cliproxy_adapter.py (1,267 LOC)
**File:** `/platforms/thegent/src/thegent/cliproxy_adapter.py`
**Issue:** Protocol adapter monolith
**Target:** Extract protocol codecs

**Refactoring Plan:**
1. [ ] Extract into:
   - [ ] `cliproxy/adapter.py` - main adapter
   - [ ] `cliproxy/codecs.py` - message serialization
   - [ ] `cliproxy/protocol.py` - protocol implementation
   - [ ] `cliproxy/errors.py` - error handling
2. [ ] Create codec traits
3. [ ] Add pluggable serialization
4. [ ] Update tests

**Estimated Time:** 2-3 days
**Risk:** LOW - isolated module
**Savings:** 150-200 LOC

---

## Phase 4: Test Optimization (2-3 weeks)

### Task 4.1: Audit Test Duplication in Thegent
**Scope:** 275,479 LOC of tests - likely significant duplication
**Approach:** Statistical sampling + targeted refactoring

**Steps:**
1. [ ] Sample 10% of test files:
   ```bash
   find tests/ -name "*.py" | shuf | head -n $(find tests/ -name "*.py" | wc -l / 10)
   ```
2. [ ] Analyze for:
   - [ ] Duplicate fixtures
   - [ ] Duplicate test scenarios
   - [ ] Parametrize redundancy
   - [ ] Mock setup duplication
3. [ ] Identify consolidation opportunities
4. [ ] Create shared fixture library
5. [ ] Refactor duplicated tests
6. [ ] Measure LOC reduction

**Estimated Time:** 1 week
**Risk:** LOW - test refactoring
**Savings:** 40,000-70,000 LOC (if 15-25% duplication found)

---

### Task 4.2: Add Missing Integration Tests to AgilePlus
**Current Ratio:** 0.15:1 (very low - 8,849 test LOC vs 58,569 source LOC)
**Target Ratio:** 0.3-0.5:1 (industry standard)

**Areas to Test:**
1. [ ] Git merge conflict resolution (git_merge.rs)
2. [ ] P2P sync protocol (p2p module)
3. [ ] Dashboard route handlers (routes.rs)
4. [ ] SQLite schema migrations
5. [ ] GRPC server endpoints
6. [ ] Event sourcing replay

**Estimated Tests to Add:**
- [ ] 20-30 integration tests
- [ ] ~2,000-3,000 LOC
- [ ] Focus on critical paths

**Estimated Time:** 1-2 weeks
**Risk:** LOW - improving coverage
**Note:** This will *increase* LOC in the short term; focus on quality

---

### Task 4.3: Remove Orphaned Test Files
**Scope:** Tests marked @pytest.mark.skip or with only TODO
**Approach:** Selective removal with tracking

**Steps:**
1. [ ] Generate list of skipped tests:
   ```bash
   grep -r "@pytest.mark.skip" tests/ | wc -l
   grep -r "@pytest.mark.xfail" tests/ | head -20
   ```
2. [ ] For each skipped test:
   - [ ] Check skip reason
   - [ ] Determine if:
     - [ ] TODO - create issue, keep @skip
     - [ ] Deprecated - move to archive
     - [ ] Broken - fix or remove
3. [ ] Archive completed tests to `tests/archive/`
4. [ ] Create issues for TODO-marked skipped tests
5. [ ] Update CI to report on skipped test count

**Estimated Time:** 2-3 days (sampling-based)
**Risk:** LOW - skipped tests already not running
**Savings:** 100-500 LOC

---

## Execution Checklist

### Pre-Work
- [ ] Ensure all branches are up-to-date with main
- [ ] Create tracking issues for each phase
- [ ] Allocate team members
- [ ] Set up LOC measurement baseline

### Phase 1 Execution
- [ ] Complete Task 1.1-1.3
- [ ] Verify no regressions
- [ ] Measure savings

### Phase 2 Execution
- [ ] Complete Task 2.1-2.4 (can be parallelized)
- [ ] Ensure all migrations tested
- [ ] Update documentation

### Phase 3 Execution
- [ ] Complete Task 3.1-3.4 (can be parallelized)
- [ ] Code review each refactoring
- [ ] Performance benchmark

### Phase 4 Execution
- [ ] Complete Task 4.1-4.3
- [ ] Update test metrics
- [ ] Validate improvements

### Post-Work
- [ ] Final LOC measurement
- [ ] Generate completion report
- [ ] Update metrics dashboard

---

## Success Metrics

| Metric | Current | Target | Timeline |
|--------|---------|--------|----------|
| **Total LOC** | 524,174 | 451,258-481,774 | Phase 4 |
| **Duplicate Modules** | 6 | 0 | Phase 2 |
| **Empty Crates** | 3 | 0 | Phase 1 |
| **Files >600 LOC** | 12 | 2-4 | Phase 3 |
| **Test Ratio (Thegent)** | 2.44:1 | 1.5:1 | Phase 4 |
| **Test Ratio (AgilePlus)** | 0.15:1 | 0.3-0.5:1 | Phase 4 |
| **TODO Files** | 425 | <50 | Phase 1-4 |

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Breaking dependencies during deletion | Use `grep -r` to verify no callers before deletion |
| Test failures after refactoring | Run full test suite after each refactoring task |
| Performance regression | Benchmark before/after refactoring |
| Incomplete migrations | Code review before merging |
| Merge conflicts | Use feature branches, merge small PRs frequently |

---

## References

- Full Audit Report: `/docs/reports/LOC_AUDIT_REPORT.md`
- Summary: `/docs/reports/LOC_AUDIT_SUMMARY.md`

---

**Status:** Ready to schedule
**Next Step:** Assign team members and create work tickets
**Timeline:** ~1 month for all 4 phases (can be parallelized)
