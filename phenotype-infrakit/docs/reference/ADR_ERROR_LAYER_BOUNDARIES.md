# ADR: Error layer boundaries (Phenotype / AgilePlus)

**Status:** Accepted  
**Date:** 2026-03-28  
**Traces to:** FR-PHENO-001, error-core consolidation

## Context

Multiple crates defined overlapping error enums (`NotFound`, `Serialization`, `Internal`, etc.). We need a single **cross-crate vocabulary** for logging, HTTP mapping, and telemetry without deleting rich domain-specific enums.

## Decision

1. **`phenotype-error-core::ErrorKind`** is the **canonical transport type** at boundaries (ports, HTTP adapters, shared telemetry). It is a closed set: `NotFound`, `Io`, `Serialization`, `Storage`, `Conflict`, `Validation`, `Internal`, `Connection`.

2. **`phenotype-errors`** remains a **thin re-export** of `phenotype-error-core` for crates that already depend on the `phenotype_errors` package name. New code should depend on `phenotype-error-core` directly when possible.

3. **`agileplus-error-core`** holds **AgilePlus-shaped** enums (`DomainError`, `ApiError`, `SyncError`, …) that implement **`Into<ErrorKind>`** (and related markers) so AgilePlus code stays expressive while still mapping to the shared taxonomy.

4. **Domain crates** (`phenotype-event-sourcing`, `phenotype-policy-engine`, `phenotype-contracts`, …) **keep** their primary `*Error` enums for API clarity. Each such enum **implements `From<…> for ErrorKind`** (or `Into<ErrorKind>`) with an explicit, reviewed mapping table. Callers that need only a stable kind use `.into()`.

5. **`ContractError`** is the **contract / port facade**: it may wrap `ErrorKind` as `Infrastructure(_)`. Mapping rules:
   - `Port` → `ErrorKind::connection`
   - `InvariantViolation`, invalid input → `validation`
   - `NotFound` / `Conflict` / `Serialization` → same-named kinds
   - `Infrastructure(k)` → `k`

## Consequences

- **Pros:** One stable `kind()` string for metrics; no forced loss of domain enums; incremental migration.
- **Cons:** Two-step errors (domain enum then `ErrorKind`) require discipline; mapping must stay updated when variants are added.
- **Not in scope here:** Replacing `git2` with `gix` in `phenotype-git-core` (track under `docs/worklogs/DEPENDENCIES.md` / RUSTSEC follow-up).

## Implementation references

- `crates/phenotype-error-core/src/lib.rs` — `ErrorKind`
- `crates/agileplus-error-core/src/domain.rs` — `Into<ErrorKind>`
- `crates/phenotype-contracts/src/error.rs` — `ContractError` bidirectional story with `ErrorKind`
- `crates/phenotype-event-sourcing/src/error.rs`, `crates/phenotype-policy-engine/src/error.rs` — `From<crate::Error> for ErrorKind`
