# Feature: phenotype-error-core - Shared Error Library Extraction

**Feature ID**: phenotype-error-core
**Status**: SPECIFIED
**Created**: 2026-03-29
**Category**: Libification / Technical Debt

## Objective

Extract and consolidate duplicated error types from 4 crates (15+ overlapping error variants across 442 LOC) into a single, shared `phenotype-error-core` library.

## Problem Statement

Currently, the Phenotype monorepo has error enums scattered across multiple crates:
- `phenotype-contracts` defines 2 separate error enums (inbound + outbound ports)
- `phenotype-event-sourcing` defines 3 error enums
- `phenotype-policy-engine` defines 1 error enum
- `phenotype-cache-adapter` and `phenotype-state-machine` have partial error handling

This creates:
- Code duplication (NotFound, Validation, Timeout variants repeated 2-3x)
- Inconsistent error naming conventions (Error vs EventSourcingError vs PolicyEngineError)
- Difficult error handling at integration boundaries
- 49% opportunity for LOC reduction

## Scope

### In Scope
- Create new `phenotype-error-core` crate with consolidated ErrorKind enum
- Migrate 5 error enums from 4 crates
- Update error construction in all dependent code
- Add comprehensive error conversion (From impls)
- Preserve domain-specific error semantics through wrapper enums
- Add tests for error handling and conversions

### Out of Scope
- Changes to error handling logic in application layer
- Changes to error message formats (except unification)
- Changes to HTTP/gRPC error code mappings (handled elsewhere)

## Acceptance Criteria

1. **New Crate Created**: `crates/phenotype-error-core/` with Cargo.toml, properly configured
2. **ErrorKind Enum**: Comprehensive enum with at least 14 variants (NotFound, Validation, Serialization, Timeout, Internal, Storage, Connection, Config, PermissionDenied, Conflict, AlreadyExists, ParseError, NetworkError, AuthError)
3. **Error Trait**: Implements `thiserror::Error` for display/conversion
4. **Serde Support**: `#[derive(Serialize, Deserialize)]` for JSON marshalling
5. **Conversions**: From implementations for common std error types (io::Error, serde_json::Error, regex::Error, etc.)
6. **Domain Wrappers**: Event sourcing and policy engine preserve domain-specific errors via wrapper enums
7. **Zero Regression**: All tests pass in all 4 migrated crates
8. **LOC Reduction**: Achieved minimum 180 LOC reduction (target 49%)
9. **PR Merged**: Feature branch merged to main with CI passing

## Technical Specification

### ErrorKind Enum Definition

```rust
#[derive(Debug, thiserror::Error, Clone, serde::Serialize, serde::Deserialize)]
pub enum ErrorKind {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("network error: {0}")]
    NetworkError(String),

    #[error("authentication error: {0}")]
    AuthError(String),
}

pub type Result<T> = std::result::Result<T, ErrorKind>;
```

### Domain-Specific Wrapper Enums

**EventSourceError** (in phenotype-event-sourcing):
- Preserves DuplicateSequence, SequenceGap, InvalidHash variants
- Can convert to/from ErrorKind::Storage or ErrorKind::Internal

**PolicyError** (in phenotype-policy-engine):
- Preserves RegexCompilation, InvalidConfiguration variants
- Can convert to/from ErrorKind::Config

### Crate Dependencies

**phenotype-error-core/Cargo.toml**:
```toml
[dependencies]
serde = { workspace = true, optional = true }
thiserror = { workspace = true }
serde_json = { workspace = true, optional = true }

[features]
default = ["serde"]
serde = ["dep:serde", "dep:serde_json"]
```

**phenotype-contracts**:
```toml
[dependencies]
phenotype-error-core = { path = "../phenotype-error-core" }
```

(Same for other crates)

## Work Packages

### WP1: Setup phenotype-error-core
- [ ] Create new crate via `cargo new --lib crates/phenotype-error-core`
- [ ] Add Cargo.toml dependencies (serde, thiserror)
- [ ] Register in workspace Cargo.toml
- [ ] Create src/lib.rs with ErrorKind enum
- [ ] Implement std::fmt::Display (via thiserror)
- [ ] Add From implementations for common error types
- **Acceptance**: Crate compiles, tests pass, ErrorKind enum has 14+ variants

### WP2: Migrate phenotype-contracts Inbound Error
- [ ] Replace inbound::Error with phenotype-error-core::ErrorKind
- [ ] Update error construction calls (CustomError::NotFound → ErrorKind::NotFound)
- [ ] Add tests for error handling
- [ ] Verify UseCase, CommandHandler, QueryHandler work with new error type
- **Acceptance**: All contract tests pass, error handling unchanged

### WP3: Migrate phenotype-contracts Outbound Error
- [ ] Replace outbound::Error with phenotype-error-core::ErrorKind
- [ ] Update Repository, CachePort, SecretPort, EventPublisher interface impls
- [ ] Add conversion tests for all port implementations
- **Acceptance**: All outbound tests pass, port contracts verified

### WP4: Migrate phenotype-event-sourcing
- [ ] Create EventSourceError wrapper (preserves domain semantics)
- [ ] Update EventStoreError to use ErrorKind for common errors
- [ ] Update HashError to use ErrorKind for common errors
- [ ] Add From<ErrorKind> and Into<ErrorKind> implementations
- [ ] Update event sourcing tests
- **Acceptance**: Event sourcing tests pass, error semantics preserved

### WP5: Migrate phenotype-policy-engine
- [ ] Create PolicyError wrapper (preserves domain semantics)
- [ ] Update PolicyEngineError to delegate common errors to ErrorKind
- [ ] Add From/Into implementations
- [ ] Verify regex compilation, policy evaluation, configuration errors work
- **Acceptance**: Policy engine tests pass, domain semantics preserved

### WP6: Verify & Integration Tests
- [ ] Run full test suite for all 4 crates
- [ ] Cross-crate error propagation tests (contracts → event-sourcing, contracts → policy-engine)
- [ ] Serde serialization/deserialization tests
- [ ] Verify error messages are consistent
- **Acceptance**: All tests pass, 0 regressions

### WP7: PR & Merge
- [ ] Create branch feat/extract-phenotype-error-core
- [ ] Commit changes with clear history
- [ ] Push to origin
- [ ] Create PR with detailed description
- [ ] Merge to main with --admin if needed
- **Acceptance**: PR merged, main branch updated, CI passing

## Definition of Done

- [x] Audit complete (ERROR_LIBRARY_EXTRACTION_AUDIT.md)
- [ ] AgilePlus spec created (this document)
- [ ] All WP1-WP6 tasks completed
- [ ] All tests pass (unit + integration)
- [ ] PR merged to main
- [ ] Documentation updated (if needed)
- [ ] Error handling guide updated in docs/

## Metrics & Success Criteria

| Metric | Target | Actual |
|--------|--------|--------|
| Error enums consolidated | 5 → 1 shared + wrappers | TBD |
| LOC reduction | ≥180 LOC (49%) | TBD |
| Test coverage | ≥80% | TBD |
| Regression tests | All pass | TBD |
| CI passes | 100% | TBD |

## Related Issues & Context

- **Phenotype Cross-Project Reuse Protocol**: This extraction supports the mandate to modularize and extract shared code across repos
- **Libification Roadmap**: Part of Phase 1 consolidation efforts (see LIBIFICATION_ROADMAP_PHASE1.md)
- **OSS Wrapping Audit**: Uses `thiserror` (maintained OSS)

## Implementation Owner Notes

- Keep domain-specific errors (DuplicateSequence, SequenceGap, RegexCompilation) in their respective crates
- Use wrapper enums to bridge between domain errors and shared ErrorKind
- Add comprehensive From impls to ensure error propagation is seamless
- Consider adding error context via backtrace support (future: anyhow integration)

## Review Checklist

- [ ] Spec aligns with Phenotype governance and libification goals
- [ ] Technical approach is sound (error hierarchy, domain preservation)
- [ ] No breaking API changes for downstream consumers
- [ ] LOC reduction target is realistic
- [ ] Test coverage is comprehensive
- [ ] Implementation timeline is feasible
