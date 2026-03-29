# GOVERNANCE.md — repos shelf governance

## Shelf Governance

This shelf is a **polyrepo of independent projects** owned by a single
individual (koosha-pari). Governance is therefore lightweight — the owner
makes all decisions, with agents acting as trusted helpers.

## Decision Making

| Decision type | Process |
|---------------|---------|
| New project | Owner creates + names, agent documents |
| Architecture (cross-project) | Owner decides, agent researches |
| Architecture (per-project) | Project owner decides |
| Dependency conflicts | Agent proposes options, owner chooses |
| PR merge | Owner reviews + merges |
| Deleting/archiving project | Owner initiates, agent executes |

## Code Review Standards

### Per-Project Rules
Each project may define its own review standards in `CONTRIBUTING.md`.

### Shelf-Level Standards
- No shelf-level changes without understanding downstream impact
- Breaking changes across projects must be coordinated
- Test coverage must not regress

### Agent Recommendations
Agents may flag concerns:
- Security implications
- Dependency bloat
- Cross-project duplication
- Performance regressions

Agents flag; owner decides.

## Release Management

Projects manage their own release cycles. Shelf-level coordination only
when:
- A project is archived or deleted
- Cross-project dependency changes
- Major shelf reorganization

## Standards & Conventions

### Naming
- Projects: kebab-case (`heliosCLI`, `thegent`, `phenotype-config`)
- Branches: `<project>/<type>/<description>`
- Sessions: `<project>:<brief-task>`
- Plans: `<project>-<YYYYMMDD>-<task>.md`

### Quality Gates
- **Rust projects**: `cargo clippy -- -D warnings` + tests
- **JS/TS projects**: lint + typecheck + tests
- **Python projects**: ruff check + pyright + tests
- **Cross-project**: duplication audit before major refactors

## Project Lifecycle

### Active Projects
Listed in `projects/INDEX.md` with status `active`.
Regular development, owned by the shelf owner.

### Maintenance Projects
Listed with status `maintenance`.
Minimal changes, bug fixes only, no new features.

### Archived Projects
Listed with status `archived` in `projects/INDEX.md`.
Actual code lives in `.archive/`.
Can be restored to active if needed.

### Deletion
Rare. Only after confirmed backup + no downstream dependencies.

## Tooling Governance

| Tool | Purpose | Governance |
|------|---------|------------|
| `agileplus` | Project management | Per-project spec system |
| `agileplus` CLI | Work tracking | AgilePlus project only |
| `cargo` | Rust build | Project-level |
| `bun` | JS/TS package management | Project-level |
| `task` | Task runner | Project-level |
| `buf` | Proto management | Project-level |
| `mise` | Runtime version management | Shelf-level dotfile |

## Agent Authority Levels

| Agent | Can edit | Can commit | Can push | Can merge |
|-------|----------|------------|----------|-----------|
| Forge | Any file | Any branch | Own worktrees | No |
| Muse | Comments only | No | No | No |
| Sage | Any file | Any branch | Own worktrees | No |
| Helios | Test/config files | Test branches | No | No |

**All agents ask before acting outside their authority.**

## Change Log

This file tracks governance changes to the shelf itself.

| Date | Change |
|------|--------|
| 2026-03-29 | Initial shelf governance written (previously AgilePlus-specific) |
