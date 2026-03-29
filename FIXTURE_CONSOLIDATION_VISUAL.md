# Test Fixture Consolidation - Visual Architecture

## Current State: Scattered Fixtures

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      BEFORE: Scattered Fixtures                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─ AgilePlus API Tests ───────────────────────────────────────────────┐    │
│  │                                                                      │    │
│  │  support/storage.rs (129 LOC)                                       │    │
│  │  ├─ MockStorage { features, work_packages, ... }                   │    │
│  │  └─ with_test_data() — hardcoded Feature + WP creation            │    │
│  │                                                                      │    │
│  │  support/mod.rs (41 LOC)                                            │    │
│  │  ├─ InMemoryCredentialStore setup (5 lines)                        │    │
│  │  ├─ AppConfig setup (3 lines)                                       │    │
│  │  └─ TestServer initialization (8 lines)                             │    │
│  │                                                                      │    │
│  │  storage_port_impl/*.rs (1,818 LOC) ◄─ HEAVY DUPLICATION          │    │
│  │  ├─ feature.rs (108 LOC) — CRUD + state queries                    │    │
│  │  ├─ work_package.rs (247 LOC) — CRUD + state queries               │    │
│  │  ├─ cycle.rs (273 LOC) — CRUD + state queries                      │    │
│  │  └─ [9 more files...] — similar patterns repeated                  │    │
│  │                                                                      │    │
│  │  *.rs test files (features_work_packages.rs, module_cycle.rs, ...)  │    │
│  │  └─ Each contains: hardcoded Feature/WP structs + server setup     │    │
│  │                                                                      │    │
│  └──────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─ AgilePlus Dashboard ───────────────────────────────────────────────┐    │
│  │                                                                      │    │
│  │  seed.rs (200+ LOC)                                                 │    │
│  │  ├─ make_shipped_feature() — hardcoded fixture builder              │    │
│  │  ├─ make_shipped_wps() — hardcoded WP builder                       │    │
│  │  └─ seed_dogfood_features() — 37 features manually created         │    │
│  │                                                                      │    │
│  │  tests/seed_integration.rs (100+ LOC)                               │    │
│  │  └─ Duplicates fixtures from seed.rs                                │    │
│  │                                                                      │    │
│  └──────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─ AgilePlus Integration Tests ──────────────────────────────────────┐    │
│  │                                                                      │    │
│  │  common/fixtures.rs (146 LOC)                                       │    │
│  │  ├─ TestFixtures struct with hardcoded features                    │    │
│  │  ├─ feature_create_payload() — hardcoded payload builder            │    │
│  │  ├─ transition_payload() — hardcoded transition builder             │    │
│  │  └─ plane_webhook_payload() — hardcoded webhook builder             │    │
│  │                                                                      │    │
│  └──────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  ┌─ Consolidated Libraries Tests ─────────────────────────────────────┐    │
│  │                                                                      │    │
│  │  event_store.rs (463 LOC)                                           │    │
│  │  ├─ Order, User structs (12 lines)                                  │    │
│  │  ├─ create_order_event() (10 lines)                                 │    │
│  │  ├─ create_user_event() (10 lines)                                  │    │
│  │  └─ [50+ test functions with similar patterns]                      │    │
│  │                                                                      │    │
│  │  cache_adapter.rs (416 LOC)                                         │    │
│  │  ├─ L1/L2 cache initialization repeated 20+ times                   │    │
│  │  └─ json!() payloads repeated throughout                            │    │
│  │                                                                      │    │
│  │  policy_engine.rs (143 LOC) — hardcoded policy fixtures             │    │
│  │  state_machine.rs (151 LOC) — hardcoded state fixtures              │    │
│  │                                                                      │    │
│  └──────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
│  PAIN POINTS:                                                                │
│  ❌ Duplication across 15+ files                                            │
│  ❌ Hard to maintain consistent test data                                   │
│  ❌ Changes to domain models require updating 10+ test files                │
│  ❌ New tests copy-paste boilerplate (error-prone)                          │
│  ❌ Inconsistent fixture patterns (some use builders, some hardcoded)      │
│  ❌ Test setup logic harder to read (buried in boilerplate)                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## After State: Centralized Fixtures

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    AFTER: Shared test-fixtures-shared                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│                                                                              │
│                   ┌───────────────────────────────────┐                     │
│                   │  crates/test-fixtures-shared      │                     │
│                   │  (Single source of truth)         │                     │
│                   └───────────────────────────────────┘                     │
│                                  │                                           │
│                    ┌─────────────┼─────────────┐                            │
│                    │             │             │                            │
│           ┌────────▼──────┐  ┌───▼─────────┐  │                            │
│           │   Builders    │  │  Factories  │  │                            │
│           └───────────────┘  └─────────────┘  │                            │
│           ┌─────────────────────────────────┐ │                            │
│           │                                 │ │                            │
│           │ • FeatureFixture (60 LOC)      │ │                            │
│           │   .new("slug", "name")         │ │                            │
│           │   .id(1)                       │ │                            │
│           │   .state(FeatureState::X)      │ │                            │
│           │   .with_shipped()              │ │                            │
│           │   .build()                     │ │                            │
│           │                                 │ │                            │
│           │ • WorkPackageFixture (50 LOC)  │ │                            │
│           │   .new(feature_id, "title")   │ │                            │
│           │   .state(WpState::Done)        │ │                            │
│           │   .with_pr("https://...")      │ │                            │
│           │   .build()                     │ │                            │
│           │                                 │ │                            │
│           │ • AuditChainFixture (40 LOC)   │ │                            │
│           │   .genesis(feature_id)         │ │                            │
│           │   .with_entry(..., "specified")│ │                            │
│           │   .build()                     │ │                            │
│           │                                 │ │                            │
│           │ • ProjectFixture (40 LOC)      │ │                            │
│           │ • CycleFixture (40 LOC)        │ │                            │
│           │ • ModuleFixture (40 LOC)       │ │                            │
│           │                                 │ │                            │
│           └─────────────────────────────────┘ │                            │
│                                               │                            │
│           ┌─────────────────────────────────┐ │                            │
│           │ EventFactory (30 LOC)           │ │                            │
│           │ CacheValueFactory (25 LOC)      │ │                            │
│           │ PolicyFactory (20 LOC)          │ │                            │
│           └─────────────────────────────────┘ │                            │
│                                               │                            │
│                                  ┌────────────▼─────────┐                  │
│                                  │  Mock Storage & Test  │                  │
│                                  │  Server Fixtures     │                  │
│                                  └──────────────────────┘                  │
│                                  ┌──────────────────────┐                  │
│                                  │ MockStorage (70 LOC) │                  │
│                                  │ • features Vec       │                  │
│                                  │ • work_packages Vec  │                  │
│                                  │ • [5 more...]        │                  │
│                                  │ • with_test_data()   │                  │
│                                  └──────────────────────┘                  │
│                                  ┌──────────────────────┐                  │
│                                  │ TestServerFixture    │                  │
│                                  │ (50 LOC)             │                  │
│                                  │ • new()              │                  │
│                                  │ • with_storage()     │                  │
│                                  │ • test_api_key()     │                  │
│                                  └──────────────────────┘                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                        ▲
                                        │ imports
                    ┌───────────────────┼───────────────────┐
                    │                   │                   │
        ┌───────────▼──────┐  ┌────────▼──────┐  ┌─────────▼──────┐
        │  AgilePlus API   │  │  Consolidated │  │ Integration &  │
        │  Tests           │  │  Libraries     │  │ Dashboard      │
        │                  │  │  Tests         │  │ Tests          │
        │ • support/*.rs   │  │                │  │                │
        │ • api_*_tests.rs │  │ • event_store  │  │ • fixtures.rs  │
        │   (now 2x LOC)   │  │   .rs (now 1x) │  │ • seed_int.rs  │
        │                  │  │ • cache_*.rs   │  │ (now 1x LOC)   │
        │  BEFORE: 1,800   │  │ • policy_*.rs  │  │                │
        │  AFTER: 600      │  │ • state_*.rs   │  │ BEFORE: 400    │
        │  SAVINGS: 1,200  │  │                │  │ AFTER: 150     │
        │                  │  │ BEFORE: 1,173  │  │ SAVINGS: 250   │
        │                  │  │ AFTER: 600     │  │                │
        │                  │  │ SAVINGS: 573   │  │                │
        └──────────────────┘  └────────────────┘  └────────────────┘

BENEFITS:
✓ Single source of truth for all fixtures
✓ Consistent builder patterns across all tests
✓ New test files = 3 lines of imports, focused test logic
✓ Domain model changes = 1 builder file update, all tests fixed
✓ ~1,200+ LOC removed from API tests
✓ ~573 LOC removed from library tests
✓ ~250 LOC removed from integration/dashboard tests
✓ TOTAL SAVINGS: ~650-700 LOC eliminated
```

## Migration Flow

```
┌──────────────────────────────────────────────────────────────────┐
│                  MIGRATION WORKFLOW                               │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  STEP 1: Create test-fixtures-shared crate                       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ $ cargo new --lib crates/test-fixtures-shared            │   │
│  │ $ cd crates/test-fixtures-shared                         │   │
│  │ $ mkdir -p src/{builders,factories,mock_storage,test_*}  │   │
│  │                                                            │   │
│  │ Result: New crate with modules ready for implementation  │   │
│  └──────────────────────────────────────────────────────────┘   │
│                           ▼                                      │
│  STEP 2: Implement builders & factories in test-fixtures        │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ • FeatureFixture { builder code 60 LOC }                │   │
│  │ • WorkPackageFixture { builder code 50 LOC }             │   │
│  │ • AuditChainFixture { builder code 40 LOC }              │   │
│  │ • MockStorage { implementation 70 LOC }                  │   │
│  │ • TestServerFixture { implementation 50 LOC }            │   │
│  │ • Factories for events, cache, policies (75 LOC)         │   │
│  │                                                            │   │
│  │ Subtotal: ~425 LOC new shared code                        │   │
│  │ Includes tests for each builder                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                           ▼                                      │
│  STEP 3: Update AgilePlus test files (6 updates)                │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ support/storage.rs:       (129 LOC) → use MockStorage    │   │
│  │ support/mod.rs:           (41 LOC)  → use TestServerFix  │   │
│  │ storage_port_impl/:       (1,818)   → use builders       │   │
│  │ features_work_packages.rs: → use FeatureFixture builder  │   │
│  │ module_cycle.rs:          → use builders                 │   │
│  │ dashboard/seed.rs:        → use factories                │   │
│  │                                                            │   │
│  │ Removes ~1,200 LOC of boilerplate                         │   │
│  └──────────────────────────────────────────────────────────┘   │
│                           ▼                                      │
│  STEP 4: Update consolidated library tests (4 updates)          │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ event_store.rs:   (463 LOC) → use EventFactory           │   │
│  │ cache_adapter.rs: (416 LOC) → use CacheValueFactory      │   │
│  │ policy_engine.rs: (143 LOC) → use PolicyFactory          │   │
│  │ state_machine.rs: (151 LOC) → use builders               │   │
│  │                                                            │   │
│  │ Removes ~573 LOC of test data boilerplate                 │   │
│  └──────────────────────────────────────────────────────────┘   │
│                           ▼                                      │
│  STEP 5: Validation & documentation                             │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ $ cargo test --all        ← All tests pass (green)        │   │
│  │ $ cargo clippy --all      ← No warnings                   │   │
│  │ $ cargo fmt -- --check    ← Formatting OK                 │   │
│  │                                                            │   │
│  │ Document builder patterns with examples                   │   │
│  │ Create migration guide for new tests                      │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  FINAL STATE:                                                    │
│  • test-fixtures-shared = single source of truth                │
│  • 15+ test files now use consistent fixture patterns           │
│  • ~650 LOC of duplication eliminated                           │
│  • All tests pass, no regressions                               │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

## Dependency Graph

```
                    ┌─ test-fixtures-shared ─┐
                    │   (Workspace member)    │
                    │                         │
                    │ • builders/             │
                    │ • factories/            │
                    │ • mock_storage/         │
                    │ • test_server/          │
                    │ • seeds/                │
                    │                         │
                    └────────────┬────────────┘
                                 │
                                 │ depends_on
                    ┌────────────┴────────────┐
                    │                         │
         ┌──────────▼──────────┐  ┌───────────▼────────┐
         │   agileplus-api     │  │ consolidated-libs  │
         │    tests/           │  │   tests/           │
         │                     │  │                    │
         │ support/*.rs   ◄────┼──┤ event_store.rs     │
         │ api_int_tests  ◄────┼──┤ cache_adapter.rs   │
         │ features.rs    ◄────┘  │ policy_engine.rs   │
         │                         │ state_machine.rs   │
         └─────────────────────────┘                    │
                                    └────────────────────┘

         ┌──────────────────────────────┐
         │   agileplus-dashboard &       │
         │   agileplus-integration       │
         │                              │
         │ seed.rs              ◄───────┤ uses builders/factories
         │ seed_integration.rs  ◄────────┤
         │ fixtures.rs          ◄────────┤
         └──────────────────────────────┘

KEY: All test files now share test-fixtures-shared
     No more peer-to-peer fixture duplication
```

## Size Comparison

```
BEFORE:
┌─────────────────────────────────────────────────────────┐
│  Total Fixture Code: ~1,800 LOC (scattered)             │
│                                                          │
│  ████████████████████████████████████████████ 1,800 LOC │
└─────────────────────────────────────────────────────────┘

AFTER:
┌─────────────────────────────────────────────────────────┐
│  Shared Fixtures: ~425 LOC (centralized)                │
│                                                          │
│  ███████████ 425 LOC (shared by all)                    │
│                                                          │
│  Test File Usage: ~3 lines each (import + usage)        │
│                                                          │
│  ███ 3-5 LOC per test file (was 100-300 LOC)           │
└─────────────────────────────────────────────────────────┘

SAVINGS:
┌─────────────────────────────────────────────────────────┐
│  Eliminated Duplication: 1,800 - 425 = 1,375 LOC       │
│                                                          │
│  Documented Target: ~650 LOC                            │
│  (Conservative estimate; actual may exceed)             │
│                                                          │
│  ████████████████ 650+ LOC REMOVED                      │
└─────────────────────────────────────────────────────────┘
```

---

**Architecture Transition Complete**: This visualization shows the transformation from scattered, duplicated test fixtures to a centralized, reusable shared crate.
