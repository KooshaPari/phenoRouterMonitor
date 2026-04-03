# Research

## Shelf worklog model
- `docs/worklogs/AGENT_ONBOARDING.md` defines a categorized worklog system, not a single monolithic file.
- `docs/worklogs/WORK_LOG.md` records a canonical multi-file structure with an index, topic-specific logs, and `.archive/`.
- `docs/worklogs/README.md` currently has unresolved merge markers, so treat it as a weak signal until cleaned.

## Template platform model
- `template-program-ops/SPEC.md` defines a single logical template surface spanning shared, starter, hexagonal, and domain repos.
- `template-program-ops/kitty-specs/layered-template-platform/ADR.md` explicitly chooses separate repositories by concern, with orchestration in `template-program-ops`.
- `template-commons/ADR.md` centralizes shared scaffolding logic, variable format, and hook protocol.
- `template-domain-service-api/ADR.md` and `template-domain-*/PLAN.md` require layered composition: commons -> language -> domain.

## Current path-state notes
- `docs/sessions/20260403-template-registry-consolidation-research/01_RESEARCH.md` shows root `template-lang-*` placeholders are stale/empty and active language surfaces live elsewhere.
- `docs/sessions/20260403-template-registry-consolidation-research/05_KNOWN_ISSUES.md` says tooling should migrate from removed placeholder paths to active paths.
- `docs/sessions/20260403-template-repo-state-sweep/01_RESEARCH.md` confirms only four real root template repos remain canonical at shelf root.

## Supporting source-of-truth pattern
- AgilePlus specs define git artifacts as source of truth and SQLite as operational state/cache, reinforcing split human/machine authority instead of one file/system doing everything.
- AgilePlus proto scaffolding requires a separate `agileplus-proto` repository as the contract source of truth.
