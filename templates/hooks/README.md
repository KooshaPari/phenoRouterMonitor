# Canonical Git Hooks

This directory contains the canonical git hooks for all projects in the Phenotype ecosystem.

## Available Hooks

| Hook | Purpose |
|------|---------|
| `pre-commit` | Trailing whitespace, merge conflict markers, large file detection |
| `pre-push` | Branch-aware checks (lint/tests based on branch type) |
| `commit-msg` | Conventional commit format validation |

## Installation

### For Individual Projects

```bash
# From project root
cp templates/hooks/* .git/hooks/
chmod +x .git/hooks/*
```

### Using git config (Recommended)

```bash
# From repository root
git config core.hooksPath templates/hooks
```

### Using install script

```bash
# From repository root
bash templates/hooks/install.sh
```

## Hook Details

### pre-commit

- **Trailing whitespace detection**: Fails if staged files contain trailing whitespace
- **Merge conflict markers**: Fails if staged files contain conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`)
- **Large file warning**: Warns if files >5MB are staged (recommends Git LFS)

### pre-push

- **main protection**: Prevents direct push to main branch
- **Feature branches** (`feature/*`): Runs lint checks only
- **Canary branches** (`canary/*`): Runs lint + tests
- **Release branches** (`beta/*`, `rc/*`): Runs lint + tests, requires ROLLBACK.md

### commit-msg

Validates conventional commit format:
```
<type>(<scope>): <message>

Types: feat, fix, chore, docs, style, refactor, perf, test, ci, build, revert

Examples:
  feat(auth): add JWT token refresh
  fix(parser): handle empty input gracefully
  docs(readme): update installation instructions
```

## Customization

Projects can extend these hooks by:

1. Copying to project-specific `.githooks/` directory
2. Modifying the project copy
3. Setting `git config core.hooksPath .githooks`

## Maintenance

These hooks are the single source of truth. Updates should be made here and propagated to projects.
