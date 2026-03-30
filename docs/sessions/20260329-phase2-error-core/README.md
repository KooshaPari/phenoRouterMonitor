# Phase 2: Error Core Implementation

## Status: SCAFFOLD AND BOUNDARIES IN PLACE; EXTEND MIGRATION AS CRATES CHANGE

## Canonical stack (current repo)

| Layer | Crate | Role |
|-------|-------|------|
| Kinds | `phenotype-error-core` | `ErrorKind` at boundaries |
| Facade | `phenotype-errors` | Re-exports `phenotype-error-core` |
| AgilePlus | `agileplus-error-core` | Domain/API/sync/storage enums with `Into<ErrorKind>` |

## Completed

- [x] Evaluate `phenotype-error-core` vs `phenotype-errors` — **`ErrorKind` in `phenotype-error-core` is canonical**
- [x] `From<…> for ErrorKind` for **`phenotype-event-sourcing`**, **`phenotype-policy-engine`**, **`ContractError`**
- [x] ADR — **`docs/reference/ADR_ERROR_LAYER_BOUNDARIES.md`**

## Remaining (incremental)

- [ ] Apply the same `Into<ErrorKind>` pattern to other crates when editing them
- [ ] Reconcile stale checkboxes in `DUPLICATION.md` / `WORK_LOG.md`
- [ ] `git2` → `gix` in `phenotype-git-core` per `docs/worklogs/DEPENDENCIES.md` (separate initiative)

## References

- `docs/reference/ADR_ERROR_LAYER_BOUNDARIES.md`
- `docs/worklogs/PLANS/ErrorCoreExtraction.md`
