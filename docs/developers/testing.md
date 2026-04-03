---
audience: [developers]
---

# Testing Guide

Testing patterns and strategies used across AgilePlus crates. Follow these practices to ensure code quality and prevent regressions.

## Test Organization

```
crate/
├── src/
│   ├── lib.rs
│   ├── models/
│   │   ├── spec.rs
│   │   └── spec.rs#tests    ← unit tests inline
│   └── engine/
│       ├── planner.rs
│       └── planner.rs#tests ← unit tests inline
└── tests/
    ├── integration/         ← Cross-module tests
    │   ├── spec_parsing_test.rs
    │   └── plan_generation_test.rs
    └── fixtures/            ← Test data
        ├── valid-oauth-spec.md
        ├── invalid-spec-missing-title.md
        └── sample-plan.md
```

## Running Tests

```bash
# All tests (unit + integration)
cargo test --all

# Single crate
cargo test -p agileplus-core

# Tests matching a pattern
cargo test spec

# Show println! output
cargo test -- --nocapture

# Single-threaded (for tests with side effects)
cargo test -- --test-threads=1

# With logging
RUST_LOG=debug cargo test

# Coverage
cargo tarpaulin --out Html
```

## Unit Tests

Write unit tests inline with the code they test:

### Example: Spec Validation

```rust
// src/models/spec.rs

pub struct Spec {
    pub title: String,
    pub description: String,
    pub requirements: Vec<Requirement>,
}

impl Spec {
    pub fn validate(&self) -> Result<()> {
        if self.title.is_empty() {
            return Err(Error::EmptyTitle);
        }
        if self.requirements.is_empty() {
            return Err(Error::NoRequirements);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_spec_passes_validation() {
        let spec = Spec {
            title: "OAuth2 Auth".to_string(),
            description: "Add OAuth2 support".to_string(),
            requirements: vec![
                Requirement::new("FR-1", "Login with Google"),
            ],
        };

        assert!(spec.validate().is_ok());
    }

    #[test]
    fn empty_title_fails_validation() {
        let spec = Spec {
            title: "".to_string(),
            description: "Add OAuth2 support".to_string(),
            requirements: vec![],
        };

        assert!(matches!(spec.validate(), Err(Error::EmptyTitle)));
    }

    #[test]
    fn no_requirements_fails_validation() {
        let spec = Spec {
            title: "OAuth2 Auth".to_string(),
            description: "".to_string(),
            requirements: vec![],
        };

        assert!(matches!(spec.validate(), Err(Error::NoRequirements)));
    }
}
```

### Best Practices for Unit Tests

```rust
// ✓ Good: Clear naming
#[test]
fn spec_requires_non_empty_title() { }

#[test]
fn spec_requires_at_least_one_requirement() { }

// ✗ Bad: Vague naming
#[test]
fn test_spec() { }

#[test]
fn spec_works() { }

// ✓ Good: Arrange-Act-Assert
#[test]
fn plan_decomposes_requirements_into_work_packages() {
    // Arrange: Set up test data
    let req = Requirement::new("FR-1", "Add login");
    let spec = Spec::with_requirements(vec![req]);

    // Act: Execute the code being tested
    let plan = generate_plan(&spec).unwrap();

    // Assert: Check the result
    assert_eq!(plan.work_packages.len(), 1);
    assert_eq!(plan.work_packages[0].title, "Login endpoint");
}

// ✗ Bad: Multiple assertions without structure
#[test]
fn test_plan() {
    let plan = generate_plan(&spec).unwrap();
    assert!(plan.is_valid());
    assert!(plan.work_packages.len() > 0);
    // What are we testing?
}
```

## Integration Tests

Integration tests live in `tests/` directory and test cross-crate interactions:

### Example: Full Pipeline

```rust
// tests/integration/full_pipeline_test.rs

use agileplus::prelude::*;
use std::fs;

#[test]
fn full_pipeline_spec_to_plan() {
    // Setup: Create a temporary directory
    let tmp = tempdir::TempDir::new("agileplus-test").unwrap();
    let root = tmp.path();

    // Create a spec file
    let spec_content = r#"
# Specification: OAuth2 Authentication

## Functional Requirements
FR-1: Users can log in with Google OAuth
FR-2: Users can log in with GitHub OAuth
FR-3: Sessions persist across requests

## Success Criteria
SC-1: Login completes in < 2 seconds
SC-2: Session valid for 30 days
"#;

    fs::write(root.join("spec.md"), spec_content).unwrap();

    // Parse the spec
    let spec = Spec::from_file(root.join("spec.md")).unwrap();
    assert_eq!(spec.title, "OAuth2 Authentication");
    assert_eq!(spec.requirements.len(), 3);

    // Generate plan
    let plan = generate_plan(&spec).unwrap();
    assert!(plan.work_packages.len() >= 2); // At least Google and GitHub
    assert!(plan.validate().is_ok());
}

#[test]
fn plan_generation_respects_dependencies() {
    let spec = load_spec("tests/fixtures/auth-spec.md");
    let plan = generate_plan(&spec).unwrap();

    // WP02 (Google) should depend on WP01 (provider config)
    let wp02 = plan.find_wp("WP02").unwrap();
    assert!(wp02.depends_on.contains(&FeatureId::new("WP01")));

    // No circular dependencies
    assert!(plan.validate_dependencies().is_ok());
}
```

## Async Testing

For async code, use `#[tokio::test]`:

```rust
// tests/integration/storage_test.rs

use agileplus_adapters::storage::FileStorage;
use agileplus_ports::StoragePort;

#[tokio::test]
async fn file_storage_roundtrips_spec() {
    let tmp = tempdir::TempDir::new("storage-test").unwrap();
    let storage = FileStorage::new(tmp.path()).unwrap();

    let spec = Spec::new("Test Feature");
    let feature_id = FeatureId::new("001");

    // Write
    storage.write_spec(&feature_id, &spec).await.unwrap();

    // Read
    let retrieved = storage.read_spec(&feature_id).await.unwrap();
    assert_eq!(retrieved.title, spec.title);
}

#[tokio::test]
async fn concurrent_writes_dont_corrupt() {
    let tmp = tempdir::TempDir::new("concurrent-test").unwrap();
    let storage = std::sync::Arc::new(FileStorage::new(tmp.path()).unwrap());

    let mut handles = vec![];

    // Write 10 specs concurrently
    for i in 0..10 {
        let storage_clone = storage.clone();
        let handle = tokio::spawn(async move {
            let spec = Spec::new(&format!("Feature {}", i));
            let id = FeatureId::new(&format!("{:03}", i));
            storage_clone.write_spec(&id, &spec).await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all were written
    let features = storage.list_features().await.unwrap();
    assert_eq!(features.len(), 10);
}
```

## Port Trait Testing with Mocks

Use mock implementations to test code that depends on ports:

```rust
// tests/mocks/mock_storage.rs

use agileplus_ports::StoragePort;
use std::collections::HashMap;

pub struct MockStorage {
    specs: HashMap<FeatureId, Spec>,
    plans: HashMap<FeatureId, Plan>,
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            specs: HashMap::new(),
            plans: HashMap::new(),
        }
    }

    pub fn with_spec(mut self, id: FeatureId, spec: Spec) -> Self {
        self.specs.insert(id, spec);
        self
    }
}

#[async_trait::async_trait]
impl StoragePort for MockStorage {
    async fn read_spec(&self, id: &FeatureId) -> Result<Spec> {
        self.specs
            .get(id)
            .cloned()
            .ok_or(Error::NotFound)
    }

    async fn write_spec(&self, id: &FeatureId, spec: &Spec) -> Result<()> {
        // In-memory mock: just succeed
        Ok(())
    }

    async fn list_features(&self) -> Result<Vec<FeatureId>> {
        Ok(self.specs.keys().cloned().collect())
    }
}

// Usage in tests
#[test]
fn planner_uses_storage_to_load_spec() {
    let storage = MockStorage::new()
        .with_spec(
            FeatureId::new("001"),
            Spec::new("Test Feature")
        );

    let planner = Planner::new(Box::new(storage));
    let plan = planner.plan_feature(&FeatureId::new("001")).unwrap();

    assert!(plan.work_packages.len() > 0);
}
```

## Test Fixtures

Store reusable test data in `tests/fixtures/`:

```
tests/fixtures/
├── oauth-spec.md          # Valid spec
├── invalid-no-title.md    # Invalid spec
├── complex-plan.md        # Multi-WP plan
└── github-sync-response.json # Mock API response
```

Load fixtures:

```rust
// In tests
let spec_content = include_str!("../fixtures/oauth-spec.md");
let spec = Spec::parse(spec_content).unwrap();

// Or from file
let spec = Spec::from_file("tests/fixtures/oauth-spec.md").unwrap();
```

### Example Fixture

```markdown
# tests/fixtures/oauth-spec.md

---
title: OAuth2 Authentication
audience: [developers, agents, pms]
---

# OAuth2 Authentication

## Functional Requirements

FR-1: Users can sign up via Google OAuth
FR-2: Users can sign up via GitHub OAuth
FR-3: Sessions persist across browser restarts

## Success Criteria

SC-1: Both OAuth flows complete in < 2 seconds
SC-2: Session valid for 30 days
SC-3: Error handling prevents account takeover
```

## Coverage Requirements

Minimum test coverage targets:

- **Core domain logic**: 90% (critical for correctness)
- **Engine/orchestration**: 85% (important for reliability)
- **Adapters**: 70% (implementation details, less critical)
- **CLI**: 50% (mostly routing, tested manually)

Measure with `cargo-tarpaulin`:

```bash
cargo tarpaulin --out Html --minimum 85
```

```html
<!-- Generated report -->
agileplus-core:     92% coverage
agileplus-engine:   88% coverage
agileplus-adapters: 74% coverage
agileplus-cli:      55% coverage

Overall: 82% (meets target of 80%+)
```

## Property-Based Testing

For complex logic, use property-based testing with `proptest`:

```rust
// tests/properties/plan_properties.rs

use proptest::prelude::*;

proptest! {
    #[test]
    fn plan_dependencies_are_acyclic(spec in any::<Spec>()) {
        let plan = generate_plan(&spec).expect("should generate plan");
        prop_assert!(plan.validate_dependencies().is_ok());
    }

    #[test]
    fn all_requirements_covered(spec in any::<Spec>()) {
        let plan = generate_plan(&spec).expect("should generate plan");
        for req in &spec.requirements {
            prop_assert!(
                plan.work_packages.iter().any(|wp| wp.covers(req)),
                "Requirement {:?} not covered by any WP",
                req
            );
        }
    }

    #[test]
    fn work_packages_dont_exceed_spec_scope(
        spec in any::<Spec>(),
        config in plan_config_strategy()
    ) {
        let plan = generate_plan_with_config(&spec, &config).expect("should generate plan");
        let total_wps = plan.work_packages.len();
        // Rough heuristic: shouldn't have more than 3x requirements
        prop_assert!(total_wps <= spec.requirements.len() * 3);
    }
}
```

## Performance Testing

For performance-critical operations:

```rust
#[bench]
fn bench_spec_parsing(b: &mut Bencher) {
    let spec_content = include_str!("../fixtures/large-spec.md");

    b.iter(|| {
        Spec::parse(spec_content).unwrap()
    });
}

#[bench]
fn bench_plan_generation(b: &mut Bencher) {
    let spec = load_spec("../fixtures/large-spec.md");

    b.iter(|| {
        generate_plan(&spec).unwrap()
    });
}
```

Run with:

```bash
cargo bench
```

## CI/CD Integration

Tests run automatically on every PR:

```yaml
# .github/workflows/test.yml

name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run tests
        run: cargo test --all --verbose

      - name: Check formatting
        run: cargo fmt --check

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Code coverage
        run: cargo tarpaulin --out Xml
```

## Tips

1. **Test behavior, not implementation** — Test what the code does, not how it does it
2. **Use descriptive names** — Names like `spec_requires_non_empty_title` are self-documenting
3. **Keep tests small** — Each test should verify one thing
4. **Use fixtures** — Avoid duplicating test data
5. **Mock external dependencies** — Use mocks to isolate code under test
6. **Test edge cases** — Empty input, maximum values, error conditions
7. **Run tests frequently** — Run tests before committing, use watch mode during development

## TestHarness: Integration Test Infrastructure

The `TestHarness` struct provides a complete, real-infrastructure test environment without mocks. It spins up:
- An in-memory SQLite database (no file I/O)
- A temporary git repository
- A NATS test server (embedded)

```rust
// crates/agileplus-engine/tests/common/harness.rs

pub struct TestHarness {
    pub storage: Arc<dyn StoragePort>,
    pub vcs: Arc<dyn VcsPort>,
    pub engine: Engine,
    pub tmp_dir: TempDir,
    pub nats: TestNatsServer,
}

impl TestHarness {
    pub async fn new() -> anyhow::Result<Self> {
        let tmp_dir = TempDir::new("agileplus-test")?;

        // In-memory SQLite
        let storage = SqliteStorageAdapter::in_memory().await?;

        // Real git repo in tmp dir
        let repo_path = tmp_dir.path().join("repo");
        git2::Repository::init(&repo_path)?;
        let vcs = GitVcsAdapter::new(repo_path.clone())?;

        // Embedded NATS for event testing
        let nats = TestNatsServer::start().await?;

        let engine = Engine::builder()
            .storage(Arc::new(storage.clone()))
            .vcs(Arc::new(vcs.clone()))
            .nats_url(&nats.url())
            .build()
            .await?;

        Ok(Self { storage: Arc::new(storage), vcs: Arc::new(vcs), engine, tmp_dir, nats })
    }

    /// Create a feature in Created state
    pub async fn create_feature(&self, slug: &str) -> Feature {
        let feature = Feature::new(slug, "Test Feature", [0u8; 32], None);
        let id = self.storage.create_feature(&feature).await.unwrap();
        Feature { id, ..feature }
    }

    /// Create a feature in Planned state with N work packages
    pub async fn planned_feature(&self, slug: &str, wp_count: usize) -> (Feature, Vec<WorkPackage>) {
        let feature = self.create_feature(slug).await;
        // ... transition through states, create WPs ...
        (feature, wps)
    }
}
```

Example usage:

```rust
#[tokio::test]
async fn feature_validates_when_all_wps_done() {
    let h = TestHarness::new().await.unwrap();
    let (feature, wps) = h.planned_feature("user-auth", 3).await;

    // Complete all WPs
    for wp in &wps {
        h.engine.transition_wp(wp.id, WpState::Doing).await.unwrap();
        h.engine.transition_wp(wp.id, WpState::ForReview).await.unwrap();
        h.engine.transition_wp(wp.id, WpState::Done).await.unwrap();
    }

    // Validate the feature
    let result = h.engine.validate_feature(feature.id).await;
    assert!(result.is_ok());

    // Verify state changed
    let updated = h.storage.get_feature_by_slug("user-auth").await.unwrap().unwrap();
    assert_eq!(updated.state, FeatureState::Validated);
}
```

## Testing the Audit Hash Chain

The audit chain is a critical integrity component. Test it thoroughly:

```rust
#[tokio::test]
async fn audit_chain_detects_tampering() {
    let h = TestHarness::new().await.unwrap();
    let feature = h.create_feature("test-feature").await;

    // Transition through several states to build up chain
    h.engine.specify_feature(feature.id, "spec content").await.unwrap();
    h.engine.research_feature(feature.id, "research content").await.unwrap();

    // Get the audit trail
    let entries = h.storage.get_audit_trail(feature.id).await.unwrap();
    assert_eq!(entries.len(), 2);

    // Verify chain integrity
    let chain = AuditChain { entries };
    assert!(chain.verify_chain().is_ok());
}

#[tokio::test]
async fn audit_chain_fails_on_modification() {
    let h = TestHarness::new().await.unwrap();
    let feature = h.create_feature("test-feature").await;
    h.engine.specify_feature(feature.id, "spec content").await.unwrap();

    // Tamper with the database directly (simulating an attack)
    sqlx::query("UPDATE audit_entries SET actor = 'attacker' WHERE id = 1")
        .execute(h.storage.pool())
        .await
        .unwrap();

    let entries = h.storage.get_audit_trail(feature.id).await.unwrap();
    let chain = AuditChain { entries };

    // Should detect the tampering
    assert!(matches!(
        chain.verify_chain(),
        Err(AuditChainError::TamperedEntry { entry_id: 1, .. })
    ));
}
```

## Testing the Dependency Graph

```rust
#[tokio::test]
async fn dependency_graph_detects_cycles() {
    let mut graph = DependencyGraph::new();

    // Create a cycle: WP1 → WP2 → WP3 → WP1
    graph.add_edge(WpDependency { wp_id: 2, depends_on: 1, dep_type: DependencyType::Explicit });
    graph.add_edge(WpDependency { wp_id: 3, depends_on: 2, dep_type: DependencyType::Explicit });
    graph.add_edge(WpDependency { wp_id: 1, depends_on: 3, dep_type: DependencyType::Explicit });

    assert!(graph.has_cycle());
    assert!(matches!(
        graph.execution_order(),
        Err(DomainError::CyclicDependency { .. })
    ));
}

#[tokio::test]
async fn dependency_graph_computes_parallel_layers() {
    let mut graph = DependencyGraph::new();

    // WP1 → no deps
    // WP2 → WP1
    // WP3 → WP1
    // WP4 → WP2, WP3
    graph.add_edge(WpDependency { wp_id: 2, depends_on: 1, dep_type: DependencyType::Explicit });
    graph.add_edge(WpDependency { wp_id: 3, depends_on: 1, dep_type: DependencyType::Explicit });
    graph.add_edge(WpDependency { wp_id: 4, depends_on: 2, dep_type: DependencyType::Explicit });
    graph.add_edge(WpDependency { wp_id: 4, depends_on: 3, dep_type: DependencyType::Explicit });

    let layers = graph.execution_order().unwrap();
    assert_eq!(layers.len(), 3);
    assert_eq!(layers[0], vec![1]);          // WP1 alone
    assert_eq!(layers[1].sort(), vec![2, 3].sort()); // WP2 and WP3 in parallel
    assert_eq!(layers[2], vec![4]);          // WP4 after both
}
```

## Benchmark Patterns with Criterion

For performance-critical operations, use `criterion`:

```toml
# crates/agileplus-domain/Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }

[[bench]]
name = "audit_chain"
harness = false
```

```rust
// crates/agileplus-domain/benches/audit_chain.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use agileplus_domain::domain::audit::{AuditEntry, AuditChain, hash_entry};

fn bench_hash_single_entry(c: &mut Criterion) {
    let entry = AuditEntry {
        id: 1,
        feature_id: 42,
        wp_id: None,
        timestamp: chrono::Utc::now(),
        actor: "human:alice".into(),
        transition: "created->specified".into(),
        evidence_refs: vec![],
        prev_hash: [0u8; 32],
        hash: [0u8; 32],
    };

    c.bench_function("hash_entry", |b| {
        b.iter(|| hash_entry(&entry))
    });
}

fn bench_verify_chain_by_length(c: &mut Criterion) {
    let mut group = c.benchmark_group("verify_chain");

    for n_entries in [10, 100, 1000].iter() {
        let entries = build_chain_of(*n_entries);
        let chain = AuditChain { entries };

        group.bench_with_input(
            BenchmarkId::from_parameter(n_entries),
            n_entries,
            |b, _| {
                b.iter(|| chain.verify_chain())
            }
        );
    }

    group.finish();
}

criterion_group!(benches, bench_hash_single_entry, bench_verify_chain_by_length);
criterion_main!(benches);
```

Run and compare:

```bash
cargo bench -p agileplus-domain -- audit_chain
# audit_chain/hash_entry     time: [1.24 µs 1.26 µs 1.28 µs]
# audit_chain/verify/10      time: [12.3 µs 12.5 µs 12.7 µs]
# audit_chain/verify/100     time: [124 µs 126 µs 128 µs]
# audit_chain/verify/1000    time: [1.24 ms 1.26 ms 1.28 ms]
```

Performance is linear in chain length — expected given SHA-256 is O(n) per entry.

## Next Steps

- [Contributing](contributing.md) — Development setup and PR workflow
- [Extending](extending.md) — Adding adapters and subcommands
- [Architecture Overview](../architecture/overview.md) — Crate structure
- [Domain Model](../architecture/domain-model.md) — Entity relationships for test data
