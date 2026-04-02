# thegent-cache Delete Readiness

**Date:** 2026-04-01
**Scope:** validate whether archived GitHub repo `KooshaPari/thegent-cache` should remain archived or be deleted

## Decision

`thegent-cache` is **not delete-ready**.

Best-practice disposition:

- keep the GitHub repo archived
- treat the archived repo as historical and provenance surface
- do not delete until the active in-tree crate identity and docs are fully migrated

## Why

- The active `thegent` workspace still includes `thegent-cache` as a real crate.
- The crate and Python package names are still `thegent-*`, not `pyfacet`.
- The shelf has no live `pyfacet` references.
- The codebase and docs still describe `thegent-cache` as an active caching component.

## Canonical Evidence

- [thegent/crates/Cargo.toml](/Users/kooshapari/CodeProjects/Phenotype/repos/thegent/crates/Cargo.toml)
- [thegent/crates/thegent-cache/Cargo.toml](/Users/kooshapari/CodeProjects/Phenotype/repos/thegent/crates/thegent-cache/Cargo.toml)
- [thegent/crates/thegent-cache/pyproject.toml](/Users/kooshapari/CodeProjects/Phenotype/repos/thegent/crates/thegent-cache/pyproject.toml)
- [platforms/thegent/crates/Cargo.toml](/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/crates/Cargo.toml)
- [consolidation-audit-2026-03-29.md](/Users/kooshapari/CodeProjects/Phenotype/repos/docs/research/consolidation-audit-2026-03-29.md)

## Outcome

Deletion is blocked. Archive retention is correct for now.
