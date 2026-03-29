# Test Fixture Consolidation - Implementation Plan

**Status**: READY FOR IMPLEMENTATION
**Target**: Create shared `test-fixtures-shared` crate and migrate 15+ test files
**Effort**: 18 tool calls, ~85 minutes wall-clock
**Start Date**: 2026-03-29

## Phase Overview

```
WP1: Scaffolding
  ├─ Create crates/test-fixtures-shared/
  ├─ Module structure & Cargo.toml
  ├─ Basic re-exports
  └─ Duration: ~15 min (4 tool calls)

WP2: Core Infrastructure (parallel with WP3)
  ├─ Feature/WorkPackage builders
  ├─ Audit chain builder
  ├─ MockStorage & TestServerFixture
  ├─ Event/Cache/Policy factories
  └─ Duration: ~20 min (5 tool calls)

WP3: AgilePlus Migration (parallel with WP2)
  ├─ API integration tests
  ├─ Dashboard seed integration
  ├─ Support/storage modules
  └─ Duration: ~25 min (6 tool calls)

WP4: Consolidated Libraries
  ├─ Event sourcing tests
  ├─ Cache adapter tests
  ├─ Policy engine tests
  ├─ State machine tests
  └─ Duration: ~15 min (4 tool calls)

WP5: Validation & Documentation
  ├─ Full test suite verification
  ├─ Duplicate detection audit
  ├─ Migration guide
  └─ Duration: ~10 min (3 tool calls)
```

## WP1: Scaffolding (4 tool calls, ~15 min)

### 1.1 Create Cargo.toml for test-fixtures-shared

**File**: `/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/AgilePlus/phenotype-docs/crates/test-fixtures-shared/Cargo.toml`

**Content**:
```toml
[package]
name = "test-fixtures-shared"
version = "0.1.0"
edition = "2021"

[dependencies]
agileplus-domain = { path = "../agileplus-domain" }
agileplus-api = { path = "../agileplus-api" }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
axum-test = "14.0"

[dev-dependencies]
tokio-test = "0.4"
```

### 1.2 Create lib.rs with module organization

**File**: `/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/AgilePlus/phenotype-docs/crates/test-fixtures-shared/src/lib.rs`

**Content**:
```rust
//! Shared test fixtures for AgilePlus and consolidated libraries.
//!
//! Provides builders, factories, and mock implementations to reduce fixture
//! duplication across 15+ test files.
//!
//! # Examples
//!
//! ```ignore
//! use test_fixtures_shared::builders::FeatureFixture;
//! use test_fixtures_shared::builders::WorkPackageFixture;
//!
//! let feature = FeatureFixture::new("my-feature", "My Feature")
//!     .id(1)
//!     .with_shipped()
//!     .build();
//!
//! let wp = WorkPackageFixture::new(feature.id, "WP01")
//!     .state(WpState::Done)
//!     .with_pr("https://github.com/org/repo/pull/1")
//!     .build();
//! ```

pub mod builders;
pub mod factories;
pub mod mock_storage;
pub mod test_server;
pub mod seeds;

pub use builders::{FeatureFixture, WorkPackageFixture, AuditChainFixture, ProjectFixture, CycleFixture, ModuleFixture};
pub use factories::{EventFactory, CacheValueFactory, PolicyFactory};
pub use mock_storage::MockStorage;
pub use test_server::TestServerFixture;
pub use seeds::dogfood_features;
```

### 1.3 Create builders/mod.rs

**File**: `/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/AgilePlus/phenotype-docs/crates/test-fixtures-shared/src/builders/mod.rs`

**Content**:
```rust
//! Fluent builders for domain objects used in tests.

mod feature_builder;
mod work_package_builder;
mod audit_builder;
mod project_builder;
mod cycle_builder;
mod module_builder;

pub use feature_builder::FeatureFixture;
pub use work_package_builder::WorkPackageFixture;
pub use audit_builder::AuditChainFixture;
pub use project_builder::ProjectFixture;
pub use cycle_builder::CycleFixture;
pub use module_builder::ModuleFixture;
```

### 1.4 Create factories/mod.rs

**File**: `/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/AgilePlus/phenotype-docs/crates/test-fixtures-shared/src/factories/mod.rs`

**Content**:
```rust
//! Factories for creating test data (events, JSON values, policies, etc.).

pub mod event_factory;
pub mod cache_factory;
pub mod policy_factory;

pub use event_factory::EventFactory;
pub use cache_factory::CacheValueFactory;
pub use policy_factory::PolicyFactory;
```

## WP2: Core Infrastructure (5 tool calls, ~20 min)

### 2.1 Implement FeatureFixture builder

**File**: `/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/AgilePlus/phenotype-docs/crates/test-fixtures-shared/src/builders/feature_builder.rs`

**Content Sketch**:
```rust
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use chrono::Utc;

pub struct FeatureFixture {
    id: i64,
    slug: String,
    friendly_name: String,
    state: FeatureState,
    spec_hash: [u8; 32],
    target_branch: String,
    plane_issue_id: Option<String>,
    plane_state_id: Option<String>,
    labels: Vec<String>,
    module_id: Option<i64>,
    project_id: Option<i64>,
}

impl FeatureFixture {
    pub fn new(slug: &str, friendly_name: &str) -> Self {
        Self {
            id: 1,
            slug: slug.to_string(),
            friendly_name: friendly_name.to_string(),
            state: FeatureState::Created,
            spec_hash: [0u8; 32],
            target_branch: "main".to_string(),
            plane_issue_id: None,
            plane_state_id: None,
            labels: vec![],
            module_id: None,
            project_id: None,
        }
    }

    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }

    pub fn state(mut self, state: FeatureState) -> Self {
        self.state = state;
        self
    }

    pub fn with_shipped(mut self) -> Self {
        self.state = FeatureState::Shipped;
        self
    }

    pub fn with_pr_url(mut self, plane_issue_id: &str) -> Self {
        self.plane_issue_id = Some(plane_issue_id.to_string());
        self
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.labels.push(label.to_string());
        self
    }

    pub fn with_project_id(mut self, project_id: i64) -> Self {
        self.project_id = Some(project_id);
        self
    }

    pub fn build(self) -> Feature {
        let mut feature = Feature::new(&self.slug, &self.friendly_name, self.spec_hash, Some(&self.target_branch));
        feature.id = self.id;
        feature.state = self.state;
        feature.plane_issue_id = self.plane_issue_id;
        feature.plane_state_id = self.plane_state_id;
        feature.labels = self.labels;
        feature.module_id = self.module_id;
        feature.project_id = self.project_id;
        feature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_builder_default() {
        let feature = FeatureFixture::new("my-feature", "My Feature").build();
        assert_eq!(feature.slug, "my-feature");
        assert_eq!(feature.friendly_name, "My Feature");
        assert_eq!(feature.state, FeatureState::Created);
    }

    #[test]
    fn feature_builder_with_custom_id() {
        let feature = FeatureFixture::new("test", "Test")
            .id(42)
            .build();
        assert_eq!(feature.id, 42);
    }

    #[test]
    fn feature_builder_with_shipped() {
        let feature = FeatureFixture::new("test", "Test")
            .with_shipped()
            .build();
        assert_eq!(feature.state, FeatureState::Shipped);
    }
}
```

### 2.2 Implement WorkPackageFixture builder

**File**: `/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/AgilePlus/phenotype-docs/crates/test-fixtures-shared/src/builders/work_package_builder.rs`

**Content Sketch**:
```rust
use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use chrono::Utc;

pub struct WorkPackageFixture {
    id: i64,
    feature_id: i64,
    title: String,
    state: WpState,
    sequence: i32,
    file_scope: Vec<String>,
    acceptance_criteria: String,
    agent_id: Option<String>,
    pr_url: Option<String>,
    pr_state: Option<String>,
    worktree_path: Option<String>,
    plane_sub_issue_id: Option<String>,
}

impl WorkPackageFixture {
    pub fn new(feature_id: i64, title: &str) -> Self {
        Self {
            id: 1,
            feature_id,
            title: title.to_string(),
            state: WpState::Todo,
            sequence: 1,
            file_scope: vec![],
            acceptance_criteria: "All tests pass".to_string(),
            agent_id: None,
            pr_url: None,
            pr_state: None,
            worktree_path: None,
            plane_sub_issue_id: None,
        }
    }

    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }

    pub fn state(mut self, state: WpState) -> Self {
        self.state = state;
        self
    }

    pub fn with_pr(mut self, pr_url: &str) -> Self {
        self.pr_url = Some(pr_url.to_string());
        self.pr_state = Some("merged".to_string());
        self
    }

    pub fn with_sequence(mut self, sequence: i32) -> Self {
        self.sequence = sequence;
        self
    }

    pub fn build(self) -> WorkPackage {
        let now = Utc::now();
        WorkPackage {
            id: self.id,
            feature_id: self.feature_id,
            title: self.title,
            state: self.state,
            sequence: self.sequence,
            file_scope: self.file_scope,
            acceptance_criteria: self.acceptance_criteria,
            agent_id: self.agent_id,
            pr_url: self.pr_url,
            pr_state: self.pr_state,
            worktree_path: self.worktree_path,
            plane_sub_issue_id: self.plane_sub_issue_id,
            created_at: now,
            updated_at: now,
        }
    }
}
```

### 2.3 Implement AuditChainFixture builder

### 2.4 Implement MockStorage struct

### 2.5 Implement TestServerFixture

## WP3: AgilePlus Migration

### 3.1 Update support/storage.rs

Replace lines 29-127 with:
```rust
use test_fixtures_shared::MockStorage;

impl MockStorage {
    pub(crate) fn with_test_data() -> Self {
        let storage = Self::default();

        let feature = FeatureFixture::new("test-feature", "Test Feature")
            .id(1)
            .state(FeatureState::Implementing)
            .build();

        let wp = WorkPackageFixture::new(1, "WP01")
            .id(1)
            .state(WpState::Done)
            .with_pr("https://github.com/org/repo/pull/1")
            .build();

        storage.features.lock().unwrap().push(feature);
        storage.work_packages.lock().unwrap().push(wp);

        // ... audit chain using AuditChainFixture

        storage
    }
}
```

### 3.2 Update support/mod.rs

Replace setup functions with `TestServerFixture::new()`.

### 3.3-3.6 Update API integration tests

Update all test files to use builders in place of inline fixture construction.

## WP4: Consolidated Libraries Migration

### 4.1-4.4 Update test files

Similar pattern: replace inline fixture creation with builder/factory calls.

## WP5: Validation

### 5.1 Run full test suite

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
cargo test --all 2>&1 | tee fixture-migration.log
```

### 5.2 Create migration guide

### 5.3 Verify duplication metrics

## Key Implementation Decisions

1. **Builders for Domain Objects**: Feature, WorkPackage, Project, Cycle, Module
   - Reason: Complex initialization with many optional fields
   - Pattern: Fluent API with sensible defaults

2. **Factories for Test Data**: Events, JSON values, policies
   - Reason: Simpler creation; less stateful construction
   - Pattern: Simple functions or static methods

3. **Shared MockStorage**: Single implementation in test-fixtures-shared
   - Reason: Reduces CRUD implementation duplication
   - Reason: Single source of truth for mock behavior

4. **TestServerFixture**: Wraps setup boilerplate
   - Reason: 40+ lines of AppState, CredentialStore, Config construction
   - Pattern: Fluent builder for custom scenarios

5. **Workspace Member**: Place in `crates/test-fixtures-shared/`
   - Reason: Part of AgilePlus ecosystem; easier versioning
   - Reason: Can publish independently if needed

## Testing Strategy

### Unit Tests (in fixtures crate)
- Each builder has `#[cfg(test)] mod tests` with 3-5 tests
- Test default values, fluent API chaining, final `build()` output

### Integration Tests (in consuming crates)
- Run existing test suites after migration
- Should see no regressions
- May see improved test readability

### Regression Checks
- Before: `cargo test --all 2>&1 | grep -E "test.*ok|test.*FAILED"`
- After: Same command, expect same or better pass rate

## Rollback Plan

If issues arise:
1. Keep old fixture code in `.archive/fixtures-old/` branch
2. Git checkout old versions if needed
3. Remove test-fixtures-shared dependency from Cargo.toml files
4. Restore original test code

## Success Metrics

1. ✓ New `test-fixtures-shared` crate created and published
2. ✓ 15+ test files migrated without regressions
3. ✓ All tests pass: `cargo test --all`
4. ✓ ~650 LOC fixture duplication eliminated
5. ✓ Builder/factory patterns documented with examples
6. ✓ Migration guide created for future tests

---

**Ready to proceed**: All phases are well-defined and can be executed sequentially or in parallel (WP2+WP3 concurrent).
