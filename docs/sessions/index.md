---
audience: [developers, agents, pms]
---

# Sessions

This directory contains session-led work bundles for active and historical waves.

## Structure

Each session lives under:

`sessions/<YYYYMMDD-descriptive-name>/`

That path is **relative to this documentation package** (the `docs/` folder in the Phenotype `repos` checkout, i.e. `repos/docs/sessions/…`).

### Bundle files

Prefer the standard set:

- `00_SESSION_OVERVIEW.md` — primary overview (some older bundles use `00_OVERVIEW.md`; treat as equivalent)
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

## Note on VitePress

Session markdown is listed under `srcExclude` in `.vitepress/config.mts` so it is **not** compiled as site routes (Vue would mis-parse Rust generics and `<` in prose). Sessions remain the source of truth in git; the published site focuses on curated guides and reference.
