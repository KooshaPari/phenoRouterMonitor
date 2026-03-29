# Test Fixture Consolidation - Executive Summary

**Audit Date**: 2026-03-29
**Scope**: 15+ test files across AgilePlus and consolidated libraries
**Opportunity**: Eliminate ~650 LOC of fixture duplication
**Confidence Level**: HIGH
**Implementation Effort**: 18 tool calls, ~85 minutes wall-clock
**Status**: AUDIT COMPLETE, READY FOR IMPLEMENTATION

---

## What We Found

### The Problem

Test fixtures and seed data are **duplicated across 15+ test files** with nearly identical implementations:

1. **Feature builders** appear in 3 files with nearly identical code
2. **WorkPackage creation** logic duplicated 5+ times
3. **Test server setup** boilerplate repeated 3+ times
4. **Audit chain construction** duplicated in 2 files
5. **Mock data generation** scattered across test modules

**Total duplicate code**: ~1,800 LOC across AgilePlus ecosystem alone

### Key Findings by Location

| Codebase | Fixture Files | LOC | Primary Issues |
|----------|---------------|-----|----------------|
| **AgilePlus** | 8+ files | ~1,800 | Storage mocks, seed builders, server setup |
| **Consolidated Libraries** | 4 files | ~1,173 | Event/cache/policy test data |
| **heliosCLI** | 5+ files | 400+ | Auth fixtures, schema fixtures, builders |

### Specific Duplication Patterns

**Pattern 1: Feature Construction** (3 files)
```rust
// storage.rs (lines 36-50)
Feature { id: 1, slug: "test-feature", ... }

// fixtures.rs (lines 20-36)
Feature { id: 1, slug: "implement-caching-layer", ... }

// seed.rs (lines 46-59)
Feature::new(...) with manual state transitions
```

**Pattern 2: WorkPackage Creation** (5 files)
```rust
// Each test file recreates this 20+ times
WorkPackage {
    id: i, feature_id: f, title: "WP01",
    state: WpState::Done, pr_url: Some(...),
    // ... 9 more fields to initialize
}
```

**Pattern 3: AppState/TestServer Setup** (3 files)
```rust
// 40 lines of boilerplate repeated:
let creds = InMemoryCredentialStore::new();
creds.set("agileplus", cred_keys::API_KEYS, TEST_API_KEY)?;
let creds_trait: Arc<dyn CredentialStore> = Arc::new(creds);
let config = Arc::new(AppConfig::default());
let state = AppState::new(storage, vcs, telemetry, config, creds_trait);
let app = create_router(state);
let server = TestServer::new(app);
// ... repeat in 3 places
```

---

## Proposed Solution

### Create Shared `test-fixtures-shared` Crate

```
crates/test-fixtures-shared/
├── src/
│   ├── builders/           # Fluent builders for domain objects
│   │   ├── feature_builder.rs       (FeatureFixture)
│   │   ├── work_package_builder.rs  (WorkPackageFixture)
│   │   ├── audit_builder.rs         (AuditChainFixture)
│   │   ├── project_builder.rs       (ProjectFixture)
│   │   ├── cycle_builder.rs         (CycleFixture)
│   │   └── module_builder.rs        (ModuleFixture)
│   ├── factories/          # Factories for test data
│   │   ├── event_factory.rs         (Event creation)
│   │   ├── cache_factory.rs         (Cache test values)
│   │   └── policy_factory.rs        (Policy test objects)
│   ├── mock_storage/       # Shared mock implementations
│   │   ├── mock_storage.rs          (MockStorage struct)
│   │   └── mock_impl.rs             (Port implementations)
│   ├── test_server/        # Test server fixture
│   │   └── server_fixture.rs        (TestServerFixture)
│   ├── seeds/              # Seed data generators
│   │   └── dogfood_seeds.rs         (Dogfood features/WPs)
│   └── lib.rs              (Module organization)
└── tests/
    └── builders_test.rs    (Builder pattern tests)
```

### Example Usage After Migration

```rust
// Before: 20 lines of boilerplate per test file
#[tokio::test]
async fn my_test() {
    let storage = Arc::new(MockStorage::default());
    let vcs = Arc::new(MockVcs);
    let telemetry = Arc::new(MockObs);
    let config = Arc::new(AppConfig::default());

    let creds = InMemoryCredentialStore::new();
    creds.set("agileplus", cred_keys::API_KEYS, TEST_API_KEY)?;
    let creds_trait: Arc<dyn CredentialStore> = Arc::new(creds);

    let state = AppState::new(storage, vcs, telemetry, config, creds_trait);
    let app = create_router(state);
    let server = TestServer::new(app);
    // ... test logic
}

// After: 2-3 lines using fixtures
#[tokio::test]
async fn my_test() {
    let server = TestServerFixture::new().await;

    let feature = FeatureFixture::new("my-feature", "My Feature")
        .id(1)
        .with_shipped()
        .build();

    let wp = WorkPackageFixture::new(feature.id, "WP01")
        .state(WpState::Done)
        .with_pr("https://github.com/org/repo/pull/1")
        .build();

    // ... test logic
}
```

---

## Detailed Implementation Plan

### Phase Overview

| Phase | Name | Files | Effort | Duration |
|-------|------|-------|--------|----------|
| **WP1** | Scaffolding | 4 new files | 4 tool calls | 15 min |
| **WP2** | Core Infrastructure | 6 builder/factory files | 5 tool calls | 20 min |
| **WP3** | AgilePlus Migration | 8 test files updated | 6 tool calls | 25 min |
| **WP4** | Consolidated Libs | 4 test files updated | 4 tool calls | 15 min |
| **WP5** | Validation & Docs | 2 docs + verification | 3 tool calls | 10 min |
| — | **TOTAL** | — | **18 tool calls** | **85 min** |

### Phase 1: Scaffolding (4 tool calls, 15 min)

1. Create `crates/test-fixtures-shared/Cargo.toml`
2. Create `src/lib.rs` with module organization
3. Create `src/builders/mod.rs` with re-exports
4. Create `src/factories/mod.rs` with re-exports

### Phase 2: Core Infrastructure (5 tool calls, 20 min)

1. Implement `FeatureFixture` builder (60 LOC)
2. Implement `WorkPackageFixture` builder (50 LOC)
3. Implement `AuditChainFixture` builder (40 LOC)
4. Implement `MockStorage` and port implementations (150 LOC)
5. Implement `TestServerFixture` (50 LOC)

### Phase 3: AgilePlus Migration (6 tool calls, 25 min)

1. Update `support/storage.rs` to use `MockStorage`
2. Update `support/mod.rs` to use `TestServerFixture`
3. Update storage_port_impl files to use builders
4. Update `dashboard/src/seed.rs` to use factories
5. Update API integration tests (5 files)
6. Verify all tests pass

### Phase 4: Consolidated Libraries (4 tool calls, 15 min)

1. Update event sourcing tests to use factories
2. Update cache adapter tests
3. Update policy engine tests
4. Update state machine tests

### Phase 5: Validation & Docs (3 tool calls, 10 min)

1. Run full test suite verification
2. Create `FIXTURE_MIGRATION_GUIDE.md` for future tests
3. Update workspace Cargo.toml dependencies

---

## Expected Outcomes

### Code Reduction

| Category | Before | After | Savings |
|----------|--------|-------|---------|
| Fixture boilerplate | ~1,800 LOC | ~425 LOC (shared) | ~1,375 LOC |
| Test file duplication | Scattered | ~415 LOC per 5 files | ~650 LOC |
| **Total Elimination** | — | — | **~650 LOC** |

### Quality Improvements

1. **Consistency**: All tests use same fixture patterns
2. **Maintainability**: Single source of truth for builders/factories
3. **Readability**: Test code focuses on logic, not setup
4. **Extensibility**: Add new builders to shared crate; all tests benefit
5. **Reusability**: New tests can copy-paste working examples

### Risk Mitigation

- Builders have sensible defaults; rarely need customization
- MockStorage preserves exact same behavior (just centralized)
- TestServerFixture delegates to exact same initialization logic
- All existing tests remain logically unchanged; just cleaner syntax
- Full test suite verification catches any regressions

---

## File Locations & References

### Key Files to Create

1. `/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/AgilePlus/phenotype-docs/crates/test-fixtures-shared/Cargo.toml`
2. `/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/AgilePlus/phenotype-docs/crates/test-fixtures-shared/src/lib.rs`
3. `/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/AgilePlus/phenotype-docs/crates/test-fixtures-shared/src/builders/*.rs` (6 files)
4. `/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/AgilePlus/phenotype-docs/crates/test-fixtures-shared/src/factories/*.rs` (3 files)
5. `/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/AgilePlus/phenotype-docs/crates/test-fixtures-shared/src/mock_storage/*.rs` (2 files)
6. `/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/AgilePlus/phenotype-docs/crates/test-fixtures-shared/src/test_server/*.rs` (1 file)

### Key Files to Update

**AgilePlus API Tests**:
- `crates/agileplus-api/tests/api_integration/support/storage.rs`
- `crates/agileplus-api/tests/api_integration/support/mod.rs`
- `crates/agileplus-api/tests/api_integration/support/storage_port_impl/*.rs`
- `crates/agileplus-api/tests/api_integration/features_work_packages.rs`
- `crates/agileplus-api/tests/api_integration/module_cycle.rs`

**AgilePlus Dashboard & Integration**:
- `crates/agileplus-dashboard/src/seed.rs`
- `crates/agileplus-dashboard/tests/seed_integration.rs`
- `crates/agileplus-integration-tests/src/common/fixtures.rs`

**Consolidated Libraries**:
- `crates/phenotype-event-sourcing/tests/event_store.rs`
- `crates/phenotype-cache-adapter/tests/cache_adapter.rs`
- `crates/phenotype-policy-engine/tests/policy_engine.rs`
- `crates/phenotype-state-machine/tests/state_machine.rs`

### Documentation Files

1. **Audit Report**: `/Users/kooshapari/CodeProjects/Phenotype/repos/FIXTURE_CONSOLIDATION_AUDIT.md` ✓ (created)
2. **Implementation Plan**: `/Users/kooshapari/CodeProjects/Phenotype/repos/FIXTURE_CONSOLIDATION_IMPLEMENTATION_PLAN.md` ✓ (created)
3. **Migration Guide** (to be created in WP5): `FIXTURE_MIGRATION_GUIDE.md`

---

## Success Criteria

- [x] Audit complete and documented
- [ ] New `test-fixtures-shared` crate scaffolded
- [ ] All builders and factories implemented
- [ ] 15+ test files migrated to use shared fixtures
- [ ] All tests pass with no regressions (green CI)
- [ ] ~650 LOC fixture duplication eliminated
- [ ] Builders + factories documented with code examples
- [ ] Migration guide created for future test writers

---

## Next Steps

**Immediate Actions**:
1. Review audit and implementation plan
2. Confirm target repositories and crate locations
3. Designate WP owners if parallel execution desired

**Ready to Execute**:
- WP1 (Scaffolding): 4 tool calls, ready to go
- WP2 (Infrastructure): Depends on WP1, 5 tool calls
- WP3 (Migration): Can run parallel with WP2, 6 tool calls
- WP4 (Libs): Depends on WP2, 4 tool calls
- WP5 (Validation): Depends on WP3+WP4, 3 tool calls

**Estimated Timeline**: 85 minutes wall-clock if executed sequentially; ~50 minutes if WP2+WP3 run in parallel.

---

## Questions & Clarifications

**Q1**: Should `test-fixtures-shared` be published as a separate crate?
**A1**: Initially keep it in workspace. Can extract later if needed for cross-org reuse (e.g., heliosCLI).

**Q2**: What about database-specific fixtures (SQLite, PostgreSQL)?
**A2**: Phase 2 work; keep in separate `test-fixtures-db` crate if needed.

**Q3**: Should we consolidate heliosCLI fixtures too?
**A3**: Recommend Phase 2 after validating approach on AgilePlus + consolidated-libraries.

**Q4**: How to handle future test file additions?
**A4**: Migration guide will include "New Test File Checklist" with fixture imports.

---

## Appendix: Sample Builder API

```rust
// Feature Builder
pub struct FeatureFixture { /* ... */ }

impl FeatureFixture {
    pub fn new(slug: &str, friendly_name: &str) -> Self { /* ... */ }
    pub fn id(mut self, id: i64) -> Self { /* ... */ }
    pub fn state(mut self, state: FeatureState) -> Self { /* ... */ }
    pub fn with_shipped(mut self) -> Self { /* ... */ }
    pub fn with_pr_url(mut self, url: &str) -> Self { /* ... */ }
    pub fn with_label(mut self, label: &str) -> Self { /* ... */ }
    pub fn with_project_id(mut self, id: i64) -> Self { /* ... */ }
    pub fn build(self) -> Feature { /* ... */ }
}

// WorkPackage Builder
pub struct WorkPackageFixture { /* ... */ }

impl WorkPackageFixture {
    pub fn new(feature_id: i64, title: &str) -> Self { /* ... */ }
    pub fn id(mut self, id: i64) -> Self { /* ... */ }
    pub fn state(mut self, state: WpState) -> Self { /* ... */ }
    pub fn with_pr(mut self, pr_url: &str) -> Self { /* ... */ }
    pub fn with_sequence(mut self, seq: i32) -> Self { /* ... */ }
    pub fn build(self) -> WorkPackage { /* ... */ }
}

// Audit Chain Builder
pub struct AuditChainFixture { /* ... */ }

impl AuditChainFixture {
    pub fn genesis(feature_id: i64) -> Self { /* ... */ }
    pub fn with_entry(self, wp_id: Option<i64>, transition: &str) -> Self { /* ... */ }
    pub fn build(self) -> Vec<AuditEntry> { /* ... */ }
}

// Test Server Fixture
pub struct TestServerFixture { server: TestServer }

impl TestServerFixture {
    pub async fn new() -> Self { /* ... */ }
    pub async fn with_storage(storage: MockStorage) -> Self { /* ... */ }
    pub fn test_api_key() -> &'static str { "test-api-key-12345" }
}
```

---

**Document Version**: 1.0
**Status**: AUDIT COMPLETE
**Ready for Implementation**: YES
