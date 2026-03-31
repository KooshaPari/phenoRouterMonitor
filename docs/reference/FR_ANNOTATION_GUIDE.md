# FR (Functional Requirement) Annotation Guide

This guide explains how to annotate tests with Functional Requirement (FR) references to maintain traceability between specifications and test coverage.

## Overview

The spec validation gate enforces three key rules:

1. **FR Uniqueness**: No duplicate FR IDs across the codebase
2. **Test Coverage**: Every FR in `FUNCTIONAL_REQUIREMENTS.md` must have ≥1 test
3. **Test Traceability**: Every test must reference ≥1 FR to ensure tests validate actual requirements

## FR Annotation Formats

### Rust Tests

Use a comment format `// Traces to: FR-XXX-YYY` before or within the test:

```rust
#[tokio::test]
async fn health_endpoint_requires_no_auth() {
    // Traces to: FR-API-005, FR-DOMAIN-014
    // Verify that /health endpoint returns service health without authentication
    let server = setup_test_server().await;
    let resp = server.get("/health").await;
    resp.assert_status_ok();

    let body: serde_json::Value = resp.json();
    assert!(body["status"].is_string());
}
```

Multiple FRs can be referenced in a single test:

```rust
#[test]
fn state_transition_blocks_backwards_movement() {
    // Traces to: FR-DOMAIN-002, FR-DOMAIN-003, FR-DOMAIN-004
    // Verify forward-only state transitions
    let mut feature = Feature::new("test");
    feature.transition_to(FeatureState::Shipped).unwrap();

    // Attempt to transition backwards should fail
    let result = feature.transition_to(FeatureState::Planned);
    assert!(result.is_err());
}
```

### Python Tests

Use either inline comments or pytest markers:

**Option 1: Comment format (preferred)**
```python
def test_feature_creation_sets_created_state():
    # Traces to: FR-DOMAIN-001, FR-CLI-001
    # Verify that a newly created feature starts in "Created" state
    feature = Feature.create(slug="test-feature", name="Test")
    assert feature.state == FeatureState.CREATED
```

**Option 2: pytest marker**
```python
@pytest.mark.requirement("FR-DOMAIN-001")
@pytest.mark.requirement("FR-CLI-001")
def test_feature_creation_sets_created_state():
    """Verify that a newly created feature starts in 'Created' state"""
    feature = Feature.create(slug="test-feature", name="Test")
    assert feature.state == FeatureState.CREATED
```

### TypeScript/JavaScript Tests

```typescript
// Traces to: FR-API-001, FR-API-007
// Verify that unauthenticated requests are rejected with 401
describe("Feature API", () => {
    it("should require authentication for feature endpoints", async () => {
        const response = await fetch("/api/features", {
            method: "GET",
            // No Authorization header
        });
        expect(response.status).toBe(401);
    });
});
```

## FR ID Format

FR IDs follow the pattern: `FR-<DOMAIN>-<NUMBER>`

- **DOMAIN**: 3-8 uppercase letters identifying the feature area (e.g., `API`, `DOMAIN`, `STORAGE`, `GRPC`)
- **NUMBER**: 3-digit zero-padded number (e.g., `001`, `002`, `100`)

Examples:
- `FR-API-001` — First API requirement
- `FR-DOMAIN-014` — Fourteenth domain model requirement
- `FR-STORAGE-004` — Fourth storage/persistence requirement

## Finding Relevant FRs

1. **By domain**: Look for `## FR-<DOMAIN>:` sections in `FUNCTIONAL_REQUIREMENTS.md`
2. **By feature**: Search for keywords in the requirement description
3. **By component**: Check the "Code Location" column for references to your module

Example:
```markdown
| FR-API-005 | The Axum HTTP server SHALL expose `GET /health` ... | E12.3 | `crates/agileplus-api/` |
```

If testing `crates/agileplus-api/`, look for FRs with that code location.

## Best Practices

### One Test, One FR

Each test should primarily trace to one FR. If a test validates multiple requirements, reference all of them:

✅ **Good**:
```rust
// Traces to: FR-API-005 (primary), FR-DOMAIN-014 (secondary)
```

❌ **Avoid**:
```rust
// Traces to: FR-API-001, FR-API-002, FR-API-003, FR-API-004, FR-API-005
```

### Include Descriptive Comments

Always explain what the test verifies:

✅ **Good**:
```rust
// Traces to: FR-DOMAIN-003
// Verify that state transitions enforce forward-only movement
```

❌ **Avoid**:
```rust
// Traces to: FR-DOMAIN-003
```

### Group Related Tests

Tests for the same FR can be grouped together:

```rust
mod state_transition_tests {
    // Traces to: FR-DOMAIN-003, FR-DOMAIN-004

    #[test]
    fn forward_transitions_succeed() { ... }

    #[test]
    fn backward_transitions_fail() { ... }
}
```

### Inline Tests in Source

For unit tests within source files (not in `tests/` directory), use the same format:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_fr_id() {
        // Traces to: FR-GOVERN-001
        // Verify that FR ID parsing correctly identifies domain and number
        let fr = FrId::parse("FR-API-042").unwrap();
        assert_eq!(fr.domain, "API");
        assert_eq!(fr.number, 42);
    }
}
```

## Validation

The spec validation gate runs on every commit and:

1. **Extracts all FR IDs** from `FUNCTIONAL_REQUIREMENTS.md`
2. **Scans test files** in `crates/` for FR references
3. **Reports coverage** as: `(FRs with tests / Total FRs) * 100%`
4. **Blocks commits** if any FR lacks test coverage (unless bypassed with `--no-verify`)

### Bypassing Validation

For work-in-progress commits, you can bypass validation:

```bash
git commit --no-verify -m "WIP: adding API features"
```

**Note**: The main branch CI/CD pipeline will still enforce 100% coverage.

### Manual Validation

To manually check FR coverage without committing:

```bash
bash .githooks/spec-validator
```

## Evolving FRs and Tests

When updating `FUNCTIONAL_REQUIREMENTS.md`:

1. **New FRs**: Add tests immediately or create WIP commits with `--no-verify`
2. **Modified FRs**: Ensure tests still align with the new requirement
3. **Removed FRs**: Update test annotations to reference remaining FRs

## Examples

### Complete Example: Feature State Machine

**Requirement** (in `FUNCTIONAL_REQUIREMENTS.md`):
```
| FR-DOMAIN-003 | System SHALL enforce that state transitions are forward-only unless explicitly allowed; any attempt to transition to a state with a lower ordinal than the current state SHALL return `DomainError::InvalidTransition` |
```

**Test with FR Traceability** (in `crates/agileplus-domain/tests/state_machine_tests.rs`):
```rust
#[test]
fn enforce_forward_only_transitions() {
    // Traces to: FR-DOMAIN-003
    // Requirement: State transitions must be forward-only; backward transitions
    // must fail with InvalidTransition error

    let mut feature = Feature {
        id: 1,
        state: FeatureState::Shipped,
        // ... other fields
    };

    // Attempt to transition from Shipped (ordinal 6) to Validated (ordinal 5)
    let result = feature.transition_to(FeatureState::Validated);

    // Assertion verifies FR-DOMAIN-003
    assert!(matches!(
        result,
        Err(DomainError::InvalidTransition { .. })
    ));
}
```

## Questions?

If unsure about which FR to reference:

1. Search `FUNCTIONAL_REQUIREMENTS.md` for keywords
2. Check the "Code Location" column for the relevant crate
3. Ask in PR review — reviewers can suggest the right FR

---

**Last Updated**: 2026-03-30
**Related**: `FUNCTIONAL_REQUIREMENTS.md`, `.githooks/spec-validator`
