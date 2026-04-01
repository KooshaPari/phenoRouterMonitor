# AGENTS.md — thegent-shm methodology

## Project Identity

This is the **thegent-shm** — a methodology shim for thegent agent framework work
inside the AgilePlus shelf. thegent is the agent framework for phenotypic development
automation.

## Agent Rules

**READ shelf-level `AGENTS.md` FIRST.** This file supplements shelf rules with
project-specific conventions for thegent development.

### Project-Specific Conventions

- thegent uses domain-driven decomposition (MODELS, PLAN, GOVERNANCE, TEAM)
- All domain extractions should preserve backward compatibility via feature flags
- GitOps refactor follows the shim split pattern defined in WP011
- Codebase atlas generation is the canonical reference for project structure

### Commit Scope

| Scope | Description |
|-------|-------------|
| `cli` | Command-line interface changes |
| `domain` | Domain extraction/refactoring |
| `atlas` | Codebase atlas generation |
| `gitops` | GitOps pipeline changes |
| `models` | MODELS domain |

### Quality Gates

Before calling `gt_done`:
1. `cargo check --all-features` passes
2. `cargo test --workspace` passes
3. No new `TODO` or `FIXME` comments introduced
