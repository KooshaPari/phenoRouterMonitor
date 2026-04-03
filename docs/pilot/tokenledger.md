# tokenledger Pilot Guide

tokenledger is a Rust project targeting crates.io for alpha pre-release publishing.

## Bootstrap

```bash
cd /path/to/tokenledger
pheno-cli bootstrap --lang rust --registry crates.io
```

## Expected Outputs

### mise.toml

```toml
[tools]
rust = "1.76"

[env]
CARGO_REGISTRY = "crates.io"
```

### CI Workflows

`.github/workflows/ci.yml` — runs on every push/PR:
- `cargo check`
- `cargo test`
- `cargo clippy -- -D warnings`

`.github/workflows/release.yml` — runs on version tags matching `v*-alpha.*`:
- `cargo test`
- `cargo publish --allow-dirty` to crates.io

### Git Hooks

`pre-commit`:
- Runs `cargo check`
- Runs `cargo fmt -- --check`

`commit-msg`:
- Enforces conventional commit format

## Alpha Publish Steps

1. Bump version to alpha:
   ```bash
   pheno-cli version bump --channel alpha --increment 1
   # Sets Cargo.toml version to: 0.1.0-alpha.1
   ```

2. Commit and tag:
   ```bash
   git add Cargo.toml
   git commit -m "chore: bump version to 0.1.0-alpha.1"
   git tag v0.1.0-alpha.1
   git push origin main --tags
   ```

3. CI picks up the tag and publishes to crates.io.

## Expected Pre-release Format

```
0.1.0-alpha.1
```

crates.io uses SemVer pre-release syntax. pheno-cli generates this via the `crates.io` registry target.

## Validation

```bash
pheno-cli validate --repo .
```

Expected output:
```
[PASS] mise.toml exists
[PASS] CI workflows installed
[PASS] Git hooks installed
[PASS] Cargo.toml version format: 0.1.0-alpha.1
[PASS] All checks passed
```
