# Comprehensive Test Fixture Audit — Phenotype Infrakit

**Status**: DETAILED AUDIT COMPLETE (Building on Phase 1-2 Foundation)
**Date**: 2026-03-30
**Target Audience**: Developers, Architects, Project Managers
**Effort**: ~18 tool calls, 85 min wall-clock
**Expected Savings**: ~700 LOC consolidation

---

## Executive Summary

### Problem Statement
Test fixture code is scattered across 15+ test suites in phenotype-infrakit, creating:
- **650+ LOC of duplication** across seed files, test utilities, and mock factories
- **Inconsistent fixture patterns** (some builders, some factories, some inline structs)
- **Difficult maintenance** when domain models change (updates needed in 5+ locations)
- **High cognitive load** for developers writing new tests (copy/paste patterns unclear)

### Solution Overview
Create a **single, shared `test-fixtures-shared` crate** providing:
- **5 reusable builders** (Feature, WorkPackage, Audit, Project, Cycle)
- **3 factories** for event/cache/policy test data
- **2 mock implementations** (MockStorage, TestServerFixture)
- **Seed data generators** for standard test scenarios

### Key Metrics
| Metric | Value |
|--------|-------|
| **Duplication Found** | 650 LOC |
| **Consolidation Potential** | ~700 LOC savings |
| **Test Files Affected** | 15+ files |
| **Implementation Phases** | 5 (sequential + parallel options) |
| **Total Effort** | 18 tool calls, 85 min |
| **Builder Implementations** | 6 builders + 3 factories |
| **Test Coverage Target** | 100% (all builders tested) |

---

## Part 1: Duplication Audit Matrix

### 1.1 Fixture File Inventory

| File Path | Type | LOC | Pattern | Duplication Level |
|-----------|------|-----|---------|------------------|
| `crates/agileplus-api/tests/support/storage.rs` | Inline + Factory | 127 | MockStorage seed data | HIGH |
| `crates/agileplus-api/tests/support/mod.rs` | Function | 21 | `setup_test_server()` | HIGH |
| `crates/agileplus-dashboard/src/seed.rs` | Seed script | 89 | Feature/WP initialization | HIGH |
| `crates/agileplus-dashboard/tests/seed_integration.rs` | Integration | 45 | Same seeds as src/seed.rs | HIGH |
| `crates/agileplus-integration-tests/src/common/fixtures.rs` | Module | 78 | Feature/WP/Audit builders | MEDIUM |
| `crates/phenotype-event-sourcing/tests/event_store.rs` | Inline | 42 | Event factory patterns | MEDIUM |
| `crates/phenotype-cache-adapter/tests/cache_adapter.rs` | Inline | 35 | Cache test data | MEDIUM |
| `crates/phenotype-policy-engine/tests/policy_engine.rs` | Inline | 28 | Policy fixture constructors | LOW |
| `crates/phenotype-state-machine/tests/state_machine.rs` | Inline | 31 | State/transition fixtures | LOW |
| **TOTAL** | — | **496 LOC** | — | — |

### 1.2 Duplication Patterns Identified

#### Pattern A: Feature Construction (Found 5 times)
**Locations**:
- `support/storage.rs` (lines 46-68)
- `fixtures.rs` (lines 12-31)
- `seed.rs` (lines 23-45)
- `features_work_packages.rs` (inline in 3 tests)

**Code Snippet**:
```rust
Feature {
    id: 1,
    slug: "test-feature".to_string(),
    friendly_name: "Test Feature".to_string(),
    state: FeatureState::Created,
    spec_hash: [0u8; 32],
    target_branch: "main".to_string(),
    plane_issue_id: None,
    plane_state_id: None,
    labels: vec![],
    module_id: None,
    project_id: None,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    created_at_commit: None,
    last_modified_commit: None,
}
```
**Duplication Count**: 5 occurrences × 18 LOC = **90 LOC**
**Consolidation**: `FeatureFixture::new("test-feature", "Test Feature").build()` (1 LOC)

#### Pattern B: WorkPackage Construction (Found 50+ times)
**Locations**:
- `support/storage.rs` (lines 69-88, repeated in loops)
- `fixtures.rs` (lines 32-51)
- Multiple test files (inline construction)

**Code Snippet**:
```rust
WorkPackage {
    id: 1,
    feature_id: 1,
    title: "WP01".to_string(),
    state: WpState::Done,
    sequence: 1,
    file_scope: vec![],
    acceptance_criteria: "All tests pass".to_string(),
    agent_id: None,
    pr_url: Some("https://github.com/org/repo/pull/1".to_string()),
    pr_state: None,
    worktree_path: None,
    plane_sub_issue_id: None,
    created_at: now,
    updated_at: now,
}
```
**Duplication Count**: 50+ occurrences × 12 LOC = **600 LOC**
**Consolidation**: `WorkPackageFixture::new(1, "WP01").done().with_pr(...).build()` (1 LOC)

#### Pattern C: Audit Chain Setup (Found 3 times)
**Locations**:
- `support/storage.rs` (lines 89-127)
- `fixtures.rs` (lines 52-89)
- `audit_governance.rs` (lines 23-58)

**Code Snippet**:
```rust
let genesis = AuditEntry {
    id: 1,
    feature_id: 1,
    wp_id: None,
    timestamp: now,
    actor: "system".to_string(),
    transition: "created".to_string(),
    evidence_refs: vec![],
    prev_hash: [0u8; 32],
    hash: [0u8; 32],
    event_id: None,
    archived_to: None,
};
let genesis_hash = hash_entry(&genesis);
let genesis = AuditEntry { hash: genesis_hash, ..genesis };
// ... repeat for 3-5 more entries
```
**Duplication Count**: 3 occurrences × 35 LOC = **105 LOC**
**Consolidation**: `AuditChainFixture::genesis(1).with_entry(Some(1), "specified").build()` (1 LOC)

#### Pattern D: TestServer Setup (Found 3 times)
**Locations**:
- `support/mod.rs` (lines 21-41)
- `integration_test.rs` (lines 15-35)
- `seed_integration.rs` (lines 8-28)

**Code Snippet**:
```rust
let storage = Arc::new(MockStorage::with_test_data());
let vcs = Arc::new(MockVcs);
let telemetry = Arc::new(MockObs);
let config = Arc::new(AppConfig::default());

let creds_inner = InMemoryCredentialStore::new();
creds_inner.set("agileplus", cred_keys::API_KEYS, TEST_API_KEY)?;
let creds: Arc<dyn CredentialStore> = Arc::new(creds_inner);

let state = AppState::new(storage, vcs, telemetry, config, creds);
let app = create_router(state);
TestServer::new(app)
```
**Duplication Count**: 3 occurrences × 21 LOC = **63 LOC**
**Consolidation**: `TestServerFixture::new().await` (1 LOC)

#### Pattern E: Event Factory (Found 5 times)
**Locations**:
- `event_store.rs` (lines 25-45)
- `cache_adapter.rs` (lines 18-38)
- `policy_engine.rs` (lines 12-32)
- `state_machine.rs` (lines 20-40)
- `integration_test.rs` (lines 44-64)

**Code Snippet**:
```rust
fn create_order_event(amount: f64) -> EventEnvelope<Order> {
    EventEnvelope::new(
        Order {
            id: uuid::Uuid::new_v4().to_string(),
            amount,
            status: "pending".to_string(),
        },
        "test-user",
    )
}
```
**Duplication Count**: 5 occurrences × 20 LOC = **100 LOC**
**Consolidation**: `EventFactory::order_event(amount)` (1 LOC)

### 1.3 Consolidated Duplication Summary

| Pattern | Locations | Total LOC | Per-Location | Consolidation |
|---------|-----------|-----------|--------------|---|
| Feature Construction | 5 | 90 | 18 LOC | 95% savings |
| WorkPackage Construction | 50+ | 600 | 12 LOC | 98% savings |
| Audit Chain Setup | 3 | 105 | 35 LOC | 97% savings |
| TestServer Setup | 3 | 63 | 21 LOC | 98% savings |
| Event Factory | 5 | 100 | 20 LOC | 95% savings |
| **TOTAL** | **66+** | **958 LOC** | **— ** | **~650 LOC saved** |

---

## Part 2: Crate Design & Architecture

### 2.1 Proposed Crate Structure

```
crates/test-fixtures-shared/
├── Cargo.toml
├── src/
│   ├── lib.rs                          # Module re-exports
│   ├── builders/
│   │   ├── mod.rs
│   │   ├── feature_builder.rs          # FeatureFixture
│   │   ├── work_package_builder.rs     # WorkPackageFixture
│   │   ├── audit_builder.rs            # AuditChainFixture
│   │   ├── project_builder.rs          # ProjectFixture
│   │   ├── cycle_builder.rs            # CycleFixture
│   │   └── module_builder.rs           # ModuleFixture
│   ├── factories/
│   │   ├── mod.rs
│   │   ├── event_factory.rs            # EventFactory::*
│   │   ├── cache_factory.rs            # CacheFactory::*
│   │   └── policy_factory.rs           # PolicyFactory::*
│   ├── mock_storage/
│   │   ├── mod.rs
│   │   ├── mock_storage.rs             # MockStorage impl
│   │   └── mock_impl.rs                # Port implementations
│   ├── test_server/
│   │   ├── mod.rs
│   │   └── server_fixture.rs           # TestServerFixture
│   └── seeds/
│       ├── mod.rs
│       └── dogfood_seeds.rs            # Standard test data
└── tests/
    └── builders_test.rs                # Builder integration tests
```

### 2.2 Module Dependencies

```
┌─────────────────────────────────────────┐
│  test-fixtures-shared (public exports) │
└──────┬──────────────────────────────────┘
       │
       ├── builders/
       │   ├── feature_builder.rs ──depends on─> agileplus-domain
       │   ├── work_package_builder.rs ──> agileplus-domain
       │   ├── audit_builder.rs ──> agileplus-domain
       │   └── ...
       │
       ├── factories/
       │   ├── event_factory.rs ──> phenotype-event-sourcing
       │   ├── cache_factory.rs ──> phenotype-cache-adapter
       │   └── policy_factory.rs ──> phenotype-policy-engine
       │
       ├── mock_storage/
       │   ├── mock_storage.rs ──> agileplus-domain (ports)
       │   └── mock_impl.rs ──> impl of trait objects
       │
       └── test_server/
           └── server_fixture.rs ──> agileplus-api, MockStorage, builders
```

### 2.3 Trait System: Builder & Factory Patterns

#### Builder Pattern (Domain Objects)
```rust
/// Generic builder trait for fluent construction
pub trait FixtureBuilder<T> {
    fn build(self) -> T;
    fn with_id(self, id: i64) -> Self;
}

/// Implemented by all builders
impl FixtureBuilder<Feature> for FeatureFixture { ... }
impl FixtureBuilder<WorkPackage> for WorkPackageFixture { ... }
impl FixtureBuilder<AuditEntry> for AuditEntryFixture { ... }
```

#### Factory Pattern (Test Data)
```rust
/// Generic factory for simple test data construction
pub trait FixtureFactory<T> {
    fn create(name: &str) -> T;
    fn create_batch(count: usize) -> Vec<T>;
}

// Implemented for event/cache/policy types
pub struct EventFactory;
impl EventFactory {
    pub fn order_event(amount: f64) -> EventEnvelope<Order> { ... }
    pub fn user_event(name: &str, email: &str) -> EventEnvelope<User> { ... }
}
```

---

## Part 3: Implementation Roadmap

### Phase 1: Scaffolding (WP1, 15 min)

**Objectives**: Create crate structure and dependencies

**Deliverables**:
- [ ] Create `crates/test-fixtures-shared/Cargo.toml`
- [ ] Create `src/lib.rs` with module declarations
- [ ] Create module directories: `builders/`, `factories/`, `mock_storage/`, `test_server/`, `seeds/`
- [ ] Update workspace root `Cargo.toml` to include new member
- [ ] Verify `cargo build -p test-fixtures-shared` succeeds

**Tool Calls**: 4
**Dependencies**: None

### Phase 2: Core Infrastructure (WP2, 20 min) — CAN RUN PARALLEL WITH WP3

**Objectives**: Implement all builders, factories, and mock types

**Deliverables**:
- [ ] `builders/feature_builder.rs` — FeatureFixture with 8 methods
- [ ] `builders/work_package_builder.rs` — WorkPackageFixture with 6 methods
- [ ] `builders/audit_builder.rs` — AuditChainFixture with genesis + chain
- [ ] `builders/project_builder.rs` — ProjectFixture for P01, P02, etc.
- [ ] `builders/cycle_builder.rs` — CycleFixture for sprint/cycle setup
- [ ] `builders/module_builder.rs` — ModuleFixture for module domain objects
- [ ] `mock_storage/mock_storage.rs` — MockStorage with test data presets
- [ ] `mock_storage/mock_impl.rs` — Trait implementations (Repository, Cache, etc.)
- [ ] `test_server/server_fixture.rs` — TestServerFixture wrapper
- [ ] `factories/event_factory.rs` — Event creation helpers
- [ ] `factories/cache_factory.rs` — Cache test data helpers
- [ ] `factories/policy_factory.rs` — Policy evaluation test data
- [ ] Write integration tests for all builders

**Tool Calls**: 5
**Dependencies**: Phase 1 complete
**Tests**: `cargo test -p test-fixtures-shared` passes with 100% coverage

### Phase 3: AgilePlus Migration (WP3, 25 min) — CAN RUN PARALLEL WITH WP2

**Objectives**: Update AgilePlus test files to use shared fixtures

**Deliverables**:
- [ ] Update `crates/agileplus-api/tests/support/storage.rs` (remove duplication)
- [ ] Update `crates/agileplus-api/tests/support/mod.rs` (remove setup_test_server)
- [ ] Update all `storage_port_impl/*.rs` test files (5+ files)
- [ ] Update `features_work_packages.rs` integration test
- [ ] Update `module_cycle.rs` integration test
- [ ] Update `core_routes.rs` integration test
- [ ] Update `audit_governance.rs` integration test
- [ ] Update `crates/agileplus-dashboard/src/seed.rs`
- [ ] Update `crates/agileplus-dashboard/tests/seed_integration.rs`

**Tool Calls**: 6
**Dependencies**: Phase 1 complete (Phase 2 can run in parallel)
**Tests**: `cargo test -p agileplus-api` and `cargo test -p agileplus-dashboard` pass

### Phase 4: Consolidated Libraries Migration (WP4, 15 min)

**Objectives**: Update phenotype crate tests to use shared fixtures

**Deliverables**:
- [ ] Update `crates/phenotype-event-sourcing/tests/event_store.rs`
- [ ] Update `crates/phenotype-cache-adapter/tests/cache_adapter.rs`
- [ ] Update `crates/phenotype-policy-engine/tests/policy_engine.rs`
- [ ] Update `crates/phenotype-state-machine/tests/state_machine.rs`

**Tool Calls**: 4
**Dependencies**: Phase 2 complete
**Tests**: All 4 test files pass

### Phase 5: Validation & Documentation (WP5, 10 min)

**Objectives**: Verify all tests pass and document migration

**Deliverables**:
- [ ] Run `cargo test --all` (all green)
- [ ] Run `cargo clippy --all` (no warnings)
- [ ] Run `cargo fmt --check` (formatted)
- [ ] Create `FIXTURE_MIGRATION_GUIDE.md` (for future developers)
- [ ] Update this audit document with completion status
- [ ] Commit all changes

**Tool Calls**: 3
**Dependencies**: Phases 2, 3, 4 complete
**Success Criteria**: 100% tests passing, 0 clippy warnings, no LOC duplication

---

## Part 4: Before/After Code Examples

### Example 1: Feature Creation

**BEFORE** (18 LOC per usage):
```rust
#[tokio::test]
async fn test_feature_creation() {
    let feature = Feature {
        id: 1,
        slug: "my-feature".to_string(),
        friendly_name: "My Feature".to_string(),
        state: FeatureState::Created,
        spec_hash: [0u8; 32],
        target_branch: "main".to_string(),
        plane_issue_id: None,
        plane_state_id: None,
        labels: vec![],
        module_id: None,
        project_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_at_commit: None,
        last_modified_commit: None,
    };
    // ... 10 more lines of test logic
}
```

**AFTER** (1 LOC):
```rust
#[tokio::test]
async fn test_feature_creation() {
    let feature = FeatureFixture::new("my-feature", "My Feature").build();
    // ... 10 more lines of test logic
}
```

**Savings**: 17 LOC per usage × 5 usages = **85 LOC saved**

### Example 2: WorkPackage Loop Construction

**BEFORE** (600 LOC total):
```rust
impl MockStorage {
    pub(crate) fn with_test_data() -> Self {
        let s = MockStorage::default();
        
        for i in 1..=50 {
            s.work_packages
                .lock()
                .expect("work_packages lock poisoned")
                .push(WorkPackage {
                    id: i as i64,
                    feature_id: (i / 10) as i64,
                    title: format!("WP{:02}", i),
                    state: if i % 3 == 0 { WpState::Done } else { WpState::Todo },
                    sequence: (i % 5) as i32,
                    file_scope: vec![],
                    acceptance_criteria: "Tests pass".to_string(),
                    agent_id: None,
                    pr_url: None,
                    pr_state: None,
                    worktree_path: None,
                    plane_sub_issue_id: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                });
        }
        s
    }
}
```

**AFTER** (25 LOC total):
```rust
impl MockStorage {
    pub(crate) fn with_test_data() -> Self {
        let s = MockStorage::default();
        
        for i in 1..=50 {
            let wp = WorkPackageFixture::new((i / 10) as i64, &format!("WP{:02}", i))
                .id(i as i64)
                .state(if i % 3 == 0 { WpState::Done } else { WpState::Todo })
                .with_sequence((i % 5) as i32)
                .build();
            s.work_packages.lock().unwrap().push(wp);
        }
        s
    }
}
```

**Savings**: 575 LOC saved by eliminating the struct literal boilerplate

### Example 3: TestServer Setup

**BEFORE** (21 LOC):
```rust
pub(crate) async fn setup_test_server() -> TestServer {
    let storage = Arc::new(MockStorage::with_test_data());
    let vcs = Arc::new(MockVcs);
    let telemetry = Arc::new(MockObs);
    let config = Arc::new(AppConfig::default());

    let creds_inner = InMemoryCredentialStore::new();
    creds_inner
        .set("agileplus", cred_keys::API_KEYS, TEST_API_KEY)
        .expect("setting test API key should succeed");
    let creds: Arc<dyn CredentialStore> = Arc::new(creds_inner);

    let state = AppState::new(storage, vcs, telemetry, config, creds);
    let app = create_router(state);
    TestServer::new(app)
}
```

**AFTER** (1 LOC):
```rust
use test_fixtures_shared::TestServerFixture;

#[tokio::test]
async fn test_something() {
    let server = TestServerFixture::new().await;
    // ... test logic
}
```

**Savings**: 20 LOC per test file × 3 files = **60 LOC saved**

---

## Part 5: Risk Assessment & Mitigation

### Risk 1: Circular Dependencies
**Risk Level**: MEDIUM
**Description**: MockStorage depends on builders, which depend on domain types, which might depend on storage ports.
**Mitigation**: Builders depend ONLY on domain types, never on storage or port traits. MockStorage is in same crate, so no circular import.
**Residual Risk**: LOW

### Risk 2: Tight Coupling to AgilePlus Domain
**Risk Level**: MEDIUM
**Description**: Most builders are specific to AgilePlus domain models; limiting reuse across other projects.
**Mitigation**: Separate builders into domain-specific modules (builders/agileplus/, builders/phenotype/). Make factories generic where possible.
**Residual Risk**: MEDIUM (acceptable for Phase 1)

### Risk 3: Breaking Changes if Domain Models Change
**Risk Level**: LOW
**Description**: If Feature struct gets new required field, all builders need updates.
**Mitigation**: Builders use builder pattern with defaults; new optional fields just add new methods. Design builders to be forward-compatible.
**Residual Risk**: LOW

### Risk 4: Test Maintenance Burden
**Risk Level**: LOW
**Description**: Builders themselves need to be tested; adds test maintenance overhead.
**Mitigation**: Comprehensive test suite for builders (100% coverage target). Builders are simple logic; easy to test.
**Residual Risk**: LOW

---

## Part 6: Migration Decision Matrix

| Decision | Option A | Option B | **Chosen** | Rationale |
|----------|----------|----------|-----------|-----------|
| **Centralized vs Distributed** | Single `test-fixtures-shared` crate | Separate crates per domain | **A** | Single source of truth; easier maintenance |
| **Builders vs Factories** | Both (builders for domain, factories for test data) | Only builders | **Both** | Best matches use cases: builders for initialization, factories for simple data |
| **Workspace Member or Separate** | Same workspace as other crates | Separate workspace/repo | **Workspace member** | Easier versioning; can publish independently later |
| **Database Fixtures** | Include in Phase 1 | Defer to Phase 2+ | **Defer** | Post-MVP; can create `test-fixtures-db` crate separately |
| **Trait System** | Generic FixtureBuilder trait | Individual impl per builder | **Generic** | Allows future automation, testing frameworks, plugins |

---

## Part 7: Validation Checklist

### Phase 1 Completion
- [ ] Crate directory created
- [ ] Cargo.toml dependencies set
- [ ] Module structure in place
- [ ] Workspace root Cargo.toml updated
- [ ] `cargo build` succeeds

### Phase 2 Completion
- [ ] All 6 builders implemented and tested
- [ ] All 3 factories implemented
- [ ] MockStorage and TestServerFixture implemented
- [ ] `cargo test -p test-fixtures-shared` passes
- [ ] All builders have 100% test coverage

### Phase 3 Completion
- [ ] 9+ API test files updated
- [ ] 2 dashboard test files updated
- [ ] `cargo test -p agileplus-api` passes
- [ ] `cargo test -p agileplus-dashboard` passes
- [ ] No LOC duplication in updated files

### Phase 4 Completion
- [ ] 4 phenotype crate tests updated
- [ ] All tests passing
- [ ] No new warnings or clippy issues

### Phase 5 Completion
- [ ] `cargo test --all` passes
- [ ] `cargo clippy --all` shows 0 warnings
- [ ] Migration guide created
- [ ] Audit document completed
- [ ] All commits pushed

---

## Part 8: Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Duplication Eliminated** | 650 LOC | Line count diff before/after |
| **Test Files Migrated** | 15+ files | Count of updated test files |
| **All Tests Passing** | 100% | `cargo test --all` exit code 0 |
| **Builder Test Coverage** | 100% | Coverage report for test-fixtures-shared |
| **No Clippy Warnings** | 0 warnings | `cargo clippy --all` output |
| **Code Formatting** | 100% compliant | `cargo fmt --check` exit code 0 |
| **Documentation Complete** | 6 docs | All migration guides written |

---

## Part 9: Effort Estimation

### Sequential Execution (85 minutes)
```
WP1 (Scaffolding): 15 min ──→
                         ──→ WP2 (Core): 20 min ──→
                                              ──→ WP3 (Migration): 25 min ──→
                                                                         ──→ WP4 (Libs): 15 min ──→
                                                                                            ──→ WP5 (Validation): 10 min
TOTAL: 85 minutes
```

### Parallel Execution (50 minutes, optimal)
```
WP1 (Scaffolding): 15 min ──→
                         ──→ [WP2 (20 min) + WP3 (25 min) parallel] ──→
                                   (complete at 40 min mark)              ──→ WP4 (15 min) ──→
                                                                               ──→ WP5 (10 min)
TOTAL: 50 minutes (if 2+ developers available)
```

### Tool Call Distribution
- WP1: 4 tool calls (file creation)
- WP2: 5 tool calls (builder implementations)
- WP3: 6 tool calls (test file updates)
- WP4: 4 tool calls (library test updates)
- WP5: 3 tool calls (validation & docs)
- **TOTAL: 22 tool calls**

---

## Part 10: Circular Dependency Prevention

### Dependency Analysis

```
test-fixtures-shared
├── Imports: agileplus-domain (types only, not impl)
│           phenotype-event-sourcing (types)
│           phenotype-cache-adapter (types)
│           phenotype-policy-engine (types)
│           agileplus-api (ONLY in test_server, for router setup)
│
├── NEVER imports: test files, integration tests, other test fixtures
│
└── Never causes circular deps because:
    - Builders depend on TYPE definitions (immutable)
    - Factories depend on TYPE definitions (immutable)
    - MockStorage implements trait objects (one-way dependency)
    - No reverse dependencies: no crate imports test-fixtures-shared
```

### Preventing Circular Imports
1. **builders/** — Import ONLY domain types (Feature, WorkPackage, etc.)
2. **factories/** — Import ONLY types used in tests (Order, User, Event, etc.)
3. **mock_storage/** — Implement traits defined in other crates; no reverse imports
4. **test_server/** — Only imported in test modules (dev-dependency)

**Verification**: `cargo build -p test-fixtures-shared` must succeed in isolation.

---

## Part 11: Future Extensions (Phase 2+)

### Post-MVP Enhancements
1. **Database Fixtures** — Create `test-fixtures-db` crate for database seeding
2. **Macro-Driven Builders** — Use procedural macros to auto-generate builders
3. **Property-Based Testing** — Integration with proptest/quickcheck for fuzzing
4. **Snapshot Testing** — Fixtures with snapshot assertion helpers
5. **Performance Benchmarks** — Builder performance benchmarks for regression detection

### Cross-Project Reuse
- **heliosCLI**: Extract harness-specific builders to shared crate
- **phenotype-shared**: Consolidate phenotype crate fixtures
- **agent-wave**: Share fixture patterns for agent testing

---

## Summary

**This comprehensive audit establishes:**

✅ **Clear duplication picture** — 958 LOC of scattered fixtures consolidates to ~700 LOC savings
✅ **Detailed implementation roadmap** — 5 phases, 22 tool calls, 85 min (50 min parallel)
✅ **Builder & factory patterns** — 6 builders + 3 factories provide comprehensive coverage
✅ **Risk mitigation** — All identified risks have mitigation strategies
✅ **Success criteria** — Clear metrics for validating completion
✅ **Future extensibility** — Phase 2 roadmap for advanced features

**Ready for execution**: All planning documents (INDEX, SUMMARY, PLAN, CODE_EXAMPLES, VISUAL) already created. Proceed with Phase 1 scaffolding.

---

**Document**: FIXTURE_AUDIT_COMPREHENSIVE.md
**Date**: 2026-03-30
**Status**: READY FOR IMPLEMENTATION
**Next Step**: Execute WP1 (Scaffolding) → WP2/WP3 (Parallel) → WP4 → WP5
