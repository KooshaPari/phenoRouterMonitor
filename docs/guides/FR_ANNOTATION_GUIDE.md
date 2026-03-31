# FR Annotation Guide

**Purpose:** Standardize how Functional Requirements (FRs) are documented and traced in code.

## Overview

FRs (Functional Requirements) are documented in `docs/reference/FR_TRACKER.md` and must be traced in code using `// Traces to: FR-XXX-NNN` comments.

## Annotation Format

### In Code Files

```rust
// Traces to: FR-DOMAIN-001
pub struct Feature {
    id: Uuid,
    name: String,
    state: FeatureState,
}

// Traces to: FR-AUDIT-002
fn compute_hash(data: &[u8]) -> [u8; 32] {
    sha2_256(data)
}
```

### In Test Files

```rust
// Traces to: FR-DOMAIN-001
#[test]
fn test_feature_crud() {
    // ...
}
```

## FR ID Format

| Category | Prefix | Example |
|----------|--------|---------|
| Domain Model | FR-DOMAIN | FR-DOMAIN-001 |
| Audit Trail | FR-AUDIT | FR-AUDIT-001 |
| CLI | FR-CLI | FR-CLI-001 |
| API | FR-API | FR-API-001 |
| Graph | FR-GRAPH | FR-GRAPH-001 |
| P2P | FR-P2P | FR-P2P-001 |

## Status Values

| Status | Meaning |
|--------|---------|
| Implemented | FR is fully implemented with passing tests |
| Partial | FR has working implementation but needs test coverage |
| Missing | FR is not yet implemented |

## Checklist for New FR

- [ ] Add entry to `docs/reference/FR_TRACKER.md`
- [ ] Add `// Traces to: FR-XXX-NNN` comment to implementation
- [ ] Add test with `// Traces to: FR-XXX-NNN`
- [ ] Update category totals in summary table

## Common Mistakes

1. **Missing comment** - Always add `// Traces to:` comment
2. **Wrong FR ID** - Double-check FR_TRACKER.md
3. **Test name mismatch** - Test names should reference FR ID

## Example PR Description

```markdown
## Traces to

- FR-DOMAIN-001: Feature entity
- FR-DOMAIN-002: FeatureState enum
- FR-AUDIT-001: AuditEntry entity
```

## Maintenance

| Date | Change |
|------|--------|
| 2026-03-28 | Initial version |
| 2026-03-30 | Updated formatting |
