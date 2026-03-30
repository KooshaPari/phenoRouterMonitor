# LOC Audit - Executive Summary

**Full Report:** `/docs/reports/LOC_AUDIT_REPORT.md`

## Key Metrics

| Metric | Value |
|--------|-------|
| **Total LOC Analyzed** | 524,174 |
| **Rust** | 111,649 LOC (21.3%) |
| **Python** | 408,428 LOC (77.8%) |
| **Projects** | 3 (AgilePlus, Thegent, Phench) |
| **Crates/Modules** | 77 |

## Critical Findings

### 1. Empty/Stub Crates (Delete Immediately)
- `phenotype-state-machine` - 0 LOC
- `phenotype-git-core` - 1 LOC
- `phenotype-cache-adapter` - 1 LOC

**Action:** Remove from workspace, update dependencies to use full implementations.

### 2. Duplicate Modules (Consolidate)

| Module | Duplication |
|--------|---|
| Config Management | 2,463 LOC (phenotype-config-core + thegent config) |
| Policy Engine | 2,664 LOC (phenotype-policy-engine + thegent-policy) |
| Cache Abstractions | 461 LOC (phenotype-cache + agileplus-cache) |
| Error Handling | ~200 LOC (phenotype-error-core + thegent error utils) |

**Action:** Merge into single shared crates.

### 3. Oversized Files (Refactor)

| File | LOC | Recommendation |
|------|-----|---|
| agileplus-dashboard/routes.rs | 2,631 | Split into 6-8 modules |
| agileplus-sqlite/lib.rs | 1,582 | Extract query builder |
| thegent/phench/service.py | 2,398 | Split service/handler layers |
| thegent/cliproxy_adapter.py | 1,267 | Extract protocol codecs |

**Action:** Refactor to reduce cognitive complexity.

### 4. Test Imbalance

| Project | Test Ratio | Status |
|---------|--|--|
| AgilePlus | 0.15:1 | TOO LOW - need more tests |
| Thegent | 2.44:1 | TOO HIGH - likely duplication |
| Phench | Unknown | Need assessment |

**Action:** Audit test duplication in Thegent, add integration tests to AgilePlus.

## Estimated Savings

| Category | Potential Reduction | Effort |
|----------|--|--|
| **Delete stubs** | 134 LOC | 2-4 hours |
| **Consolidate modules** | 1,250-1,750 LOC | 1-2 weeks |
| **Refactor large files** | 1,150-1,650 LOC | 3-4 weeks |
| **Optimize tests** | 40,000-70,000 LOC | 2-3 weeks |
| **TOTAL** | **42,400-72,916 LOC** | **~1 month** |

**Impact:** 8-14% codebase reduction with improved quality.

## Recommended Execution Order

### Phase 1: Quick Wins (2-4 hours)
1. Delete 3 stub crates
2. Remove/complete phench stubs

### Phase 2: Consolidation (1-2 weeks)
1. Merge config modules
2. Unify error handling
3. Consolidate cache abstractions
4. Unify policy engine

### Phase 3: Refactoring (3-4 weeks)
1. Split agileplus-dashboard routes
2. Modularize agileplus-sqlite
3. Split thegent service.py
4. Refactor cliproxy_adapter.py

### Phase 4: Test Optimization (2-3 weeks)
1. Audit test duplication
2. Remove orphaned tests
3. Add missing integration tests

## Cross-Project Reuse Opportunities

Proposed new shared crates:
- `phenotype-config` - unified configuration
- `phenotype-errors` - error type definitions
- `phenotype-policy` - policy evaluation engine
- `phenotype-cache` - pluggable cache abstraction
- `phenotype-git` - git operations (merge, materialize)
- `phenotype-state-machine` - complete FSM implementation

**Benefit:** Eliminate duplication, shared interfaces, code reuse across all 3 projects.

---

**Full analysis available in:** `/docs/reports/LOC_AUDIT_REPORT.md`
