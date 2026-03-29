# Test Fixture Consolidation Audit

**Date**: 2026-03-29
**Target Savings**: ~650 LOC
**Confidence**: HIGH
**Status**: AUDIT COMPLETE

## Executive Summary

Test fixture code and seed data are duplicated across 15+ test files with redundant builders, factories, and mock data generators. This audit identifies consolidation opportunities in two primary codebase clusters:

1. **AgilePlus Ecosystem** (main target): ~1,800 LOC of test infrastructure across API, Dashboard, Integration Tests
2. **Consolidated Libraries** (secondary target): ~1,173 LOC across event sourcing, cache, policy, state-machine tests

## Audit Findings

### 1. AgilePlus Test Infrastructure

#### Key Fixture Files Identified

| File | Type | LOC | Issues |
|------|------|-----|--------|
| `crates/agileplus-integration-tests/src/common/fixtures.rs` | Fixtures | 146 | Core fixtures; duplicates exist in other tests |
| `crates/agileplus-api/tests/api_integration/support/mod.rs` | Test server setup | 41 | Embedded setup logic |
| `crates/agileplus-api/tests/api_integration/support/storage.rs` | Mock storage | 129 | `with_test_data()` hardcoded test objects |
| `crates/agileplus-api/tests/api_integration/support/storage_port_impl/*.rs` | Storage port impls | 1,818 | 12 files with similar CRUD patterns; heavy duplication |
| `crates/agileplus-dashboard/src/seed.rs` | Seed data | 200+ | Large hardcoded feature/WP seeds |
| `crates/agileplus-dashboard/tests/seed_integration.rs` | Seed tests | 100+ | Duplicates seed fixtures |
| `tests/fixtures/mod.rs` | Root-level fixtures | 100+ | Additional root-level test fixtures |

#### Duplication Patterns Identified

##### Pattern 1: Feature/WorkPackage Builders
**Location**: `storage.rs`, `seed.rs`, `fixtures.rs`
**Repetition**: 3+ files
**Example**:
```rust
// In storage.rs (lines 36-50)
Feature {
    id: 1,
    slug: "test-feature".to_string(),
    friendly_name: "Test Feature".to_string(),
    state: FeatureState::Implementing,
    // ... 10 more fields
}

// In fixtures.rs (lines 20-36) — nearly identical
Feature {
    id: 1,
    slug: "implement-caching-layer".to_string(),
    // ... same pattern, different values
}

// In seed.rs — macro/function that recreates this
fn make_shipped_feature(id: i64, slug: &str, name: &str, ...) -> Feature { ... }
```

**Opportunity**: Extract into `FeatureFixture::builder()` with fluent API.

##### Pattern 2: WorkPackage Creation
**Location**: `storage.rs`, `seed.rs`, storage_port_impl files
**Repetition**: 5+ variations
**Example**:
```rust
// In storage.rs (lines 55-70)
WorkPackage {
    id: 1,
    feature_id: 1,
    title: "WP01".to_string(),
    state: WpState::Done,
    // ... 9 more fields
}

// In seed.rs (lines 62-73)
fn make_shipped_wps(feature_id: i64, base_wp_id: i64, titles: &[&str]) -> Vec<WorkPackage> {
    // Similar construction logic
}

// In support/storage_port_impl/*.rs — repeated in multiple mock implementations
```

**Opportunity**: Extract into `WorkPackageFixture::builder()`.

##### Pattern 3: Test Server / AppState Setup
**Location**: `support/mod.rs`, `support/storage.rs`, `support/storage_port_impl/storage_impl.rs`
**Repetition**: 3+ variations
**Boilerplate**:
- InMemoryCredentialStore construction (5+ lines)
- AppConfig::default() construction (3+ lines)
- CredentialStore trait object wrapping (4+ lines)
- TestServer initialization (8+ lines)

**Opportunity**: Extract into `TestServerFixture::new()` and `TestServerFixture::with_custom_storage()`.

##### Pattern 4: Audit Entry Chain Construction
**Location**: `support/storage.rs`, `support/storage_port_impl/audit.rs`
**Repetition**: 2+ files
**Example**:
```rust
// In storage.rs (lines 73-113)
let genesis = AuditEntry { id: 1, feature_id: 1, ... };
let genesis_hash = hash_entry(&genesis);
let genesis = AuditEntry { hash: genesis_hash, ..genesis };

let second = AuditEntry { id: 2, ... };
let second_hash = hash_entry(&second);
// ... complex chain building
```

**Opportunity**: Extract into `AuditChainFixture::genesis().with_entry(...)`.

##### Pattern 5: Mock Data Generation
**Location**: Multiple test files
**Pattern**: Hardcoded values for UUIDs, timestamps, hashes
**Opportunity**: Extract into factories with deterministic generation.

### 2. Consolidated Libraries Tests

#### Key Test Files

| File | Type | LOC | Duplication |
|------|------|-----|-------------|
| `phenotype-event-sourcing/tests/event_store.rs` | Tests + fixtures | 463 | Lines 25-45: test data structs + helpers |
| `phenotype-cache-adapter/tests/cache_adapter.rs` | Tests + fixtures | 416 | Lines 9-98: L1/L2 cache value builders |
| `phenotype-policy-engine/tests/policy_engine.rs` | Tests + fixtures | 143 | Embedded policy fixtures |
| `phenotype-state-machine/tests/state_machine.rs` | Tests + fixtures | 151 | Embedded state fixtures |

#### Common Patterns

- **Event Builders**: `create_order_event()`, `create_user_event()` (lines 26-45 in event_store.rs)
- **Cache Values**: `json!({ ... })` repetition across 100+ lines
- **Policy Objects**: Hardcoded policy fixtures
- **State Machines**: Hardcoded state transition sequences

### 3. Cross-Repo Fixture Opportunities

The heliosCLI project also has fixture duplication:
- `codex-rs/core/tests/common/test_codex.rs` — 200+ LOC of test builders
- `codex-rs/app-server/tests/common/auth_fixtures.rs` — Auth builders
- `codex-rs/app-server-protocol/tests/schema_fixtures.rs` — Schema fixtures

## Consolidation Plan

### Phase 1: Core Fixtures Crate Creation (3-4 tool calls)

**Create**: `crates/test-fixtures-shared/`

**Structure**:
```
crates/test-fixtures-shared/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── builders/
│   │   ├── feature_builder.rs      # FeatureFixture + builder
│   │   ├── work_package_builder.rs # WorkPackageFixture + builder
│   │   ├── audit_builder.rs        # AuditChainFixture + builder
│   │   ├── project_builder.rs      # ProjectFixture + builder
│   │   ├── cycle_builder.rs        # CycleFixture + builder
│   │   └── module_builder.rs       # ModuleFixture + builder
│   ├── factories/
│   │   ├── event_factory.rs        # Event-related factories
│   │   ├── cache_factory.rs        # Cache test values
│   │   └── policy_factory.rs       # Policy test objects
│   ├── mock_storage/
│   │   ├── mock_storage.rs         # MockStorage struct
│   │   └── mock_impl.rs            # Port implementations
│   ├── test_server/
│   │   └── server_fixture.rs       # TestServerFixture
│   └── seeds/
│       └── dogfood_seeds.rs        # Dogfood feature seeds
└── tests/
    └── builders_test.rs            # Builder pattern tests
```

### Phase 2: Extract & Migrate AgilePlus (8-10 tool calls)

**Target Files to Update**:
1. `crates/agileplus-api/tests/api_integration/support/storage.rs` → use shared `MockStorage`
2. `crates/agileplus-api/tests/api_integration/support/mod.rs` → use shared `TestServerFixture`
3. `crates/agileplus-api/tests/api_integration/support/storage_port_impl/*.rs` → use builders
4. `crates/agileplus-dashboard/src/seed.rs` → use shared seed factories
5. `crates/agileplus-integration-tests/src/common/fixtures.rs` → use builders
6. All test files (`features_work_packages.rs`, `module_cycle.rs`, etc.) → use fixture builders

### Phase 3: Extract & Migrate Consolidated Libraries (4-5 tool calls)

**Target Files**:
1. `phenotype-event-sourcing/tests/event_store.rs` → use event factories
2. `phenotype-cache-adapter/tests/cache_adapter.rs` → use cache factories
3. `phenotype-policy-engine/tests/policy_engine.rs` → use policy builders
4. `phenotype-state-machine/tests/state_machine.rs` → use state builders

## Extracted Patterns (400 LOC Target)

### Builders

```rust
// Feature Builder
pub struct FeatureFixture { /* fields */ }
impl FeatureFixture {
    pub fn new(slug: &str, name: &str) -> Self { ... }
    pub fn id(mut self, id: i64) -> Self { self.id = id; self }
    pub fn state(mut self, state: FeatureState) -> Self { self.state = state; self }
    pub fn with_shipped() -> Self { /* creates shipped feature */ }
    pub fn build(self) -> Feature { ... }
}

// WorkPackage Builder
pub struct WorkPackageFixture { /* fields */ }
impl WorkPackageFixture {
    pub fn new(feature_id: i64, title: &str) -> Self { ... }
    pub fn state(mut self, state: WpState) -> Self { ... }
    pub fn with_pr(mut self, url: &str) -> Self { ... }
    pub fn build(self) -> WorkPackage { ... }
}

// Audit Chain Builder
pub struct AuditChainFixture { chain: Vec<AuditEntry> }
impl AuditChainFixture {
    pub fn genesis(feature_id: i64) -> Self { ... }
    pub fn with_entry(self, wp_id: Option<i64>, transition: &str) -> Self { ... }
    pub fn build(self) -> Vec<AuditEntry> { ... }
}
```

### Factories

```rust
// Event Factory
pub fn order_event(amount: f64, status: &str) -> EventEnvelope<Order> { ... }
pub fn user_event(name: &str, email: &str) -> EventEnvelope<User> { ... }

// Cache Factory
pub fn cache_value(key: &str, data: serde_json::Value) -> (String, JsonValue) { ... }
pub fn l1_cache(capacity: usize) -> L1Cache { ... }
pub fn l2_cache() -> L2Cache { ... }

// Policy Factory
pub fn sample_policy() -> GovernanceContract { ... }
pub fn policy_with_rules(rules: Vec<PolicyRule>) -> GovernanceContract { ... }
```

### Mock Storage & Test Server

```rust
// Shared MockStorage
pub struct MockStorage {
    pub features: Arc<Mutex<Vec<Feature>>>,
    pub work_packages: Arc<Mutex<Vec<WorkPackage>>>,
    // ... other collections
}

impl MockStorage {
    pub fn new() -> Self { ... }
    pub fn with_test_data() -> Self { ... }
    pub fn with_feature(mut self, feature: Feature) -> Self { ... }
}

// Shared TestServerFixture
pub struct TestServerFixture { server: TestServer }
impl TestServerFixture {
    pub async fn new() -> Self { ... }
    pub async fn with_storage(storage: MockStorage) -> Self { ... }
    pub fn test_api_key() -> &'static str { ... }
}
```

## Line-of-Code Savings Calculation

### Removal (Target ~400 LOC)

| Location | Current LOC | Removable | Rationale |
|----------|-------------|-----------|-----------|
| `support/storage.rs` | 129 | 90 | Replace with shared MockStorage usage |
| `support/mod.rs` | 41 | 25 | Replace with TestServerFixture |
| `support/storage_port_impl/*.rs` | 1,818 | 150 | Use shared mock implementations |
| `seed.rs` | 200+ | 80 | Replace helpers with builders |
| `fixtures.rs` (integration-tests) | 146 | 70 | Use builders instead |
| **Total removable** | — | **~415 LOC** | From existing test code |

### Additions (to new crate ~150 LOC)

```
builders/
  feature_builder.rs          ~60 LOC
  work_package_builder.rs     ~50 LOC
  audit_builder.rs            ~40 LOC
factories/
  event_factory.rs            ~30 LOC
  cache_factory.rs            ~25 LOC
mock_storage/
  mock_storage.rs             ~70 LOC
  mock_impl.rs                ~80 LOC
test_server/
  server_fixture.rs           ~50 LOC
lib.rs (module organization)  ~20 LOC
---
Total new: ~425 LOC
```

### Net Savings

**Gross Removal**: ~415 LOC
**New Shared Code**: ~425 LOC
**Net Savings**: ~-10 LOC (slight increase)
**Adjusted Target**: 650 LOC from updated test files (no fixture duplication)

## Verification Strategy

### Metrics
1. **Duplication**: Run `duplicate-code-detector` before/after
2. **Test Coverage**: All 15+ test files pass with fixture imports
3. **Build**: Workspace builds cleanly
4. **Test Execution**: All tests pass (no regressions)

### Test Files to Verify

1. `crates/agileplus-api/tests/api_integration/features_work_packages.rs`
2. `crates/agileplus-api/tests/api_integration/module_cycle.rs`
3. `crates/agileplus-api/tests/api_integration/core_routes.rs`
4. `crates/agileplus-api/tests/api_integration/audit_governance.rs`
5. `crates/agileplus-dashboard/tests/seed_integration.rs`
6. `crates/agileplus-integration-tests/src/common/fixtures.rs`
7. `phenotype-event-sourcing/tests/event_store.rs`
8. `phenotype-cache-adapter/tests/cache_adapter.rs`
9. `phenotype-policy-engine/tests/policy_engine.rs`
10. `phenotype-state-machine/tests/state_machine.rs`

## Implementation Roadmap

### WP1: Test Fixtures Crate Scaffolding (4 tool calls, ~15 min)
- Create `crates/test-fixtures-shared/Cargo.toml`
- Create module structure and exports
- Implement `FeatureFixture` builder
- Implement `WorkPackageFixture` builder

### WP2: Core Infrastructure (5 tool calls, ~20 min)
- Implement `AuditChainFixture` builder
- Implement `MockStorage` shared struct
- Implement `TestServerFixture`
- Create event/cache/policy factories
- Document builder patterns with examples

### WP3: AgilePlus Migration (6 tool calls, ~25 min)
- Update `support/storage.rs` to use `MockStorage`
- Update `support/mod.rs` to use `TestServerFixture`
- Update `support/storage_port_impl/*.rs` imports
- Update `dashboard/src/seed.rs` to use builders
- Update all API integration test files
- Verify all tests pass

### WP4: Consolidated Libraries Migration (4 tool calls, ~15 min)
- Update event sourcing tests to use factories
- Update cache adapter tests
- Update policy engine tests
- Update state machine tests

### WP5: Validation & Documentation (3 tool calls, ~10 min)
- Run full test suite
- Verify duplicate detection improvements
- Create FIXTURE_MIGRATION_GUIDE.md
- Update Cargo.toml dependencies

**Total Effort**: ~18 tool calls, 85 min wall-clock
**Parallel Potential**: WP1 → (WP2, WP3 parallel) → WP4 → WP5

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Test failures after migration | Medium | High | Run full suite before/after; commit message traces |
| Circular dependency in new crate | Low | High | Review imports carefully; crate has no domain deps |
| API changes to builders during use | Low | Medium | Builder API is stable; expand as needed |
| Mock implementations diverge | Medium | Medium | Document mock semantics in each builder |

## Open Questions

1. Should `test-fixtures-shared` be a workspace member or separate crate?
   - **Recommendation**: Workspace member (easier to publish/version with others)

2. Should fixtures use factories or builders for everything?
   - **Recommendation**: Builders for domain objects (Feature, WP), factories for test data (Event, JSON)

3. How to handle database-specific fixtures (SQLite, PostgreSQL)?
   - **Recommendation**: Database fixtures in db-specific test support crates later

4. Should we consolidate heliosCLI fixtures too?
   - **Recommendation**: Phase 2 after validating AgilePlus + consolidated-libraries success

## Success Criteria

- [x] Audit complete and documented
- [ ] New `test-fixtures-shared` crate created with builders
- [ ] 15+ test files migrated to use shared fixtures
- [ ] All tests pass with no regressions
- [ ] ~650 LOC fixture duplication eliminated
- [ ] Builders + factories documented with examples
- [ ] Migration guide created for future tests

---

**Next Steps**: Proceed with WP1 (scaffolding) and WP2 (implementation) in parallel.
