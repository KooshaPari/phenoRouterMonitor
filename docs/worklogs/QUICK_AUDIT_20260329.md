# Quick Audit Findings - 2026-03-29

## Critical Worktree Sync
- **AgilePlus/phenotype-docs**: Synced 12 files (Storage Adapter Unification Analysis + Specs) to `chore/integrate-phenotype-docs`.
- **merge-spec-docs**: Synced to `chore/consolidate-cost-tracking`.
- **Note**: PRs for these require manual merge due to unrelated histories with main.

## Duplication / Libification Targets
- **Stub Crates**: `phenotype-port-traits`, `phenotype-validation`, `phenotype-health` (partially) are stubs.
- **Action**: Move traits from `agileplus-domain/src/ports/` to `phenotype-port-traits`.
- **Error Consolidation**: 14+ public error enums found. Move to `phenotype-errors` or `phenotype-error-core`.

## 3rd Party Replacement (Research)
- **cqrs-es**: Candidate to replace custom `phenotype-event-sourcing` (~2k LOC savings).
- **casbin-rs**: Candidate for `phenotype-policy-engine` (~3k LOC savings).
- **gitoxide (gix)**: Replace `git2` in all CLI tools.
