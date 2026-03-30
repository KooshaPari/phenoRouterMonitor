# ADR 001: Task Runner Selection and Standardization

**Status**: Accepted

**Date**: 2026-03-01

## Context

The Phenotype organization operates 47 repositories with a heterogeneous mix of task runner solutions:

- **Taskfile** (task): Used in ~18 repos, primarily Go and mixed-language projects
- **npm scripts**: Used in ~15 repos, primarily TypeScript/Node.js projects
- **GNU Make**: Used in ~14 repos, primarily Rust and legacy projects

This fragmentation creates several operational problems:

1. **Onboarding friction**: Engineers must learn different invocation patterns per repo (`task lint` vs `npm run lint` vs `make lint`)
2. **CI/CD complexity**: Pipeline templates must branch on repo type to invoke the correct runner
3. **Automation gaps**: The pheno-cli toolchain cannot uniformly invoke lint, test, build, or audit across repos
4. **Audit inconsistency**: Security audit tasks (`cargo audit`, `pip-audit`, `npm audit`) are present in only ~60% of repos and invoked differently

## Decision

Standardize on **mise** (v2026.x) as the organization-wide task runner, with per-repo `mise.toml` configuration files.

### Why mise

- **Polyglot**: Native support for Go, Rust, Python, TypeScript, and shell tasks in a single config format
- **Tool version management**: Doubles as a version manager (replaces `.nvmrc`, `.python-version`, `rust-toolchain.toml`)
- **TOML-based**: Human-readable, diff-friendly config; no special DSL
- **Performance**: Tasks run faster than npm scripts due to no Node.js startup overhead
- **Ecosystem**: Active maintenance, broad adoption, compatible with existing tools (cargo, go, ruff, eslint)
- **Standard task names**: Enforces uniform task naming via validation

### Alternatives Considered

| Tool | Reason Rejected |
|------|-----------------|
| Taskfile (task) | Already in use but YAML verbosity; no tool version management |
| Make | Non-portable (BSD vs GNU), whitespace-sensitive, poor Windows support |
| npm scripts | Node.js dependency for non-JS repos; JSON not ideal for complex tasks |
| just | Good ergonomics but no tool version management; smaller ecosystem |
| Earthly | Build-system overkill for task running; steeper learning curve |

## Required Tasks

Every repo `mise.toml` must define the following tasks:

| Task | Purpose |
|------|---------|
| `lint` | Static analysis (golangci-lint, ruff check, eslint, clippy) |
| `test` | Run test suite |
| `build` | Produce build artifact |
| `format` | Auto-format source code |

Optional but recommended:

| Task | Purpose |
|------|---------|
| `audit` | Security vulnerability scan |
| `docs:build` | Generate documentation |
| `release:promote` | Promote release candidate |
| `release:status` | Check release pipeline status |

## Migration Path

1. **Phase 1 (WP11)**: Define reference `mise.toml` templates per language; implement validator in pheno-cli
2. **Phase 2 (bootstrap)**: `pheno-cli repo bootstrap` generates `mise.toml` from detected language
3. **Phase 3 (CI)**: Update CI templates to invoke `mise run lint`, `mise run test`, etc.
4. **Phase 4 (deprecation)**: Remove legacy `Taskfile.yml`, `Makefile`, and npm-script task equivalents after all repos are migrated

## Consequences

### Positive

- Uniform `mise run <task>` invocation across all 47 repos
- pheno-cli can drive lint/test/build/audit without repo-type detection
- Tool versions pinned in `mise.toml` eliminate "works on my machine" issues
- Single config file replaces up to 4 separate version-pin files

### Negative

- **Migration cost**: Each repo needs a `mise.toml` authored or generated; ~47 PRs
- **Learning curve**: Engineers unfamiliar with mise need orientation (~30 min)
- **Dual maintenance**: During migration, both old and new task runners coexist in some repos

### Neutral

- mise is a Rust binary distributed via install script or Homebrew; no runtime dependency on Node/Python/Go
- Existing Taskfile/Makefile tasks can be invoked from mise tasks during transition
