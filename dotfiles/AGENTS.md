# dotfiles — Agent Guidance

## Project Identity

This is a **template library**, not a runnable project. It contains governance templates and pre-commit hook configurations used across the Phenotype ecosystem.

## What This Project Is

- **Purpose**: Provide canonical templates for CLAUDE.md files and pre-commit hooks
- **Type**: Template library (read-only templates)
- **No runtime**: There is nothing to build, test, or run

## Agent Behavior

### DO

- Read templates from `governance/` and `hooks/` directories
- Copy templates to projects that need them
- Update template documentation when patterns change

### DON'T

- Try to build or test this project
- Run `cargo test`, `npm test`, or similar
- Add runtime code or dependencies
