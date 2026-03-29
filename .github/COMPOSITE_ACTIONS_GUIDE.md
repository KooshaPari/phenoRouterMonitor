# GitHub Actions Composite Actions Guide

This document describes the composite actions created to consolidate and reduce duplication in GitHub Actions workflows.

## Overview

Composite actions bundle multiple workflow steps into reusable units. They reduce YAML duplication, improve maintainability, and ensure consistency across workflows.

**Created: 5 composite actions**
- `setup-env` - Environment setup (checkout, Rust toolchain, caches, protoc)
- `run-tests` - Rust testing & linting
- `build-rust-binary` - Cross-compile Rust binaries
- `security-checks` - Unified security scanning
- `run-benchmarks` - Benchmark execution & storage

**Updated: 6 workflows** (ci, release, security, benchmark, codeql, tag-automation)

**Estimated Savings: ~100 LOC of duplicated steps**

---

## Composite Actions Reference

### `setup-env` (.github/actions/setup-env/action.yml)

Initializes the build environment with checkout, Rust toolchain, caching, and Protocol Buffers support.

**Inputs:**
- `rust-version` (default: `stable`) - Toolchain version (stable, nightly, etc.)
- `setup-protoc` (default: `false`) - Install Protocol Buffers compiler
- `checkout-depth` (default: `1`) - Git fetch depth (0 = full history)

**Replaces:**
- `actions/checkout@v4` / `@v6`
- `dtolnay/rust-toolchain@*`
- `Swatinem/rust-cache@v2`
- `arduino/setup-protoc@v3` (optional)

**Usage Example:**

```yaml
steps:
  - uses: ./.github/actions/setup-env
    with:
      rust-version: stable
      setup-protoc: 'true'
      checkout-depth: '0'  # full history for tag extraction
```

---

### `run-tests` (.github/actions/run-tests/action.yml)

Executes Rust tests and clippy linting with customizable commands.

**Inputs:**
- `test-command` (default: `cargo test --all`) - Custom test command
- `lint-command` (default: `cargo clippy -- -D warnings`) - Custom lint command
- `skip-lint` (default: `false`) - Skip linting step

**Replaces:**
- `run: cargo test --all`
- `run: cargo clippy -- -D warnings`

**Usage Example:**

```yaml
steps:
  - uses: ./.github/actions/run-tests
    with:
      test-command: cargo test --all
      skip-lint: 'false'
```

---

### `build-rust-binary` (.github/actions/build-rust-binary/action.yml)

Builds optimized Rust release binaries with cross-compilation support, binary stripping, and artifact upload.

**Inputs:**
- `target` (required) - Rust target triple (e.g., x86_64-unknown-linux-gnu)
- `use-cross` (default: `false`) - Enable cross-compilation tool
- `binary-name` (default: `agileplus`) - Output binary name
- `artifact-name` - GitHub artifact name (defaults to `{binary-name}-{target}`)
- `strip-binary` (default: `false`) - Strip binary after build

**Replaces:**
- `cargo install cross` (conditional)
- `cross build` / `cargo build --release`
- Binary stripping logic
- `actions/upload-artifact@v4` calls

**Usage Example:**

```yaml
steps:
  - uses: ./.github/actions/build-rust-binary
    with:
      target: x86_64-apple-darwin
      use-cross: 'true'
      binary-name: agileplus
      artifact-name: agileplus-x86_64-apple-darwin
      strip-binary: 'true'
```

---

### `security-checks` (.github/actions/security-checks/action.yml)

Unified security scanning: cargo-audit, cargo-deny, gitleaks, and Python bandit.

**Inputs:**
- `cargo-audit` (default: `true`) - Run cargo-audit
- `cargo-deny` (default: `true`) - Run cargo-deny
- `cargo-deny-config` (default: `rust/deny.toml`) - Config path
- `gitleaks` (default: `true`) - Run gitleaks
- `python-bandit` (default: `false`) - Run Python bandit
- `bandit-path` (default: `python/src`) - Path to scan

**Replaces:**
- `rustsec/audit-check@v2.0.0` setup
- `cargo install cargo-deny` + `cargo deny check`
- `gitleaks/gitleaks-action@v2` setup
- Python bandit setup & execution

**Usage Example:**

```yaml
steps:
  - uses: ./.github/actions/setup-env
    with:
      checkout-depth: '0'  # gitleaks needs full history

  - uses: ./.github/actions/security-checks
    with:
      cargo-audit: 'true'
      cargo-deny: 'true'
      gitleaks: 'true'
      python-bandit: 'true'
      bandit-path: python/src
```

---

### `run-benchmarks` (.github/actions/run-benchmarks/action.yml)

Executes Rust benchmarks and stores results to GitHub Pages.

**Inputs:**
- `benchmark-dir` (default: `rust/benches`) - Benchmarks directory path
- `tool` (default: `cargo`) - Benchmark tool name
- `output-file` (default: `target/criterion/output.txt`) - Output file path

**Replaces:**
- Benchmarks directory detection logic
- `cargo bench --all --no-run`
- `benchmark-action/github-action-benchmark@v1` setup

**Usage Example:**

```yaml
steps:
  - uses: ./.github/actions/run-benchmarks
    with:
      benchmark-dir: rust/benches
      tool: cargo
      output-file: target/criterion/output.txt
```

---

## Updated Workflows

### ci.yml
- **Before:** 13 lines (basic test + lint)
- **After:** 12 lines
- **Reduction:** 1 line (but consolidated setup steps)

```yaml
# After
steps:
  - uses: ./.github/actions/setup-env
    with:
      rust-version: stable
  - uses: ./.github/actions/run-tests
```

### release.yml
- **Before:** 83 lines
- **After:** 86 lines (added create-release refactor)
- **Key Reduction:** build-release job reduced from 28 steps → 2 actions

```yaml
# build-release job after
steps:
  - uses: ./.github/actions/setup-env
    with:
      rust-version: stable
      setup-protoc: 'true'

  - uses: ./.github/actions/build-rust-binary
    with:
      target: ${{ matrix.target }}
      use-cross: ${{ matrix.use_cross }}
      binary-name: agileplus
      artifact-name: agileplus-${{ matrix.target }}
      strip-binary: 'true'
```

### security.yml
- **Before:** 59 lines (5 separate jobs)
- **After:** 45 lines
- **Reduction:** 14 lines + 3 jobs merged into 1 + 1 dedicated CodeQL job

```yaml
# After: unified security-checks job
- uses: ./.github/actions/setup-env
  with:
    rust-version: stable
    checkout-depth: '0'

- uses: ./.github/actions/security-checks
  with:
    cargo-audit: 'true'
    cargo-deny: 'true'
    gitleaks: 'true'
    python-bandit: 'true'
```

### benchmark.yml
- **Before:** 34 lines
- **After:** 21 lines
- **Reduction:** 13 lines

```yaml
# After
steps:
  - uses: ./.github/actions/setup-env
    with:
      rust-version: nightly
      setup-protoc: 'true'

  - uses: ./.github/actions/run-benchmarks
    with:
      benchmark-dir: rust/benches
      tool: cargo
      output-file: target/criterion/output.txt
```

### codeql.yml
- **Before:** 27 lines
- **After:** 16 lines
- **Reduction:** 11 lines

```yaml
# After
steps:
  - uses: ./.github/actions/setup-env

  - uses: github/codeql-action/init@v3
    with:
      languages: ${{ matrix.language }}
```

### tag-automation.yml
- **Before:** 81 lines
- **After:** 78 lines
- **Reduction:** 3 lines (checkout consolidated)

---

## Benefits

1. **Reduced Duplication**: Common setup patterns consolidated into reusable composites
2. **Consistency**: All workflows use the same toolchain, cache, and checkout logic
3. **Maintainability**: Updates to setup/build logic apply across all workflows
4. **Clarity**: Composite action names document intent better than raw step lists
5. **Version Control**: Toolchain versions (rust-toolchain@stable, actions/checkout@v4) managed in one place
6. **Flexibility**: Input parameters allow customization without workflow YAML editing

---

## Adding New Workflows

When creating new CI/CD workflows:

1. **Use `setup-env`** for any Rust project:
   ```yaml
   - uses: ./.github/actions/setup-env
     with:
       rust-version: stable
       setup-protoc: 'true'  # only if needed
   ```

2. **Use `run-tests`** for test/lint jobs:
   ```yaml
   - uses: ./.github/actions/run-tests
   ```

3. **Use `build-rust-binary`** for release builds:
   ```yaml
   - uses: ./.github/actions/build-rust-binary
     with:
       target: x86_64-unknown-linux-gnu
       use-cross: 'false'
   ```

4. **Use `security-checks`** for security scanning:
   ```yaml
   - uses: ./.github/actions/security-checks
   ```

5. **Use `run-benchmarks`** for performance testing:
   ```yaml
   - uses: ./.github/actions/run-benchmarks
   ```

---

## Testing Composite Actions Locally

Composite actions run on GitHub Actions runners only. To verify behavior:

1. **Push a test branch** with changes to a composite action
2. **Monitor workflow runs** on that branch
3. **Adjust inputs** in test workflows as needed

Example quick test workflow:

```yaml
name: Test Composite Actions

on: workflow_dispatch

jobs:
  test-setup:
    runs-on: ubuntu-latest
    steps:
      - uses: ./.github/actions/setup-env
        with:
          rust-version: stable

      - run: rustc --version
      - run: cargo --version
      - run: git log --oneline | head -5
```

---

## Troubleshooting

### Action Not Found
- Error: `The action './.github/actions/setup-env' is not defined in this repository`
- Cause: Composite action file not committed to repo
- Solution: `git add .github/actions/*/action.yml && git push`

### Input Not Recognized
- Error: `Unexpected input 'unknown-param'`
- Cause: Typo in input name or name not defined in action.yml
- Solution: Check input names in the action definition, use correct casing

### Cache Not Working
- Cause: `setup-env` runs checkout with default depth=1, which may limit cache hits
- Solution: Increase `checkout-depth` if needed (but default 1 is fastest)

### Protoc Setup Issues
- Cause: `setup-protoc: 'true'` not working
- Solution: Ensure repo token is available in `GITHUB_TOKEN` secret
- Note: This is automatic in GitHub Actions (always available)

---

## Future Enhancements

1. **Multi-language support**: Extend `setup-env` for Python, Go, TypeScript
2. **Deployment actions**: Add composites for Docker builds, Kubernetes deployments
3. **Artifact management**: Composite for downloading multiple artifacts
4. **Notifications**: Slack/Discord status composites
5. **Matrix strategies**: Helper composites for complex test matrices

---

## References

- [GitHub Actions: Composite Actions Documentation](https://docs.github.com/en/actions/creating-actions/metadata-syntax-for-github-actions#runs-for-composite-actions)
- [Workflow Reusability & DRY Principles](https://docs.github.com/en/actions/learn-github-actions/workflow-syntax-for-github-actions#jobs)
