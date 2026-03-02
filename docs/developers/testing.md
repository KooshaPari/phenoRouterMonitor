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
