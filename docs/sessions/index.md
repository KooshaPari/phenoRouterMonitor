---
audience: [developers, agents, pms]
---

# Sessions

This directory contains session-led work bundles for active and historical waves.

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

## Active / recent bundles

| Session | Purpose |
|---------|---------|
| [20260330-stacked-pr-sbom](./20260330-stacked-pr-sbom/00_OVERVIEW.md) | Stacked PR / SBOM wave |
| [20260329-phase2-error-core](./20260329-phase2-error-core/README.md) | Error-core phase notes |
| [20260329-phase4-http-client](./20260329-phase4-http-client/README.md) | HTTP client audit phase |
| [20260329-phase5-config-core](./20260329-phase5-config-core/README.md) | Config-core phase notes |
