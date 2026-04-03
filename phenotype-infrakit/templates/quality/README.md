# Quality Gates — Consolidated Templates

This directory contains canonical quality gate scripts and configuration templates for the Phenotype ecosystem. These templates provide standardized approaches to testing, linting, type checking, and security validation.

## Quick Start

```bash
# Copy the base quality gate script
cp path/to/thegent/templates/quality/quality-gate.base.sh my-project/hooks/quality-gate.sh
chmod +x my-project/hooks/quality-gate.sh

# Customize for your project
# - Uncomment language-specific gates for your tech stack
# - Adjust thresholds and paths
# - Add project-specific checks

# Test the script
cd my-project
./hooks/quality-gate.sh
```

## Files in This Directory

### quality-gate.base.sh
**Canonical base quality gate script** — Template for all Phenotype projects.

**Includes**:
- 8 quality gates covering syntax, linting, type checking, testing, security, coverage, traceability, and documentation
- Language-specific implementations (Python, Rust, TypeScript, Go)
- Modular gate functions that can be run individually
- Summary reporting with error/warning tracking
- Colored output for readability

**How to Use**:
1. Copy to your project: `cp quality-gate.base.sh my-project/hooks/quality-gate.sh`
2. Customize sections marked `[CUSTOMIZE]` for your languages
3. Make executable: `chmod +x hooks/quality-gate.sh`
4. Run: `./hooks/quality-gate.sh`
5. Integrate with Taskfile or CI/CD

**Language-Specific Gates**:

| Language | Gate 2 | Gate 3 | Gate 4 | Notes |
|----------|--------|--------|--------|-------|
| Python | ruff | basedpyright | pytest | Include all three |
| Rust | rustfmt + clippy | cargo check | cargo test | Include all three |
| TypeScript | prettier + oxlint | tsc | vitest/jest | Include all three |
| Go | gofmt | cargo check | go test | Include all three |

---

## The 8 Quality Gates

### Gate 1: Syntax & Format Validation
Checks basic syntax correctness of all file formats.

**Checks**:
- YAML validation (yq)
- JSON validation
- TOML validation
- Custom format checks

**Stage**: pre-commit

```bash
./quality-gate.sh gate1
```

### Gate 2: Linting & Formatting
Enforces code style and catches common mistakes.

**Tools** (language-dependent):
- Python: `ruff check` + `ruff format`
- Rust: `rustfmt` + `cargo clippy`
- TypeScript: `oxlint` + `prettier`
- Go: `gofmt` + `golangci-lint`

**Stage**: pre-commit

```bash
./quality-gate.sh gate2
```

### Gate 3: Type Checking
Validates static types and catches type-related bugs.

**Tools** (language-dependent):
- Python: `basedpyright` (strict mode)
- TypeScript: `tsc --noEmit`
- Rust: `cargo check`
- Go: `go vet`

**Stage**: pre-push

```bash
./quality-gate.sh gate3
```

### Gate 4: Testing
Runs test suite and ensures coverage.

**Tools** (language-dependent):
- Python: `pytest`
- Rust: `cargo test`
- TypeScript: `vitest` or `jest`
- Go: `go test`

**Stage**: pre-push

```bash
./quality-gate.sh gate4
```

### Gate 5: Security & Secrets
Detects secrets, vulnerabilities, and security issues.

**Tools**:
- Secret detection: `trufflehog`
- Dependency audit: `pip-audit` (Python), `cargo audit` (Rust), `npm audit` (JS)
- SAST: `semgrep` (optional)

**Stage**: pre-push

```bash
./quality-gate.sh gate5
```

### Gate 6: Code Coverage
Ensures test coverage meets threshold.

**Thresholds**:
- Default: 80% line coverage
- Configurable per project

**Tools** (language-dependent):
- Python: `coverage.py`
- Rust: `tarpaulin`
- TypeScript: `vitest` coverage
- Go: `go test -cover`

**Stage**: pre-push (optional)

```bash
./quality-gate.sh gate6
```

### Gate 7: Specification Traceability
Verifies all functional requirements have corresponding tests.

**Checks**:
- FUNCTIONAL_REQUIREMENTS.md exists
- All FR-{PROJECT}-XXX entries have tests
- All tests reference an FR ID

**Stage**: pre-push

```bash
./quality-gate.sh gate7
```

### Gate 8: Documentation
Validates documentation quality and completeness.

**Tools**:
- Markdown linting: `vale` (grammar/style)
- Documentation presence: README.md, ADR.md, etc.
- Links validation (optional)

**Stage**: pre-push

```bash
./quality-gate.sh gate8
```

---

## Using Individual Gates

You can run individual gates for faster iteration during development:

```bash
# Run only linting (useful during development)
./quality-gate.sh gate2

# Run only tests
./quality-gate.sh gate4

# Run only security checks
./quality-gate.sh gate5

# Run all gates (default, useful for CI/CD)
./quality-gate.sh

# Show help
./quality-gate.sh help
```

---

## Integration with Taskfile

Create a `Taskfile.yml` with quality gate tasks:

```yaml
version: '3'

tasks:
  quality:
    desc: Run quality gates (all)
    cmds:
      - ./hooks/quality-gate.sh

  quality:gate1:
    desc: Run Gate 1 (syntax validation)
    cmds:
      - ./hooks/quality-gate.sh gate1

  quality:gate2:
    desc: Run Gate 2 (linting)
    cmds:
      - ./hooks/quality-gate.sh gate2

  quality:gate3:
    desc: Run Gate 3 (type checking)
    cmds:
      - ./hooks/quality-gate.sh gate3

  quality:gate4:
    desc: Run Gate 4 (tests)
    cmds:
      - ./hooks/quality-gate.sh gate4

  quality:gate5:
    desc: Run Gate 5 (security)
    cmds:
      - ./hooks/quality-gate.sh gate5

  quality:full:
    desc: Run all gates + format checking
    cmds:
      - ./hooks/quality-gate.sh
      - ruff format --check .
```

Then run:
```bash
task quality         # Run all gates
task quality:gate2   # Run linting only
task quality:full    # Include format checks
```

---

## Integration with GitHub Actions

Create `.github/workflows/quality.yml`:

```yaml
name: Quality Gates

on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ['3.11', '3.12']
    steps:
      - uses: actions/checkout@v3

      - uses: actions/setup-python@v4
        with:
          python-version: ${{ matrix.python-version }}

      - name: Install dependencies
        run: |
          python -m pip install --upgrade pip
          pip install -r requirements.txt
          pip install pytest coverage basedpyright ruff

      - name: Run quality gates
        run: ./hooks/quality-gate.sh
```

---

## Customization Guide

### For Python Projects

```bash
#!/usr/bin/env bash
# hooks/quality-gate.sh (customized for Python)

# Uncomment and customize these gates:
gate2_linting() {
  # ruff linting
  ruff check . --fix
  ruff format .
}

gate3_type_checking() {
  # basedpyright
  basedpyright .
}

gate4_testing() {
  # pytest with coverage
  pytest tests/ -v --cov=src --cov-report=term-missing
}

gate6_coverage() {
  # Coverage threshold check
  coverage run -m pytest tests/
  coverage report --fail-under=80
}
```

### For Rust Projects

```bash
#!/usr/bin/env bash
# hooks/quality-gate.sh (customized for Rust)

gate2_linting() {
  # rustfmt + clippy
  cargo fmt -- --check
  cargo clippy --all-targets -- -D warnings
}

gate3_type_checking() {
  # cargo check
  cargo check --all-targets
}

gate4_testing() {
  # cargo test
  cargo test --all
}

gate5_security() {
  # cargo audit
  cargo audit
}
```

### For TypeScript Projects

```bash
#!/usr/bin/env bash
# hooks/quality-gate.sh (customized for TypeScript)

gate2_linting() {
  # oxlint + prettier
  oxlint . --fix
  prettier --write .
}

gate3_type_checking() {
  # tsc
  tsc --noEmit
}

gate4_testing() {
  # vitest
  vitest run
}
```

### For Monorepos (Multiple Languages)

```bash
#!/usr/bin/env bash
# hooks/quality-gate.sh (monorepo example)

gate2_linting() {
  # Python
  if [[ -d "python/" ]]; then
    (cd python && ruff check . --fix && ruff format .)
  fi

  # Rust
  if [[ -d "rust/" ]]; then
    (cd rust && cargo fmt -- --check && cargo clippy --all-targets -- -D warnings)
  fi

  # TypeScript
  if [[ -d "packages/" ]]; then
    (cd packages && oxlint . --fix && prettier --write .)
  fi
}

gate4_testing() {
  # Python
  if [[ -d "python/" ]]; then
    (cd python && pytest tests/ -v)
  fi

  # Rust
  if [[ -d "rust/" ]]; then
    (cd rust && cargo test --all)
  fi

  # TypeScript
  if [[ -d "packages/" ]]; then
    (cd packages && vitest run)
  fi
}
```

---

## Common Customizations

### Adjust Coverage Threshold

```bash
# In gate6_coverage()
coverage_min=85  # Change from default 80
```

### Add Custom Language Check

```bash
# In gate2_linting()
# Add custom linter for your language
if command_exists custom-linter; then
  custom-linter . --fix
fi
```

### Skip Optional Gates

```bash
# Remove functions for gates you don't use
# E.g., delete gate6_coverage() if not tracking coverage
```

### Add Project-Specific Checks

```bash
# Add custom gate for your project
gate9_custom_checks() {
  log_info "=== GATE 9: Custom Checks ==="

  # Custom architecture validation
  if command_exists import-linter; then
    import-linter check
  fi

  # Custom database schema check
  if [[ -d "schema/" ]]; then
    validate_schema schema/
  fi
}

# Call in main()
gate9_custom_checks
```

---

## Troubleshooting

### Issue: Script not found
```bash
chmod +x hooks/quality-gate.sh
```

### Issue: Tool not installed
```bash
# Install missing tools
pip install ruff basedpyright pytest coverage

# For Rust
cargo install --locked cargo-clippy

# For TypeScript
npm install --save-dev oxlint prettier typescript
```

### Issue: Script hangs on tests
```bash
# Run specific gate to identify slow check
./hooks/quality-gate.sh gate4

# Add timeout if needed
timeout 300 pytest tests/  # 5-minute timeout
```

### Issue: False positives in security checks
```bash
# Exclude test fixtures in gate5_security()
if [[ "$file" == tests/fixtures/* ]]; then
  continue
fi
```

---

## Testing Your Quality Gate Script

```bash
# Run all gates
./hooks/quality-gate.sh

# Run with verbose output
bash -x ./hooks/quality-gate.sh

# Run specific gate
./hooks/quality-gate.sh gate2

# Show help
./hooks/quality-gate.sh help
```

---

## Related Templates

- **Pre-commit Configuration**: `dotfiles/hooks/.pre-commit-config.base.yaml`
- **CLAUDE.md**: `dotfiles/governance/CLAUDE.base.md`
- **Linter Configs**: `templates/<language>/` (ruff.toml, clippy.toml, etc.)
- **Language Templates**: `templates/python/`, `templates/rust/`, `templates/typescript/`

---

## Integration Strategy

### For New Projects

1. Copy `quality-gate.base.sh` to `hooks/quality-gate.sh`
2. Customize language-specific gates
3. Create `Taskfile.yml` with quality tasks
4. Integrate with `.pre-commit-config.yaml`
5. Add to GitHub Actions workflows

### For Existing Projects

1. Compare current quality checks with gates
2. Adopt missing gates incrementally
3. Consolidate custom checks into unified script
4. Test thoroughly before deploying

---

## Version Management

- **Current Version**: 1.0 (2026-03)
- **Stability**: Stable
- **Update Frequency**: Quarterly
- **Changelog**: See `worklogs/GOVERNANCE.md`

---

## See Also

- **Pre-commit Hooks**: `dotfiles/hooks/.pre-commit-config.base.yaml`
- **Linter Configs**: `templates/<language>/`
- **Taskfile Templates**: `templates/shared/Taskfile.*.yml`
- **GitHub Actions**: `.github/workflows/`

---

## Template Info

- **Location**: `thegent/templates/quality/`
- **Canonical URL**: See thegent repository
- **Maintained By**: Phenotype Governance Team
- **Last Updated**: 2026-03-29
