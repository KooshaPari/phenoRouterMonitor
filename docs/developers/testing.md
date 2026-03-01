---
audience: [developers]
---

# Testing Guide

Testing patterns and strategies used across AgilePlus crates.

## Test Organization

```
crate/
├── src/
│   └── lib.rs
└── tests/
    ├── unit/          # Fast, isolated tests
    ├── integration/   # Cross-module tests
    └── fixtures/      # Test data
```

## Running Tests

```bash
cargo test                       # All tests
cargo test -p agileplus-core     # Single crate
cargo test governance            # Tests matching name
cargo test -- --nocapture        # Show stdout
```

## Unit Tests

Keep unit tests next to the code they test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_validates_required_fields() {
        let spec = Spec::builder()
            .title("Test Feature")
            .build();

        assert!(spec.validate().is_ok());
    }

    #[test]
    fn spec_rejects_empty_title() {
        let spec = Spec::builder()
            .title("")
            .build();

        assert!(spec.validate().is_err());
    }
}
```

## Integration Tests

Integration tests live in `tests/` and test cross-crate interactions:

```rust
#[test]
fn full_pipeline_creates_worktree_and_branch() {
    let tmp = tempdir().unwrap();
    let project = TestProject::init(&tmp);

    project.specify("Add login");
    project.plan("001");
    project.implement("WP01");

    assert!(tmp.path().join(".worktrees/001-add-login-WP01").exists());
}
```

## Port Trait Testing

Use mock implementations of port traits:

```rust
struct MockStorage {
    specs: HashMap<FeatureId, Spec>,
}

impl StoragePort for MockStorage {
    fn read_spec(&self, id: &FeatureId) -> Result<Spec> {
        self.specs.get(id).cloned().ok_or(Error::NotFound)
    }
    // ...
}
```

## Test Fixtures

Place test data in `tests/fixtures/`:

```
tests/fixtures/
├── valid-spec.md
├── invalid-spec-missing-title.md
└── sample-plan.md
```

Load fixtures with:

```rust
let spec_content = include_str!("fixtures/valid-spec.md");
```
