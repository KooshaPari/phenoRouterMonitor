# Phase 2: Error Core Implementation

## Status: IN PROGRESS

## Goals
1. Consolidate error handling into phenotype-errors crate
2. Deprecate phenotype-error-core (or promote it)
3. Create shared error wrapper pattern

## Key Findings
- 6 error enums across crates with duplicated variants
- phenotype-errors used by phenotype-test-infra, phenotype-telemetry
- phenotype-error-core unused but present in workspace
- Duplicated variants: NotFound, Serialization, Conflict, Internal

## Implementation Plan
1. Audit which crate should be canonical (phenotype-errors vs phenotype-error-core)
2. Extract shared error types to phenotype-error-core
3. Migrate phenotype-test-infra, phenotype-telemetry to use phenotype-error-core
4. Document error hierarchy in ADR

## Action Items
- [ ] Evaluate phenotype-error-core vs phenotype-errors
- [ ] Create shared error wrapper pattern
- [ ] Document error hierarchy in ADR
