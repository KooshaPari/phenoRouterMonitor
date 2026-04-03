# Phase 2: Error Core Implementation

## Status: SCAFFOLD COMPLETE; MIGRATION AND ADR REMAINING

## Verified architecture (repos checkout)

| Layer | Crate | Role |
|-------|-------|------|
| Canonical kinds | `phenotype-error-core` | `ErrorKind` / `ErrorKindInner`; shared taxonomy |
| Facade | `phenotype-errors` | Re-exports `phenotype-error-core` for legacy call sites |
| AgilePlus enums | `agileplus-error-core` | `DomainError`, `ApiError`, `StorageError`, `SyncError`, `SerializationError` with `Into<phenotype_error_core::ErrorKind>` |

Workspace member: `crates/agileplus-error-core` (see root `Cargo.toml`). Unit tests in `agileplus-error-core` cover `StorageError` and `SerializationError` mapping to `ErrorKind`.

## Goals (original)

1. ~~Consolidate on one canonical kind type~~ — **`phenotype-error-core::ErrorKind`**
2. ~~Deprecate or promote `phenotype-error-core`~~ — **Canonical; `phenotype-errors` is thin re-export layer**
3. **Shared wrapper pattern + ADR** — still open (document when to use `ContractError` vs `DomainError` vs raw `ErrorKind`)

## Completed checklist

- [x] Evaluate `phenotype-error-core` vs `phenotype-errors` — **Use `ErrorKind` from `phenotype-error-core`; prefer `phenotype-errors` only for crate boundaries that already depend on it**
- [x] Confirm `agileplus-error-core` exists and maps to `ErrorKind` — **Done in tree**

## Remaining action items

- [ ] Document error hierarchy in an ADR (port / domain / infrastructure boundaries)
- [ ] Add `agileplus-error-core` (or `phenotype-error-core`) dependencies to member crates that still define parallel enums where overlap is high (`phenotype-event-sourcing`, `phenotype-policy-engine`, etc.) — **requires AgilePlus spec + incremental PRs**
- [ ] When excluded `agileplus-*` crates re-enter the workspace, migrate their `*Error` types per `docs/worklogs/PLANS/ErrorCoreExtraction.md`
- [ ] Reconcile worklog checkboxes in `DUPLICATION.md` / `WORK_LOG.md` that still say “create agileplus-error-core” (stale)

## References

- `docs/worklogs/PLANS/ErrorCoreExtraction.md`
- `docs/worklogs/DEPENDENCIES.md` (FORK-002 error pattern)
- `crates/agileplus-error-core/src/lib.rs`
