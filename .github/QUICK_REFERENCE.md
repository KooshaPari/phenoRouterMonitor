# GitHub Actions Composite Actions - Quick Reference

## One-Minute Overview

5 reusable composite actions consolidate common CI/CD patterns:

| Action | Use When | Key Inputs |
|--------|----------|-----------|
| `setup-env` | Starting any Rust job | `rust-version`, `setup-protoc`, `checkout-depth` |
| `run-tests` | Running tests & linting | `test-command`, `lint-command`, `skip-lint` |
| `build-rust-binary` | Building release binaries | `target`, `use-cross`, `binary-name` |
| `security-checks` | Running security scans | `cargo-audit`, `cargo-deny`, `gitleaks`, `python-bandit` |
| `run-benchmarks` | Running performance tests | `benchmark-dir`, `tool`, `output-file` |

---

## Copy-Paste Examples

### Example 1: Simple Test & Lint
```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: ./.github/actions/setup-env
      - uses: ./.github/actions/run-tests
```

### Example 2: Release Binary Build
```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target: [x86_64-unknown-linux-gnu, x86_64-apple-darwin]
    steps:
      - uses: ./.github/actions/setup-env
        with:
          rust-version: stable
          setup-protoc: 'true'

      - uses: ./.github/actions/build-rust-binary
        with:
          target: ${{ matrix.target }}
          use-cross: ${{ matrix.target != 'x86_64-unknown-linux-gnu' }}
```

### Example 3: Full Security Scan
```yaml
jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: ./.github/actions/setup-env
        with:
          checkout-depth: '0'  # full history for gitleaks

      - uses: ./.github/actions/security-checks
        with:
          cargo-audit: 'true'
          cargo-deny: 'true'
          gitleaks: 'true'
          python-bandit: 'false'  # only if Python code exists
```

### Example 4: Benchmarks with Nightly Rust
```yaml
jobs:
  bench:
    runs-on: ubuntu-latest
    steps:
      - uses: ./.github/actions/setup-env
        with:
          rust-version: nightly

      - uses: ./.github/actions/run-benchmarks
```

---

## Default Values (No Inputs Needed)

```yaml
# Minimal setup
- uses: ./.github/actions/setup-env
# Equivalent to:
#   rust-version: stable
#   setup-protoc: 'false'
#   checkout-depth: '1'

# Minimal tests
- uses: ./.github/actions/run-tests
# Equivalent to:
#   test-command: cargo test --all
#   lint-command: cargo clippy -- -D warnings
#   skip-lint: 'false'

# Minimal benchmarks
- uses: ./.github/actions/run-benchmarks
# Equivalent to:
#   benchmark-dir: rust/benches
#   tool: cargo
#   output-file: target/criterion/output.txt
```

---

## Input Cheat Sheet

### setup-env
- `rust-version`: `stable` (default), `nightly`, `1.70.0`
- `setup-protoc`: `'true'` / `'false'` (default)
- `checkout-depth`: `'1'` (default), `'0'` (full history)

### run-tests
- `test-command`: any valid `cargo ...` command
- `lint-command`: any valid `cargo clippy ...` command
- `skip-lint`: `'true'` to skip clippy

### build-rust-binary
- `target`: `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, etc.
- `use-cross`: `'true'` for cross-compilation, `'false'` (default)
- `binary-name`: default `agileplus`
- `artifact-name`: auto-generated if not specified
- `strip-binary`: `'true'` / `'false'` (default)

### security-checks
- All checks: `'true'` (default) / `'false'`
- Configs: `cargo-deny-config` (path), `bandit-path` (path)

### run-benchmarks
- `benchmark-dir`: where benches live (default: `rust/benches`)
- `tool`: `cargo` (default)
- `output-file`: where criterion outputs results

---

## Common Patterns

### Pattern 1: CI with Tests + Security
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: ./.github/actions/setup-env
      - uses: ./.github/actions/run-tests

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: ./.github/actions/setup-env
        with:
          checkout-depth: '0'
      - uses: ./.github/actions/security-checks
```

### Pattern 2: Release with Multiple Targets
```yaml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - x86_64-unknown-linux-gnu
          - aarch64-unknown-linux-gnu
          - x86_64-apple-darwin
    steps:
      - uses: ./.github/actions/setup-env
        with:
          setup-protoc: 'true'

      - uses: ./.github/actions/build-rust-binary
        with:
          target: ${{ matrix.target }}
          use-cross: ${{ matrix.target != 'x86_64-unknown-linux-gnu' }}
```

### Pattern 3: Comprehensive Quality Gate
```yaml
name: Quality Gate

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: ./.github/actions/setup-env
      - uses: ./.github/actions/run-tests

  bench:
    runs-on: ubuntu-latest
    steps:
      - uses: ./.github/actions/setup-env
        with:
          rust-version: nightly
      - uses: ./.github/actions/run-benchmarks

  security:
    runs-on: ubuntu-latest
    steps:
      - uses: ./.github/actions/setup-env
        with:
          checkout-depth: '0'
      - uses: ./.github/actions/security-checks
        with:
          python-bandit: 'false'  # no Python in this project
```

---

## Troubleshooting

### "Action not found"
- Composite actions live in `.github/actions/*/action.yml`
- Make sure to commit them: `git add .github/actions && git push`

### "Input not recognized"
- Check spelling and case (inputs are case-sensitive)
- Verify input name exists in the action.yml file

### "Artifact upload failed"
- `build-rust-binary` uploads to: `target/{target}/release/{binary-name}`
- Verify binary name matches what cargo creates

### "Bench detection failing"
- `run-benchmarks` looks for `rust/benches/` or `benches/` directory
- Customize with `benchmark-dir` input if different location

### "Setup is slow"
- Default `checkout-depth: '1'` is fast (shallow clone)
- Use `checkout-depth: '0'` only when needed (gitleaks, changelog generation)

---

## Files to Know About

**Composites (read for customization ideas):**
- `.github/actions/setup-env/action.yml`
- `.github/actions/run-tests/action.yml`
- `.github/actions/build-rust-binary/action.yml`
- `.github/actions/security-checks/action.yml`
- `.github/actions/run-benchmarks/action.yml`

**Full Documentation:**
- `.github/COMPOSITE_ACTIONS_GUIDE.md` (detailed reference)
- `.github/CONSOLIDATION_SUMMARY.md` (background & metrics)
- `.github/QUICK_REFERENCE.md` (this file)

**Real-World Examples:**
- `.github/workflows/ci.yml` (minimal example)
- `.github/workflows/release.yml` (complex multi-target)
- `.github/workflows/security.yml` (unified checks)

---

## Quick Syntax Validation

To validate a workflow file locally:
```bash
# Use GitHub's official action validator
# (requires Docker)
docker run -v $(pwd):/repo:ro ghcr.io/actions/action-validator

# Or validate YAML syntax only
ruby -ryaml -e "YAML.load(File.read('.github/workflows/ci.yml'))"
```

---

## Need Help?

1. **For specific action details** → `.github/COMPOSITE_ACTIONS_GUIDE.md`
2. **For background & design** → `.github/CONSOLIDATION_SUMMARY.md`
3. **For examples** → check updated `.github/workflows/*.yml` files
4. **For GitHub Actions docs** → https://docs.github.com/en/actions

---

**Last Updated:** 2026-03-29
**Status:** All composites validated ✓ All workflows validated ✓
