# Fixture Consolidation — Migration Sequence & Dependency Analysis

**Purpose**: Step-by-step migration plan with detailed file-by-file updates
**Audience**: Implementers executing Phases 2-5
**Status**: DETAILED ROADMAP FOR EXECUTION

---

## Part 1: Dependency Graph & Execution Order

### 1.1 Critical Path Analysis

```
WP1: Scaffolding (BLOCKING)
│
├─→ Create crate structure
├─→ Update Cargo.toml (workspace)
├─→ cargo build (verify)
│
└──→ WP2 & WP3 can begin in parallel
    │
    ├──→ WP2: Core Infrastructure (20 min)
    │   ├─→ Implement builders (6)
    │   ├─→ Implement factories (3)
    │   ├─→ Implement mocks (2)
    │   └─→ Test builders
    │
    ├──→ WP3: AgilePlus Migration (25 min) [CAN RUN IN PARALLEL WITH WP2]
    │   ├─→ Update support/storage.rs
    │   ├─→ Update support/mod.rs
    │   ├─→ Update test files (15+)
    │   └─→ Test agileplus crates
    │
    └──→ WP2 completion gates WP4
        │
        ├──→ WP4: Consolidated Libraries (15 min)
        │   ├─→ Update phenotype tests (4)
        │   └─→ Test phenotype crates
        │
        └──→ WP5: Validation (10 min)
            ├─→ Full test suite
            ├─→ Linting & formatting
            └─→ Documentation
```

### 1.2 Dependency Matrix

| Phase | Depends On | Blocking | Duration | Critical Path |
|-------|-----------|----------|----------|---|
| **WP1** | None | WP2, WP3 | 15 min | YES |
| **WP2** | WP1 | WP4 | 20 min | YES (indirect) |
| **WP3** | WP1 | WP5 | 25 min | YES |
| **WP4** | WP2 | WP5 | 15 min | NO |
| **WP5** | WP3, WP4 | None | 10 min | YES |

### 1.3 Parallel Execution Timeline

```
Time    Sequential Path          Parallel Path (Optimal)
────────────────────────────────────────────────────────
  0 min ┌─ WP1 (15 min)         ┌─ WP1 (15 min)
        │                       │
 15 min ├─ WP2 (20 min)         ├─ WP2 (20 min) ┐
        │                       │               ├─ Parallel
 35 min ├─ WP3 (25 min)         ├─ WP3 (25 min) ┘
        │                       │
 60 min ├─ WP4 (15 min)         ├─ WP4 (15 min)
        │                       │
 75 min ├─ WP5 (10 min)         ├─ WP5 (10 min)
        │                       │
 85 min └─ COMPLETE              └─ COMPLETE @ 50 min

Savings: 35 minutes (41%) by running WP2 + WP3 in parallel
```

---

## Part 2: WP1 — Scaffolding (15 min)

### 2.1 File Creation Checklist

- [ ] Create `crates/test-fixtures-shared/`
- [ ] Create `crates/test-fixtures-shared/Cargo.toml`
- [ ] Create `crates/test-fixtures-shared/src/lib.rs`
- [ ] Create module directories
- [ ] Update workspace `Cargo.toml`

### 2.2 Cargo.toml (Root Workspace)

**File**: `/Users/kooshapari/CodeProjects/Phenotype/repos/Cargo.toml`

```toml
[workspace]
members = [
    "crates/agileplus-domain",
    "crates/agileplus-api",
    "crates/agileplus-dashboard",
    "crates/test-fixtures-shared",              # ADD THIS
    "crates/agileplus-integration-tests",
    "crates/phenotype-errors",
    "crates/phenotype-error-core",
    # ... other members
]
```

### 2.3 Create Crate Structure

```bash
mkdir -p crates/test-fixtures-shared/src/{builders,factories,mock_storage,test_server,seeds}
touch crates/test-fixtures-shared/{Cargo.toml,src/lib.rs}
touch crates/test-fixtures-shared/src/builders/{mod.rs,feature_builder.rs,work_package_builder.rs,audit_builder.rs,project_builder.rs,cycle_builder.rs,module_builder.rs}
touch crates/test-fixtures-shared/src/factories/{mod.rs,event_factory.rs,cache_factory.rs,policy_factory.rs}
touch crates/test-fixtures-shared/src/mock_storage/{mod.rs,mock_storage.rs,mock_impl.rs}
touch crates/test-fixtures-shared/src/test_server/{mod.rs,server_fixture.rs}
touch crates/test-fixtures-shared/src/seeds/{mod.rs,dogfood_seeds.rs}
mkdir -p crates/test-fixtures-shared/tests
touch crates/test-fixtures-shared/tests/builders_test.rs
```

### 2.4 Crate Cargo.toml

**File**: `crates/test-fixtures-shared/Cargo.toml`

```toml
[package]
name = "test-fixtures-shared"
version = "0.1.0"
edition = "2021"

[dependencies]
agileplus-domain = { path = "../agileplus-domain" }
phenotype-event-sourcing = { path = "../phenotype-event-sourcing" }
phenotype-cache-adapter = { path = "../phenotype-cache-adapter" }
phenotype-policy-engine = { path = "../phenotype-policy-engine" }

chrono = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
tokio = { version = "1.0", features = ["full"], optional = true }

[dev-dependencies]
tokio = { version = "1.0", features = ["full"] }
axum-test = "14.0"

[features]
default = []
with-test-server = ["tokio"]
```

### 2.5 Root lib.rs

**File**: `crates/test-fixtures-shared/src/lib.rs`

```rust
//! Shared test fixtures for AgilePlus and Phenotype crates.
//!
//! Provides:
//! - Builders for domain objects (FeatureFixture, WorkPackageFixture, etc.)
//! - Factories for test data (EventFactory, CacheFactory, etc.)
//! - Mock implementations (MockStorage, TestServerFixture)
//! - Seed data for standard test scenarios

pub mod builders;
pub mod factories;
pub mod mock_storage;
pub mod seeds;

#[cfg(feature = "with-test-server")]
pub mod test_server;

// Re-export commonly used types
pub use builders::{
    FeatureFixture, WorkPackageFixture, AuditChainFixture, 
    ProjectFixture, CycleFixture, ModuleFixture,
};
pub use factories::{EventFactory, CacheFactory, PolicyFactory};
pub use mock_storage::MockStorage;

#[cfg(feature = "with-test-server")]
pub use test_server::TestServerFixture;

pub use seeds::DogfoodSeeds;
```

### 2.6 Verification

```bash
cd crates/test-fixtures-shared
cargo build
# Should succeed with message:
# Compiling test-fixtures-shared v0.1.0 (...)
#    Finished dev [unoptimized + debuginfo] target(s) in X.XXs
```

---

## Part 3: WP2 — Core Infrastructure (20 min)

### 3.1 Implementation Order

1. **builders/feature_builder.rs** — Foundation builder (5 min)
2. **builders/work_package_builder.rs** — Common builder (4 min)
3. **builders/audit_builder.rs** — Complex builder with validation (3 min)
4. **Project, Cycle, Module builders** — Similar pattern (4 min)
5. **mock_storage/mock_storage.rs** — Mock with builders (3 min)
6. **mock_storage/mock_impl.rs** — Trait implementations (2 min)
7. **factories/* and test_server/** — Shorter implementations (4 min)
8. **Write integration tests** — Verify all (2 min)

### 3.2 File-by-File Implementation

See FIXTURE_CONSOLIDATION_CODE_EXAMPLES.md for complete code.

### 3.3 Testing & Verification

```bash
# Build the crate
cd crates/test-fixtures-shared
cargo build

# Run all tests
cargo test

# Check for warnings
cargo clippy

# Verify it's importable
cargo test --lib --doc
```

---

## Part 4: WP3 — AgilePlus Migration (25 min)

### 4.1 Files to Update (In Order)

| File | Lines Changed | Type | Action |
|------|---------------|------|--------|
| `crates/agileplus-api/tests/support/storage.rs` | 127 | Refactor | Remove duplication, use builders |
| `crates/agileplus-api/tests/support/mod.rs` | 21 | Remove | Delete setup_test_server, use fixture |
| `crates/agileplus-api/tests/api_integration/support/storage_port_impl/feature.rs` | 45 | Update | Use builders |
| `crates/agileplus-api/tests/api_integration/support/storage_port_impl/work_package.rs` | 42 | Update | Use builders |
| `crates/agileplus-api/tests/api_integration/features_work_packages.rs` | 89 | Update | Use TestServerFixture |
| `crates/agileplus-api/tests/api_integration/module_cycle.rs` | 67 | Update | Use fixtures |
| `crates/agileplus-api/tests/api_integration/core_routes.rs` | 78 | Update | Use fixtures |
| `crates/agileplus-api/tests/api_integration/audit_governance.rs` | 95 | Update | Use AuditChainFixture |
| `crates/agileplus-dashboard/src/seed.rs` | 89 | Replace | Use DogfoodSeeds |
| `crates/agileplus-dashboard/tests/seed_integration.rs` | 45 | Update | Use DogfoodSeeds |
| `crates/agileplus-integration-tests/src/common/fixtures.rs` | 78 | Replace | Use shared fixture exports |

### 4.2 Migration Pattern Template

**BEFORE**:
```rust
// crates/agileplus-api/tests/support/mod.rs
use crate::support::storage::MockStorage;

pub(crate) async fn setup_test_server() -> TestServer {
    let storage = Arc::new(MockStorage::with_test_data());
    // ... 18 more lines of boilerplate
    TestServer::new(app)
}

#[tokio::test]
async fn test_example() {
    let server = setup_test_server().await;
    // ... test logic
}
```

**AFTER**:
```rust
// crates/agileplus-api/tests/support/mod.rs
use test_fixtures_shared::{TestServerFixture, MockStorage};

// Remove setup_test_server completely — use TestServerFixture instead

#[tokio::test]
async fn test_example() {
    let server = TestServerFixture::new().await;
    // ... test logic
}
```

### 4.3 Update Cargo.toml

**File**: `crates/agileplus-api/Cargo.toml` (dev-dependencies section)

```toml
[dev-dependencies]
test-fixtures-shared = { path = "../test-fixtures-shared", features = ["with-test-server"] }
agileplus-domain = { path = "../agileplus-domain" }
tokio = { version = "1", features = ["full"] }
axum-test = "14.0"
# ... other existing deps
```

### 4.4 Verification

```bash
# Build agileplus-api
cd crates/agileplus-api
cargo build

# Run all tests
cargo test

# Verify no clippy warnings
cargo clippy

# Check same-crate compilation
cargo check --tests
```

---

## Part 5: WP4 — Consolidated Libraries (15 min)

### 5.1 Files to Update

| Crate | Test File | Changes | Type |
|-------|-----------|---------|------|
| `phenotype-event-sourcing` | `tests/event_store.rs` | Remove event creation boilerplate | Refactor |
| `phenotype-cache-adapter` | `tests/cache_adapter.rs` | Use CacheFactory | Replace |
| `phenotype-policy-engine` | `tests/policy_engine.rs` | Use PolicyFactory | Replace |
| `phenotype-state-machine` | `tests/state_machine.rs` | Use state builders | Update |

### 5.2 Update Pattern

**BEFORE** (event_store.rs):
```rust
#[test]
fn test_order_events() {
    struct Order {
        id: String,
        amount: f64,
        status: String,
    }
    
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
    
    let event = create_order_event(100.0);
    // ... test logic
}
```

**AFTER**:
```rust
#[test]
fn test_order_events() {
    use test_fixtures_shared::EventFactory;
    
    let event = EventFactory::order_event(100.0, "pending");
    // ... test logic
}
```

### 5.3 Cargo.toml Updates

Add to [dev-dependencies]:
```toml
test-fixtures-shared = { path = "../../test-fixtures-shared" }
```

Note the relative path — these are in crates/ directory.

### 5.4 Verification

```bash
# Test all consolidated libraries in parallel
cargo test -p phenotype-event-sourcing &
cargo test -p phenotype-cache-adapter &
cargo test -p phenotype-policy-engine &
cargo test -p phenotype-state-machine &

# Wait for all to complete and verify
wait
```

---

## Part 6: WP5 — Validation & Documentation (10 min)

### 6.1 Full Test Suite

```bash
# Clean build
cargo clean
cargo build --all

# Full test suite
cargo test --all --verbose

# Check formatting
cargo fmt --all -- --check

# Lint everything
cargo clippy --all --all-targets
```

### 6.2 Specific Test Validations

```bash
# AgilePlus tests (most extensive)
cargo test -p agileplus-api --test '*'
cargo test -p agileplus-dashboard --test '*'
cargo test -p agileplus-integration-tests

# Phenotype tests (should all still pass)
cargo test -p phenotype-event-sourcing
cargo test -p phenotype-cache-adapter
cargo test -p phenotype-policy-engine
cargo test -p phenotype-state-machine

# Shared fixtures tests
cargo test -p test-fixtures-shared --lib
```

### 6.3 Create Migration Guide

**File**: `FIXTURE_MIGRATION_GUIDE.md`

```markdown
# Test Fixture Migration Guide

## Overview
This guide helps developers understand and use the new test-fixtures-shared crate.

## Quick Start

### Using FeatureFixture
```rust
use test_fixtures_shared::FeatureFixture;

let feature = FeatureFixture::new("my-feature", "My Feature")
    .id(1)
    .build();
```

### Using TestServerFixture
```rust
use test_fixtures_shared::TestServerFixture;

#[tokio::test]
async fn test_my_api() {
    let server = TestServerFixture::new().await;
    // ... make requests
}
```

### Using Factories
```rust
use test_fixtures_shared::EventFactory;

let event = EventFactory::order_event(100.0, "pending");
```

## Common Patterns

[See FIXTURE_CONSOLIDATION_CODE_EXAMPLES.md for full patterns]

## FAQs

**Q: What if I need a custom builder?**
A: Check if a builder method exists. If not, submit a PR to add it.

**Q: Can I use multiple builders together?**
A: Yes, compose them: create objects with builders, store in MockStorage, use in tests.

**Q: Where do I add new fixtures?**
A: Add to test-fixtures-shared crate, not in individual test files.
```

### 6.4 Documentation Updates

- [ ] Update this audit document with completion status
- [ ] Create migration guide
- [ ] Add examples to README.md
- [ ] Update CONTRIBUTING.md with fixture patterns

### 6.5 Commit & Push

```bash
git add -A
git commit -m "feat: consolidate test fixtures into shared crate

- Extract 700 LOC of duplication from 15+ test files
- Create test-fixtures-shared with 6 builders + 3 factories
- Implement MockStorage and TestServerFixture
- Update AgilePlus and phenotype tests to use shared fixtures
- All tests passing, 0 clippy warnings

Consolidates: FIXTURE_CONSOLIDATION_AUDIT, CODE_EXAMPLES, IMPLEMENTATION_PLAN"
git push origin main
```

---

## Part 7: Rollback Plan

### If Something Goes Wrong

#### Option A: Partial Rollback (Preserve Some Changes)
```bash
# Revert specific commits
git revert <commit-hash>

# Re-run WP1 + WP2 to ensure fixtures crate is clean
cd crates/test-fixtures-shared
cargo test
```

#### Option B: Full Rollback (Start Over)
```bash
# Remove the new crate
rm -rf crates/test-fixtures-shared

# Revert workspace Cargo.toml
git checkout Cargo.toml

# Revert all test file changes
git revert <first-migration-commit>..<last-migration-commit>
```

#### Option C: Partial Acceptance (Keep Core, Revert Migrations)
```bash
# Keep test-fixtures-shared and WP1/WP2
# But revert WP3 migrations

git revert <WP3-first-commit>..<WP3-last-commit>

# Test-fixtures-shared exists but unused (no harm, easy to complete later)
```

### Recovery Time
- Option A: 5-10 min (find specific commit, revert, test)
- Option B: 15-20 min (full cleanup and revert)
- Option C: 10-15 min (selective revert of migrations)

---

## Part 8: Success Criteria Checklist

### Must-Have (Blocking)
- [ ] `cargo test --all` passes 100%
- [ ] `cargo clippy --all` shows 0 warnings
- [ ] `cargo fmt --check` passes
- [ ] test-fixtures-shared crate builds cleanly
- [ ] All 15+ migrated test files pass

### Should-Have (High Priority)
- [ ] ~650 LOC of duplication eliminated
- [ ] Migration guide created
- [ ] All builders have 100% test coverage
- [ ] Documentation updated

### Nice-to-Have (Low Priority)
- [ ] Performance benchmarks added
- [ ] Integration with CI/CD automated
- [ ] Custom builder patterns documented

---

## Part 9: Metrics & Reporting

### Pre-Migration Baseline
```
Total LOC in test fixtures: 958 LOC
Files with duplication: 15+
Duplication instances: 66+
Code paths needing updates: 20+
```

### Post-Migration Target
```
Shared fixtures crate: ~250 LOC
Test file updates: 15+ files migrated
Elimination: ~650 LOC consolidated
New builders/factories: 9 total
Test coverage: 100% for builders
```

### Success Report Template
```markdown
# Fixture Consolidation Completion Report

**Date**: 2026-03-31
**Duration**: 50 minutes (parallel execution)
**Tool Calls**: 22 total

## Results
- ✅ 958 LOC of fixtures consolidated to 250 LOC
- ✅ 15+ test files migrated successfully
- ✅ 650 LOC of duplication eliminated
- ✅ 100% test coverage maintained
- ✅ All builders tested (100% coverage)
- ✅ 0 clippy warnings
- ✅ All formatting compliant

## Phases Completed
- ✅ WP1: Scaffolding (15 min)
- ✅ WP2: Core Infrastructure (20 min, parallel)
- ✅ WP3: AgilePlus Migration (25 min, parallel)
- ✅ WP4: Consolidated Libraries (15 min)
- ✅ WP5: Validation & Documentation (10 min)

## Metrics
| Metric | Value |
|--------|-------|
| LOC Consolidated | 650 |
| Test Files Migrated | 15+ |
| Builders Implemented | 6 |
| Factories Implemented | 3 |
| Mock Implementations | 2 |
| Test Coverage | 100% |
| Build Time | 45s (check), 120s (test) |

[See full results in git log]
```

---

## Summary

**This document provides:**

✅ **Detailed migration sequence** — File-by-file, tool-by-tool
✅ **Dependency analysis** — Parallel execution saves 35 minutes
✅ **Phase breakdown** — WP1 through WP5 with checklists
✅ **Before/after examples** — Every file type covered
✅ **Rollback procedures** — 3 options with recovery times
✅ **Success criteria** — Must-have, should-have, nice-to-have
✅ **Reporting templates** — How to document completion

**Ready for execution**: All supporting documents complete (CODE_EXAMPLES.md, TRAIT_SYSTEM.md, COMPREHENSIVE.md)

**Next Step**: Begin WP1 Scaffolding → Execute WP2/WP3 in parallel → Complete WP4/WP5

---

**Document**: FIXTURE_MIGRATION_SEQUENCE.md
**Status**: READY FOR EXECUTION (Implementation Phase)
**Last Updated**: 2026-03-30
