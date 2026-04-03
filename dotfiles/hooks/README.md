# Pre-commit Hooks — Consolidated Templates

This directory contains the canonical pre-commit hook configuration for the Phenotype ecosystem. All projects should use `.pre-commit-config.base.yaml` as their foundation.

## Quick Start

```bash
# Copy the base config to your project
cp path/to/thegent/dotfiles/hooks/.pre-commit-config.base.yaml my-project/.pre-commit-config.yaml

# Install hooks locally
cd my-project
pre-commit install

# Test your config
pre-commit run --all-files
```

## Templates

### .pre-commit-config.base.yaml
**Canonical base configuration** — Use this for all Phenotype projects.

**Includes**:
- Gate 1: Base syntax validation (YAML, JSON, TOML, merge conflicts, large files)
- Gate 2: Linting & formatting (Python, Rust, TypeScript, Go, Proto)
- Gate 3: Type checking (Python, TypeScript)
- Gate 4: Quality gates (max lines, custom checks)
- Gate 5: Security & governance (secret detection, policy checks)
- Conventional commits validation

**How to Use**:
1. Copy to project root as `.pre-commit-config.yaml`
2. Uncomment/customize sections for your language stack
3. Add project-specific local hooks
4. Run `pre-commit install`

**Language-Specific Gates**:

| Language | Gate 2 (Lint) | Gate 3 (Type) | Notes |
|----------|---------------|---------------|-------|
| Python | ruff check + ruff format | basedpyright | Include both |
| Rust | rustfmt + clippy | (via clippy -D warnings) | Keep both local hooks |
| TypeScript | oxlint + prettier | tsc --noEmit | Include all three |
| Go | gofmt + golangci-lint | (via lint) | Keep both |
| Proto | buf lint | (built-in) | For .proto files |

---

## Universal Hooks (all projects)

These hooks are required for ALL Phenotype projects. Never remove or disable them:

### Gate 1: Syntax Validation

| Hook | Purpose | Stages |
|------|---------|--------|
| `trailing-whitespace` | Remove trailing spaces | pre-commit, pre-push |
| `end-of-file-fixer` | Ensure files end with newline | pre-commit, pre-push |
| `check-yaml` | Validate YAML syntax | pre-commit, pre-push |
| `check-toml` | Validate TOML syntax | pre-commit, pre-push |
| `check-json` | Validate JSON syntax | pre-commit, pre-push |
| `check-merge-conflict` | Prevent merge conflict markers | pre-commit, pre-push |
| `check-added-large-files` | Block files > 500 KB | pre-commit, pre-push |
| `detect-private-key` | Prevent credential commits | pre-commit, pre-push |
| `no-commit-to-branch` | Prevent commits to main/master | pre-commit, pre-push |

### Gate 2-3: Secret Detection

| Hook | Purpose | Stages |
|------|---------|--------|
| `trufflehog` | Production-grade secret scanning | pre-commit, pre-push |

### Gate X: Commit Messages

| Hook | Purpose | Stages |
|------|---------|--------|
| `conventional-pre-commit` | Enforce conventional commits | commit-msg |

---

## Language-Specific Hooks

### Python Projects

**Required Gates**:
- Gate 2: ruff (lint + format)
- Gate 3: basedpyright (strict type checking)

```yaml
# .pre-commit-config.yaml
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.10.0
    hooks:
      - id: ruff
        stages: [pre-commit, pre-push]
        args: ["--fix"]
      - id: ruff-format
        stages: [pre-commit, pre-push]

  - repo: local
    hooks:
      - id: basedpyright
        name: basedpyright
        entry: basedpyright
        language: system
        types: [python]
        pass_filenames: true
        stages: [pre-push]
```

**Configuration Files**:
- `pyproject.toml` — Ruff config
- `basedpyright.json` — Type checking config

### Rust Projects

**Required Gates**:
- Gate 2: rustfmt + clippy

```yaml
  - repo: local
    hooks:
      - id: rustfmt
        name: rustfmt
        entry: bash -c 'cargo fmt -- --check'
        language: system
        files: '\.rs$'
        pass_filenames: false
        stages: [pre-commit, pre-push]

      - id: clippy
        name: clippy
        entry: bash -c 'cargo clippy --all-targets -- -D warnings'
        language: system
        files: '\.rs$'
        pass_filenames: false
        stages: [pre-commit, pre-push]
```

**Configuration Files**:
- `clippy.toml` or `.clippy.toml` — Clippy linter config
- `Cargo.toml` — Rust workspace config

### TypeScript/JavaScript Projects

**Required Gates**:
- Gate 2: oxlint + prettier
- Gate 3: tsc (TypeScript Compiler)

```yaml
  - repo: local
    hooks:
      - id: oxlint
        name: oxlint
        entry: oxlint --fix .
        language: system
        files: '\.(ts|tsx|js|jsx)$'
        pass_filenames: false
        stages: [pre-commit, pre-push]

      - id: prettier
        name: prettier
        entry: prettier --write .
        language: system
        files: '\.(ts|tsx|js|jsx|json|yaml|md)$'
        pass_filenames: false
        stages: [pre-commit, pre-push]

      - id: tsc
        name: TypeScript Compiler
        entry: tsc --noEmit
        language: system
        files: '\.(ts|tsx)$'
        pass_filenames: false
        stages: [pre-push]
```

**Configuration Files**:
- `tsconfig.json` — TypeScript config
- `.eslintrc.json` or `oxlint.json` — Linter config
- `.prettierrc` or `prettier.config.js` — Formatter config

### Go Projects

**Required Gates**:
- Gate 2: gofmt + golangci-lint

```yaml
  - repo: local
    hooks:
      - id: gofmt
        name: gofmt
        entry: bash -c 'go fmt ./...'
        language: system
        files: '\.go$'
        pass_filenames: false
        stages: [pre-commit, pre-push]

      - id: golangci-lint
        name: golangci-lint
        entry: golangci-lint run
        language: system
        files: '\.go$'
        pass_filenames: false
        stages: [pre-commit, pre-push]
```

**Configuration Files**:
- `.golangci.yml` — Linter config
- `go.mod` — Go module config

### Protocol Buffers

**Required Gates**:
- Gate 2: buf lint

```yaml
  - repo: local
    hooks:
      - id: buf-lint
        name: buf lint
        entry: buf lint
        language: system
        files: '\.proto$'
        pass_filenames: false
        stages: [pre-commit, pre-push]
```

---

## Project-Specific Hooks

Add custom local hooks for project-specific checks. Examples:

### Documentation Build Verification

```yaml
  - repo: local
    hooks:
      - id: docs-build
        name: Build Documentation
        entry: bash hooks/pre-commit-docs.sh
        language: script
        pass_filenames: false
        files: '^docs/'
        stages: [pre-push]
```

### Custom Architecture Validation

```yaml
  - repo: local
    hooks:
      - id: architecture-check
        name: Architecture Boundary Check
        entry: bash scripts/check-architecture.sh
        language: system
        files: '\.py$'
        pass_filenames: false
        stages: [pre-push]
```

### Integration Test Subset

```yaml
  - repo: local
    hooks:
      - id: integration-tests
        name: Integration Tests (subset)
        entry: pytest tests/integration -v
        language: system
        pass_filenames: false
        stages: [pre-push]
```

---

## Testing & Debugging

### Run All Hooks
```bash
pre-commit run --all-files
```

### Run Specific Hook
```bash
pre-commit run <hook-id> --all-files
```

### List All Hooks
```bash
pre-commit run --list
```

### Skip Specific Hook
```bash
SKIP=hook-id git commit -m "message"
```

### Bypass All Pre-commit Hooks
```bash
git commit --no-verify -m "message"
```

### View Hook Output
```bash
pre-commit run -v <hook-id> --all-files
```

### Install/Uninstall Hooks
```bash
pre-commit install           # Install git hooks
pre-commit install-hooks     # Install hook environments
pre-commit uninstall         # Remove git hooks
```

---

## Customization Guide

### For New Projects

1. Copy `.pre-commit-config.base.yaml` to your project root as `.pre-commit-config.yaml`
2. Uncomment sections for your language stack
3. Customize paths if your project structure differs from standard
4. Add project-specific local hooks
5. Test with `pre-commit run --all-files`
6. Commit `.pre-commit-config.yaml` to version control
7. Run `pre-commit install` when cloning project

### For Existing Projects

1. Compare your current `.pre-commit-config.yaml` with the base template
2. Ensure universal hooks (Gate 1-2) are present
3. Add missing language-specific hooks
4. Test with `pre-commit run --all-files`
5. Commit changes to version control

### Adjusting File Exclusions

```yaml
# Example: Exclude vendor/template directories from JSON check
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v5.0.0
    hooks:
      - id: check-json
        exclude: |
          (?x)^(
            node_modules/.+|
            vendor/.+|
            test_fixtures/.+
          )$
```

---

## Common Patterns

### Multi-Language Monorepo

```yaml
# .pre-commit-config.yaml for monorepo with Python, Rust, TypeScript

# Python
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.10.0
    hooks:
      - id: ruff
        args: ["--fix"]
      - id: ruff-format

# Rust (if rust/ subdirectory)
  - repo: local
    hooks:
      - id: rustfmt
        entry: bash -c 'cd rust && cargo fmt -- --check'
        files: 'rust/.*\.rs$'

# TypeScript (if packages/ subdirectory)
  - repo: local
    hooks:
      - id: oxlint
        entry: bash -c 'cd packages && oxlint --fix .'
        files: 'packages/.*\.(ts|tsx|js)$'
```

### Monorepo with Workspace-Specific Hooks

```yaml
# Run hooks only for changed files in specific workspace
  - repo: local
    hooks:
      - id: python-tests
        name: Python Tests
        entry: bash -c 'python -m pytest python/tests -k "changed" --tb=short'
        language: system
        files: '^python/'
        pass_filenames: false
        stages: [pre-push]

      - id: rust-tests
        name: Rust Tests
        entry: bash -c 'cd rust && cargo test --all'
        language: system
        files: '^rust/.*\.rs$'
        pass_filenames: false
        stages: [pre-push]
```

---

## Staging Strategies

### Pre-commit Stage (Runs on Every Commit)
Fast checks only:
- Syntax validation (YAML, JSON, TOML)
- Merge conflict detection
- Private key detection
- Trailing whitespace
- File size limits

**Recommended hooks for pre-commit stage**:
- check-yaml, check-json, check-toml
- trailing-whitespace
- check-added-large-files
- detect-private-key
- no-commit-to-branch

### Pre-push Stage (Runs Before Pushing)
Comprehensive checks:
- All Gate 1-2 hooks
- Linting (ruff, oxlint, clippy, golangci-lint)
- Secret detection (trufflehog)
- Type checking (basedpyright, tsc)

**Recommended hooks for pre-push stage**:
- All hooks from pre-commit
- Plus: ruff, clippy, oxlint, basedpyright, trufflehog

### Commit-msg Stage
- conventional-pre-commit

---

## Troubleshooting

### Issue: Hook takes too long
**Solution**: Move expensive hooks to pre-push stage only
```yaml
  - id: expensive-hook
    stages: [pre-push]  # Don't run on every commit
```

### Issue: Hook fails for vendor/generated code
**Solution**: Add exclusions
```yaml
  - id: ruff
    exclude: |
      (?x)^(
        vendor/.+|
        .*_generated\.py$
      )$
```

### Issue: Hook not running
**Troubleshoot**:
```bash
pre-commit run <hook-id> -v --all-files  # Show verbose output
pre-commit validate-config                 # Check YAML syntax
pre-commit validate-manifest               # Check hook manifests
```

### Issue: False positives in secret detection
**Solution**: Add test fixture exclusions
```yaml
  - id: detect-private-key
    exclude: |
      (?x)^(
        tests/fixtures/.*\.pem|
        test_data/.*\.key
      )$
```

---

## Integration with CI/CD

### GitHub Actions

```yaml
# .github/workflows/lint.yml
name: Pre-commit Checks

on: [push, pull_request]

jobs:
  pre-commit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-python@v4
        with:
          python-version: '3.11'
      - uses: pre-commit/action@v3
```

### Local CI Equivalent

```bash
#!/bin/bash
# scripts/run-pre-commit.sh
pre-commit run --all-files
if [ $? -ne 0 ]; then
  echo "Pre-commit checks failed"
  exit 1
fi
```

---

## Version Management

- **Current Version**: 1.0 (2026-03)
- **Stability**: Stable
- **Update Frequency**: Quarterly
- **Breaking Changes**: Major version increment
- **Changelog**: See `worklogs/GOVERNANCE.md`

---

## See Also

- **CLAUDE.md Template**: `dotfiles/governance/CLAUDE.base.md`
- **Linter Configs**: `templates/<language>/`
- **Quality Gates**: `templates/quality/`
- **Pre-commit Official Docs**: https://pre-commit.com

---

## Template Info

- **Location**: `thegent/dotfiles/hooks/`
- **Canonical URL**: See thegent repository
- **Maintained By**: Phenotype Governance Team
- **Last Updated**: 2026-03-29
