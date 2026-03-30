# Linter Configuration Templates

This directory contains canonical linter configuration templates for the Phenotype ecosystem. All projects should use these templates as their foundation for code quality enforcement.

## Quick Start

```bash
# Copy language-specific linter config
cp path/to/thegent/templates/linters/python/ruff.toml my-project/ruff.toml
cp path/to/thegent/templates/linters/rust/clippy.toml my-project/.clippy.toml
cp path/to/thegent/templates/linters/typescript/eslint.config.js my-project/eslint.config.js

# Customize for your project (usually minimal changes needed)
# See customization guide below

# Test the config
cd my-project
ruff check .           # Python
cargo clippy .         # Rust
eslint .               # TypeScript
```

## Template Organization

```
templates/linters/
├── README.md                    # This file
├── python/
│   ├── ruff.toml               # Ruff linter + formatter config
│   ├── basedpyright.json       # Type checker config
│   └── README.md               # Python-specific guide
├── rust/
│   ├── clippy.toml             # Clippy linter config
│   ├── rustfmt.toml            # Formatter config
│   └── README.md               # Rust-specific guide
├── typescript/
│   ├── eslint.config.js        # ESLint config (flat format)
│   ├── prettier.config.js      # Prettier formatter config
│   └── README.md               # TypeScript-specific guide
├── go/
│   ├── .golangci.yml           # golangci-lint config
│   └── README.md               # Go-specific guide
└── shared/
    ├── vale.yaml               # Markdown linting
    ├── .typos.toml             # Spell checking
    └── README.md               # Shared tools guide
```

---

## Python Linting

### Files
- **ruff.toml** — Combined linter + formatter config (all rules enabled, strict)
- **basedpyright.json** — Type checker config (strict mode)
- **pyproject.toml** (alt) — Ruff config can go in pyproject.toml under `[tool.ruff]`

### Quick Setup

```bash
# Option 1: Copy as standalone
cp templates/linters/python/ruff.toml my-project/ruff.toml

# Option 2: Copy into pyproject.toml [tool.ruff] section
cat templates/linters/python/ruff.toml >> my-project/pyproject.toml

# Install and test
pip install ruff basedpyright
cd my-project
ruff check .             # Run linter
ruff format .            # Auto-format
basedpyright .           # Type check
```

### Key Settings

| Setting | Value | Purpose |
|---------|-------|---------|
| line-length | 120 | Python line length limit |
| target-version | py312 | Target Python 3.12+ |
| select | (all rules) | Enable comprehensive rule set |
| mccabe max-complexity | 10 | Cyclomatic complexity limit |
| pydocstyle convention | google | Google-style docstrings |

### Customization

**For smaller projects** (reduce rules):
```toml
# Remove from 'select' list
# - "D" (docstrings) if not enforcing docs
# - "ANN" (type annotations) if not strict typing
```

**For data science projects** (add pandas rules):
```toml
[lint]
select = [..., "PD"]  # pandas-vet rules
```

**For test files** (ignore specific rules):
```toml
[lint.per-file-ignores]
"tests/**/*.py" = ["S101", "ANN", "D"]  # Ignore docstrings, type hints in tests
"conftest.py" = ["S101", "ANN", "D"]
```

See `templates/linters/python/README.md` for detailed customization.

---

## Rust Linting

### Files
- **clippy.toml** — Clippy linter config (strict, warnings-as-errors)
- **rustfmt.toml** — Formatter config
- **.cargo/config.toml** (alt) — Can store clippy config here

### Quick Setup

```bash
# Copy clippy config
cp templates/linters/rust/clippy.toml my-project/.clippy.toml

# Install clippy (comes with rustup)
rustup update
cargo update

# Test
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
```

### Key Settings

| Setting | Value | Purpose |
|---------|-------|---------|
| disallow | [unwrap, expect, todo, panic] | Catch unsafe patterns |
| cognitive-complexity | warn | Catch overly complex functions |
| type-complexity | warn | Catch over-complex type signatures |
| type-complexity-threshold | 250 | Type complexity limit |
| msrv | 1.70 | Minimum supported Rust version |

### Customization

**For WASM projects** (stricter):
```toml
[clippy]
disallow = [
    "std_instead_of_core",
    "std_instead_of_alloc",
]
```

**For unsafe code** (when necessary):
```toml
# In specific modules:
#[allow(unsafe_code)]
mod safe_wrapper {
    // Unsafe code here
}
```

**For performance-critical code** (relaxed):
```toml
# Per-file via
#![allow(clippy::module_name_repetitions)]
```

See `templates/linters/rust/README.md` for detailed customization.

---

## TypeScript/JavaScript Linting

### Files
- **eslint.config.js** — ESLint config (flat config format, modern)
- **prettier.config.js** — Prettier formatter config
- **.eslintrc.json** (legacy, not recommended)

### Quick Setup

```bash
# Copy configs
cp templates/linters/typescript/eslint.config.js my-project/
cp templates/linters/typescript/prettier.config.js my-project/

# Install tools
npm install --save-dev eslint prettier typescript oxlint

# Test
oxlint .              # Fast linting (Rust-based)
eslint . --fix        # ESLint with fixes
prettier --write .    # Format code
```

### Key Settings

| Setting | Value | Purpose |
|---------|-------|---------|
| lang | ts | TypeScript support enabled |
| rules.strict | error | Require 'use strict' |
| parserOptions.ecmaVersion | 2024 | Latest ECMAScript |
| semi | true | Require semicolons |
| quotes | single | Single quotes |
| trailingComma | all | Trailing commas in multiline |

### Customization

**For React projects** (add React rules):
```javascript
import react from 'eslint-plugin-react';
export default [
  {
    settings: { react: { version: 'detect' } },
    rules: {
      'react/react-in-jsx-scope': 'off',  // React 17+
      'react/prop-types': 'warn',         // PropTypes optional
    },
  },
];
```

**For strict typing** (enforce strict checks):
```javascript
{
  languageOptions: { parserOptions: { strict: true } },
  rules: {
    '@typescript-eslint/strict-boolean-expressions': 'error',
    '@typescript-eslint/no-explicit-any': 'error',
  },
}
```

See `templates/linters/typescript/README.md` for detailed customization.

---

## Go Linting

### Files
- **.golangci.yml** — golangci-lint config (comprehensive)
- **.gofumpt.yaml** (optional) — gofumpt formatter config

### Quick Setup

```bash
# Copy golangci config
cp templates/linters/go/.golangci.yml my-project/

# Install golangci-lint
go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest

# Test
golangci-lint run
go fmt ./...
gofumpt -w .
```

### Key Linters

| Linter | Purpose |
|--------|---------|
| govet | Go vet (built-in) |
| errcheck | Error checking |
| staticcheck | Static analysis (Dominik Honnef) |
| gosec | Security checks |
| gocyclo | Cyclomatic complexity |
| golint | Style |
| gofmt | Formatting |

### Customization

**For strict projects**:
```yaml
linters:
  disable-all: true
  enable:
    - govet
    - errcheck
    - staticcheck
    - gosec
    - ineffassign
    - misspell
    - vet
```

**For lenient projects**:
```yaml
linters:
  enable-all: true
  disable:
    - exhaustruct
    - execinquery
    - nonamedreturns
```

See `templates/linters/go/README.md` for detailed customization.

---

## Shared Tools

### Vale (Markdown Linting)

**File**: `templates/linters/shared/vale.yaml`

Enforces prose style, grammar, and technical writing standards.

```bash
# Install
brew install vale  # macOS
apt install vale   # Linux
choco install vale # Windows

# Test
vale docs/
```

**Key Checks**:
- Grammar and spell checking
- Sentence length
- Passive voice detection
- Technical terminology validation

### Typos (Spell Checking)

**File**: `templates/linters/shared/.typos.toml`

Catches common spelling mistakes across all file types.

```bash
# Install
brew install typos  # macOS
cargo install typos # Rust

# Test
typos .
```

**Features**:
- Fast spell checking
- Exclude patterns support
- Custom dictionary support
- Binary-safe (doesn't flag code)

---

## Linter Priority & When to Use

### Primary Linters (use in all projects)

1. **Syntax Validators** (pre-commit stage)
   - ruff (Python)
   - rustfmt + clippy (Rust)
   - prettier + oxlint (TypeScript)
   - gofmt (Go)

2. **Type Checkers** (pre-push stage)
   - basedpyright (Python)
   - tsc (TypeScript)
   - cargo check (Rust)

### Secondary Linters (use as applicable)

3. **Security Scanners**
   - gosec (Go)
   - semgrep (all languages, optional)

4. **Documentation**
   - vale (Markdown/prose)
   - typos (spell checking)

---

## Integration with Pre-commit

Include linter configs in `.pre-commit-config.yaml`:

```yaml
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.10.0
    hooks:
      - id: ruff
        args: ["--fix"]
      - id: ruff-format

  - repo: local
    hooks:
      - id: clippy
        entry: cargo clippy --all-targets -- -D warnings
        language: system
        files: '\.rs$'
```

---

## Integration with Taskfile

```yaml
version: '3'

tasks:
  lint:
    desc: Run all linters
    cmds:
      - ruff check . --fix
      - basedpyright .
      - cargo clippy --all-targets
      - oxlint . --fix

  format:
    desc: Auto-format code
    cmds:
      - ruff format .
      - cargo fmt
      - prettier --write .

  quality:
    desc: Full quality check
    cmds:
      - task: lint
      - task: format
      - cargo test
      - pytest tests/
```

---

## Testing Linter Configs

### Verify Config Syntax

```bash
# Python
ruff check --select ALL .

# Rust
cargo clippy --explain E0001

# TypeScript
eslint --print-config tsconfig.json

# Go
golangci-lint config
```

### Run on Sample Code

```bash
# Create test file
echo 'x=1; y  =2' > test.py  # Bad spacing

# Run linter
ruff check test.py

# Should report issues
```

### Check for Rule Conflicts

```bash
# Python: ruff format vs ruff check
ruff format test.py
ruff check test.py
# Should have no conflicts

# TypeScript: prettier vs eslint
prettier test.ts
eslint test.ts --fix
# Should be idempotent
```

---

## Performance Tips

### Cache Results

```bash
# Ruff caches automatically
# Clear cache if needed:
rm -rf .ruff_cache

# Rust: incremental compilation
export CARGO_BUILD_INCREMENTAL=1

# golangci-lint: cache enabled by default
golangci-lint cache clean
```

### Run on Changed Files Only

```bash
# Git diff to changed files
git diff --name-only main | xargs ruff check

# Or use pre-commit (runs only on staged)
pre-commit run --hook-stage pre-push
```

### Parallel Execution

```bash
# Ruff is single-threaded, but fast
# For monorepos, run per-package:
for pkg in packages/*/; do
  (cd "$pkg" && ruff check .)
done
```

---

## Troubleshooting

### Issue: Linter says OK, but CI fails

**Solution**: Version mismatch. Pin versions:

```yaml
# .pre-commit-config.yaml
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.10.0  # Explicit version
```

```bash
# pyproject.toml
[tool.ruff]
required-version = "0.10.0"
```

### Issue: Formatter conflicts with linter

**Solution**: Use formatter-aware linter rules:

```toml
# ruff.toml
ignore = [
    "E501",    # Line too long (let formatter handle)
    "COM812",  # Trailing comma (formatter conflict)
    "ISC001",  # String concat (formatter conflict)
]
```

### Issue: False positives in tests

**Solution**: Add per-file ignores:

```toml
[lint.per-file-ignores]
"tests/**/*.py" = ["S101", "D"]  # Ignore assert warnings in tests
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

- **Pre-commit Configuration**: `dotfiles/hooks/.pre-commit-config.base.yaml`
- **Quality Gates**: `templates/quality/quality-gate.base.sh`
- **CLAUDE.md**: `dotfiles/governance/CLAUDE.base.md`

---

## Template Info

- **Location**: `thegent/templates/linters/`
- **Canonical URL**: See thegent repository
- **Maintained By**: Phenotype Governance Team
- **Last Updated**: 2026-03-29
