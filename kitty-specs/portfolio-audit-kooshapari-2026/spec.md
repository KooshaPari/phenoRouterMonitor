# Portfolio audit: KooshaPari legacy + Pheno SDK

## Goal
Establish a single tracked program to inventory, assess, and modernize KooshaPari-era and CodeProjects-local work (2023–2026), with Pheno SDK (`phenoSDK` GitHub) as the primary monolith input and Phenotype `libs/*` as extraction targets.

## Scope
- GitHub org `KooshaPari` (249 repos): triage by last push, archive status, overlap with `Phenotype/repos`.
- Local `CodeProjects/*` (KooshaPari/Dino, archive, orphans, Dev, learning): shallow health + link to remote if any.
- Canonical SDK tree: `Phenotype/repos/worktrees/phenoSDK/main` (clone of `github.com/KooshaPari/phenoSDK`). Legacy `pheno-sdk` remote is empty; do not block on it.

## Acceptance criteria
1. Inventory artifacts committed or referenced from `docs/reports/` in Phenotype repos.
2. Per-repo or per-cluster backlog: hexagonal boundaries, CI/QA gaps, decomposition candidates (no copy-paste from third-party capstone; inspiration-only for atoms-era ideas).
3. Pheno SDK: language map, hot-spot modules, proposed package split aligned with `docs/governance/23_ARCHITECTURAL_GOVERNANCE.md` (polyrepo, XDD, productized libs).
4. Optional acceleration: PyO3/Rust/Zig only where profiling justifies; default remains Python ports with thin native adapters.

## Non-goals (this phase)
- Rewriting Pheno SDK in one pass.
- Resolving GitHub Actions billing on KooshaPari account.

## Work packages (initial)
- WP-A1: Freeze org inventory + local path map.
- WP-A2: Pheno SDK structure + tokei baseline + test/lint smoke.
- WP-A3: Crosswalk `phenoSDK` packages to `libs/python/phenotype-sdk`, `phenotype-go-sdk`, `phenotype-ts-sdk`, etc.
- WP-A4: AgilePlus `plan` + `research` for top three extraction epics.
