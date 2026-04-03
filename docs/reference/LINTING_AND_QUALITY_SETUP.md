# Linting & Quality Gates Setup — Tier 1 Repos

**Status**: Phase 1 Implementation
**Last Updated**: 2026-03-30
**Scope**: phenotype-infrakit, heliosCLI, platforms/thegent

This guide establishes baseline code quality enforcement across Tier 1 repositories with opinionated, pre-configured linting, formatting, and type checking hooks.

---

## Overview

### Quality Gate Layers

| Layer | Trigger | Tools | Purpose |
|-------|---------|-------|---------|
| **Layer 1: Local Pre-Commit** | Before each commit | rustfmt, clippy, ruff, black, gofmt, eslint | Fast, local feedback |
| **Layer 2: Local Quality Gate** | Before push | Layer 1 + typechecking | Comprehensive local validation |
| **Layer 3: GitHub Actions** | PR open/push | Layer 2 + coverage, security | CI/CD pipeline validation |
| **Layer 4: Branch Protection** | Before merge | All CI checks + reviews | Final quality assurance |

### Repository Language Stacks

#### phenotype-infrakit
- **Primary**: Rust (Cargo workspace)
- **Status**: Scaffold (README only)
- **Setup**: Defer until crates added

#### heliosCLI
- **Primary**: Rust (Cargo workspace, 18 crates)
- **Secondary**: Python (PyO3 FFI)
- **Status**: In development
- **Setup**: Prioritized

#### platforms/thegent
- **Primary**: Go (distributed services)
- **Secondary**: Python (tooling, scripts)
- **Tertiary**: Rust (security hooks)
- **Status**: Mature monorepo (16,745 Go files)
- **Setup**: Already configured (thegent `.pre-commit-config.yaml` is comprehensive)

---

## Layer 1: Pre-Commit Configuration

Pre-commit hooks run locally before each commit. They are **fast**, **blocking**, and **immediate feedback**.

### Installation

```bash
# Install pre-commit framework (global)
pip install pre-commit

# In each repo: install hooks
cd <repo>
pre-commit install
pre-commit install --hook-type pre-push

# Run hooks on all files (optional, for CI/validation)
pre-commit run --all-files
```

### Base Configuration (All Repos)

Save as `.pre-commit-config.yaml` in repo root:

```yaml
repos:
  # --- Syntax & General Validation (Gate 1) ---
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v5.0.0
    hooks:
      - id: trailing-whitespace
        stages: [pre-commit, pre-push]
      - id: end-of-file-fixer
        stages: [pre-commit, pre-push]
      - id: check-yaml
        stages: [pre-commit, pre-push]
      - id: check-toml
        stages: [pre-commit, pre-push]
      - id: check-json
        stages: [pre-commit, pre-push]
      - id: check-merge-conflict
        stages: [pre-commit, pre-push]
      - id: check-added-large-files
        stages: [pre-commit, pre-push]
        args: ["--maxkb=500"]
      - id: detect-private-key
        stages: [pre-commit, pre-push]
      - id: no-commit-to-branch
        stages: [pre-commit, pre-push]
        args: ["--branch", "main", "--branch", "master"]

  # --- Commit Message Conventions ---
  - repo: https://github.com/compilerla/conventional-pre-commit
    rev: v4.0.0
    hooks:
      - id: conventional-pre-commit
        stages: [commit-msg]
        args: [feat, fix, docs, style, refactor, perf, test, chore, ci, build, revert]

  # --- Secret Detection ---
  - repo: https://github.com/Yelp/detect-secrets
    rev: v1.4.0
    hooks:
      - id: detect-secrets
        stages: [pre-commit, pre-push]
        args: ["--baseline", ".secrets.baseline"]
```

### Language-Specific Additions

#### Rust (heliosCLI, phenotype-infrakit)

Add to `.pre-commit-config.yaml`:

```yaml
  # --- Rust: Format + Lint ---
  - repo: local
    hooks:
      - id: rustfmt
        name: rustfmt
        entry: bash -c 'cargo fmt --check'
        language: system
        files: '\.rs$'
        pass_filenames: false
        stages: [pre-commit, pre-push]

      - id: clippy
        name: clippy
        entry: bash -c 'cargo clippy --all-targets --all-features -- -D warnings'
        language: system
        files: '\.rs$'
        pass_filenames: false
        stages: [pre-push]
```

**Configuration Files:**

- **`rustfmt.toml`** (repo root):
  ```toml
  edition = "2021"
  max_width = 100
  hard_tabs = false
  tab_spaces = 4
  comment_width = 80
  wrap_comments = true
  normalize_comments = true
  format_strings = true
  format_code_in_doc_comments = true
  ```

- **`Cargo.toml` clippy config** (workspace section):
  ```toml
  [lints.clippy]
  all = "warn"
  pedantic = "warn"
  nursery = "warn"
  correctness = "deny"
  suspicious = "deny"
  complexity = "warn"
  perf = "warn"
  style = "warn"

  # Lint groups disabled (adjust per project)
  cast_possible_truncation = "allow"
  cast_possible_wrap = "allow"
  module_name_repetitions = "allow"
  ```

#### Python (heliosCLI, platforms/thegent)

Add to `.pre-commit-config.yaml`:

```yaml
  # --- Python: Format + Lint ---
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.10.0
    hooks:
      - id: ruff
        stages: [pre-commit, pre-push]
        args: ["--fix", "--show-fixes"]
      - id: ruff-format
        stages: [pre-commit, pre-push]

  # --- Python: Type Checking (pre-push only, slower) ---
  - repo: https://github.com/pre-commit/mirrors-mypy
    rev: v1.13.0
    hooks:
      - id: mypy
        stages: [pre-push]
        additional_dependencies:
          - "types-all"
          - "pydantic"
          - "structlog"
        args: ["--strict", "--show-error-codes", "--ignore-missing-imports"]
        exclude: "^(tests|scripts|benchmarks)/"
```

**Configuration File: `pyproject.toml`**

```toml
[tool.ruff]
line-length = 100
target-version = "py311"
include = ["src/**/*.py", "tests/**/*.py"]
exclude = ["venv", ".venv", "build", "dist", "__pycache__"]

[tool.ruff.lint]
select = [
  "E",    # pycodestyle errors
  "W",    # pycodestyle warnings
  "F",    # Pyflakes
  "I",    # isort
  "N",    # pep8-naming
  "UP",   # pyupgrade
  "B",    # flake8-bugbear
  "C4",   # flake8-comprehensions
  "ARG",  # flake8-unused-arguments
  "RUF",  # Ruff-specific rules
  "SIM",  # flake8-simplify
]
ignore = [
  "E501",  # line-length (managed by formatter)
  "W503",  # line break before binary operator
]

[tool.ruff.lint.per-file-ignores]
"__init__.py" = ["F401"]  # unused imports in __init__
"tests/**" = ["ARG", "SIM", "N"]  # test pragmatism

[tool.ruff.format]
quote-style = "double"
indent-style = "space"
skip-magic-trailing-comma = false

[tool.black]
line-length = 100
target-version = ["py311"]
include = '\.pyi?$'
exclude = '''
/(
  \.git
  | \.hg
  | \.mypy_cache
  | \.tox
  | \.venv
  | build
  | dist
)/
'''

[tool.mypy]
python_version = "3.11"
warn_return_any = true
warn_unused_configs = true
disallow_untyped_defs = true
disallow_incomplete_defs = true
check_untyped_defs = true
no_implicit_optional = true
strict_equality = true
strict_optional = true
```

#### Go (platforms/thegent)

Add to `.pre-commit-config.yaml`:

```yaml
  # --- Go: Format + Lint ---
  - repo: https://github.com/golangci/pre-commit-hooks
    rev: v1.0.3
    hooks:
      - id: golangci-lint
        stages: [pre-commit, pre-push]
        args: ["run", "--new-from-rev", "HEAD"]

  # --- Go: Format ---
  - repo: local
    hooks:
      - id: gofmt
        name: gofmt
        entry: bash -c 'gofmt -w -l .'
        language: system
        files: '\.go$'
        pass_filenames: false
        stages: [pre-commit, pre-push]
```

**Configuration File: `.golangci.yml`**

```yaml
run:
  timeout: 5m
  modules-download-mode: readonly
  build-tags:
    - integration

output:
  format: colored-line-number

linters-settings:
  revive:
    rules:
      - name: exported
        arguments: ["checkPrivateReceivers", "checkTypeParams"]
  gocyclo:
    min-complexity: 12
  govet:
    enable-all: true
  golint:
    min-confidence: 0.8
  gofmt:
    simplify: true

linters:
  disable-all: true
  enable:
    - revive
    - gocyclo
    - govet
    - golint
    - gofmt
    - errcheck
    - gosimple
    - ineffassign
    - misspell
    - staticcheck
    - typecheck
    - unused
    - varcheck

issues:
  exclude-rules:
    - path: "_test.go$"
      linters:
        - revive
        - govet
    - path: "testdata/"
      linters:
        - revive
```

---

## Layer 2: Local Quality Gate Script

The quality gate runs all checks locally before push. This catches issues **before** CI.

**File: `scripts/quality-gate.sh`**

```bash
#!/usr/bin/env bash
set -euo pipefail

# Quality Gate v1: Lint, Format, Type Check
# Run this before `git push` to catch issues locally

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 Quality Gate: Lint + Format + Type Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

FAILED=0

# Detect languages
HAS_RUST=false
HAS_PYTHON=false
HAS_GO=false

[ -f "Cargo.toml" ] && HAS_RUST=true
[ -f "pyproject.toml" ] || [ -f "requirements.txt" ] && HAS_PYTHON=true
[ -f "go.mod" ] && HAS_GO=true

# --- RUST ---
if [ "$HAS_RUST" = true ]; then
  echo "📦 Rust: cargo fmt + clippy"
  if ! cargo fmt --check; then
    echo "  ❌ Format check failed. Run: cargo fmt"
    FAILED=1
  else
    echo "  ✅ Format check passed"
  fi

  if ! cargo clippy --all-targets -- -D warnings; then
    echo "  ❌ Clippy check failed"
    FAILED=1
  else
    echo "  ✅ Clippy check passed"
  fi
  echo ""
fi

# --- PYTHON ---
if [ "$HAS_PYTHON" = true ]; then
  echo "🐍 Python: ruff + black + mypy"

  if ! ruff check . --fix; then
    echo "  ❌ Ruff check failed"
    FAILED=1
  else
    echo "  ✅ Ruff check passed"
  fi

  if ! ruff format --check .; then
    echo "  ❌ Ruff format check failed. Run: ruff format ."
    FAILED=1
  else
    echo "  ✅ Ruff format check passed"
  fi

  if command -v mypy &>/dev/null; then
    if ! mypy src/ tests/ --strict --ignore-missing-imports; then
      echo "  ⚠️  mypy check failed (non-blocking)"
    else
      echo "  ✅ mypy type check passed"
    fi
  fi
  echo ""
fi

# --- GO ---
if [ "$HAS_GO" = true ]; then
  echo "🐹 Go: gofmt + golangci-lint"

  if ! gofmt -l . | grep -q ""; then
    echo "  ✅ gofmt passed"
  else
    echo "  ❌ gofmt check failed. Run: gofmt -w ."
    FAILED=1
  fi

  if command -v golangci-lint &>/dev/null; then
    if ! golangci-lint run ./...; then
      echo "  ❌ golangci-lint failed"
      FAILED=1
    else
      echo "  ✅ golangci-lint passed"
    fi
  fi
  echo ""
fi

# --- SUMMARY ---
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $FAILED -eq 0 ]; then
  echo "✅ All quality checks passed!"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  exit 0
else
  echo "❌ Quality gate failed. Fix errors above and try again."
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  exit 1
fi
```

**Setup:**

```bash
# Make executable
chmod +x scripts/quality-gate.sh

# Run before push
./scripts/quality-gate.sh

# Optional: Add to pre-push hook for automation
echo "#!/bin/sh\n./scripts/quality-gate.sh" > .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```

---

## Layer 3: GitHub Actions Workflow

CI/CD validation for PRs and merges.

**File: `.github/workflows/quality-gate.yml`**

```yaml
name: Quality Gate

on:
  pull_request:
    types: [opened, synchronize, reopened]
  push:
    branches: [main, master]

jobs:
  lint-and-format:
    name: Lint & Format Check
    runs-on: ubuntu-latest
    timeout-minutes: 10

    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      # --- Rust ---
      - name: Install Rust
        if: hashFiles('Cargo.toml') != ''
        uses: dtolnay/rust-toolchain@stable

      - name: Rust: Format Check
        if: hashFiles('Cargo.toml') != ''
        run: cargo fmt --check

      - name: Rust: Clippy
        if: hashFiles('Cargo.toml') != ''
        run: cargo clippy --all-targets -- -D warnings

      # --- Python ---
      - name: Set up Python
        if: hashFiles('pyproject.toml') != '' || hashFiles('requirements.txt') != ''
        uses: actions/setup-python@v5
        with:
          python-version: '3.11'

      - name: Python: Install Ruff
        if: hashFiles('pyproject.toml') != '' || hashFiles('requirements.txt') != ''
        run: pip install ruff

      - name: Python: Ruff Check
        if: hashFiles('pyproject.toml') != '' || hashFiles('requirements.txt') != ''
        run: ruff check .

      - name: Python: Ruff Format
        if: hashFiles('pyproject.toml') != '' || hashFiles('requirements.txt') != ''
        run: ruff format --check .

      # --- Go ---
      - name: Set up Go
        if: hashFiles('go.mod') != ''
        uses: actions/setup-go@v5
        with:
          go-version: '^1.22'

      - name: Go: Format Check
        if: hashFiles('go.mod') != ''
        run: |
          if [ -n "$(gofmt -l .)" ]; then
            echo "gofmt check failed"
            exit 1
          fi

      - name: Go: golangci-lint
        if: hashFiles('go.mod') != ''
        uses: golangci/golangci-lint-action@v4
        with:
          version: latest

  type-check:
    name: Type Check
    runs-on: ubuntu-latest
    timeout-minutes: 15

    steps:
      - uses: actions/checkout@v4

      - name: Set up Python
        if: hashFiles('pyproject.toml') != ''
        uses: actions/setup-python@v5
        with:
          python-version: '3.11'

      - name: Install deps (Python)
        if: hashFiles('pyproject.toml') != ''
        run: pip install mypy ruff

      - name: Python: mypy
        if: hashFiles('pyproject.toml') != ''
        run: mypy src/ tests/ --strict --ignore-missing-imports || true

      - name: Install Rust
        if: hashFiles('Cargo.toml') != ''
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rust-src

      - name: Rust: Type Check (cargo check)
        if: hashFiles('Cargo.toml') != ''
        run: cargo check --all

  summary:
    name: Quality Gate Summary
    runs-on: ubuntu-latest
    if: always()
    needs: [lint-and-format, type-check]
    steps:
      - name: Check job status
        run: |
          if [[ "${{ needs.lint-and-format.result }}" == "failure" || "${{ needs.type-check.result }}" == "failure" ]]; then
            echo "❌ Quality gate failed"
            exit 1
          fi
          echo "✅ Quality gate passed"
```

---

## Layer 4: Branch Protection & Merge Enforcement

Configure branch protection in GitHub:

1. **Go to**: Settings → Branches → Add rule
2. **Pattern**: `main` (or `master`)
3. **Require status checks**:
   - `quality-gate / lint-and-format`
   - `quality-gate / type-check`
4. **Require branches to be up to date before merging**: ✅
5. **Require code reviews**: ✅ (2 for critical, 1 for standard)
6. **Require conversation resolution**: ✅

---

## Developer Setup Guide

### Quick Start (Per Repo)

```bash
# 1. Clone repo (if not done)
git clone <repo-url>
cd <repo-name>

# 2. Install pre-commit hooks
pip install pre-commit
pre-commit install
pre-commit install --hook-type pre-push

# 3. Run quality gate locally (optional, auto-runs on commit)
./scripts/quality-gate.sh

# 4. Make changes and commit (hooks run automatically)
git add .
git commit -m "feat: add new feature"  # hooks run here

# 5. Before push, quality gate runs (pre-push hooks)
# If all pass, push succeeds
git push origin <branch>
```

### Suppressing Warnings (With Justification)

**When to suppress:**
- Pre-existing violations (already in codebase, inherited from main)
- Tool false positives
- Performance-critical code requiring unsafe constructs
- Framework-specific patterns (e.g., PyO3 macros)

**How to suppress:**

#### Rust:
```rust
#[allow(clippy::cast_possible_truncation)]  // JUSTIFICATION: value guaranteed <256
let byte = value as u8;
```

#### Python:
```python
# noqa: E501  # JUSTIFICATION: URL too long
url = "https://example.com/very/long/url/that/exceeds/line/length"
```

#### Go:
```go
// nolint:gosec  // JUSTIFICATION: crypto/rand for non-cryptographic use
```

**Document suppressions:**
- Include why in a comment
- Create an issue for future cleanup
- Run: `./scripts/quality-gate.sh --check-suppressions` to audit

---

## Troubleshooting

### Hooks Not Running

```bash
# Reinstall hooks
pre-commit install
pre-commit install --hook-type pre-push

# Test hook execution
pre-commit run --all-files
```

### Formatting Conflicts

```bash
# Auto-fix formatting
cargo fmt
ruff format .
gofmt -w .
black src/ tests/

# Then re-commit
git add .
git commit --amend --no-edit
```

### Slow Hook Execution

Hooks that timeout? Adjust `.pre-commit-config.yaml`:

```yaml
  - id: clippy
    stages: [pre-push]  # Move to pre-push instead of pre-commit
    timeout: 60  # Increase timeout
```

### Skipping Hooks (Last Resort)

```bash
git commit --no-verify  # Bypasses pre-commit hooks
git push --no-verify    # Bypasses pre-push hooks
```

**⚠️ Never bypass on main/master. Use only for emergency hotfixes on feature branches.**

---

## Continuous Improvement

### Regular Audits

Run quarterly to identify patterns and opportunities:

```bash
# Count suppressions
grep -r "noqa:\|allow(\|nolint:" . --include="*.rs" --include="*.py" --include="*.go" | wc -l

# Trend violations
pre-commit run --all-files --verbose
```

### Upgrading Tools

```bash
# Update pre-commit frameworks
pre-commit autoupdate

# Update Rust toolchain
rustup update stable

# Update Python tools
pip install --upgrade ruff black mypy

# Update Go tools
go get -u github.com/golangci/golangci-lint
```

---

## Summary Table

| Layer | Trigger | Tools | When | Purpose |
|-------|---------|-------|------|---------|
| **1: Pre-Commit** | Before commit | rustfmt, clippy, ruff, gofmt | Local | Fast feedback loop |
| **2: Quality Gate** | Before push | Layer 1 + type checks | Local | Comprehensive validation |
| **3: GitHub Actions** | PR/push | All tools + coverage | CI/CD | Final validation |
| **4: Branch Protection** | Before merge | All CI + reviews | Merge gate | Quality assurance |

---

## References

- **Rust**: [Clippy](https://github.com/rust-lang/rust-clippy), [rustfmt](https://github.com/rust-lang/rustfmt)
- **Python**: [Ruff](https://github.com/astral-sh/ruff), [Black](https://github.com/psf/black), [mypy](https://github.com/python/mypy)
- **Go**: [golangci-lint](https://golangci-lint.run/), [gofmt](https://golang.org/cmd/gofmt/)
- **General**: [pre-commit framework](https://pre-commit.com/)

