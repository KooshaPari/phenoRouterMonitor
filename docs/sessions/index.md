---
audience: [developers, agents, pms]
---

# Sessions

This directory contains session-led work bundles for active and historical waves.

## Active / recent bundles

| Session | Entry file |
|---------|------------|
| PR Review (2026-03-30) | [`pr-review-20260330/README.md`](./pr-review-20260330/README.md) |
| Stacked PR + SBOM (2026-03-30) | [`20260330-stacked-pr-sbom/00_OVERVIEW.md`](./20260330-stacked-pr-sbom/00_OVERVIEW.md) |
| Phase 2 error-core (2026-03-29) | [`20260329-phase2-error-core/README.md`](./20260329-phase2-error-core/README.md) |
| Phase 4 http-client (2026-03-29) | [`20260329-phase4-http-client/README.md`](./20260329-phase4-http-client/README.md) |
| Phase 5 config-core (2026-03-29) | [`20260329-phase5-config-core/README.md`](./20260329-phase5-config-core/README.md) |

## Structure

Each session should live under:

`docs/sessions/<YYYYMMDD-descriptive-name>/`

and should normally contain:

- `00_SESSION_OVERVIEW.md`
- `01_RESEARCH.md`
- `02_SPECIFICATIONS.md`
- `03_DAG_WBS.md`
- `04_IMPLEMENTATION_STRATEGY.md`
- `05_KNOWN_ISSUES.md`
- `06_TESTING_STRATEGY.md`

## Rules

- Keep transient execution evidence inside the session bundle.
- Promote only durable repo-wide guidance into canonical docs.
- Update the active session bundle continuously so later waves can resume cleanly.
