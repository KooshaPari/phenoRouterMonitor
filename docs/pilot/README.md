# Pilot Rollout Plan

This document describes the pilot rollout of pheno-cli governance tooling across 4 target repositories.

## Target Repositories

| Repo | Language | Registry | Visibility |
|------|----------|----------|------------|
| AgilePlus | TypeScript | npm (private) | Private |
| tokenledger | Rust | crates.io | Public |
| thegent | Python | PyPI | Public |
| agentapi-plusplus | Go | go proxy | Public |

## Bootstrap Checklist (per repo)

- [ ] Run `pheno-cli bootstrap` in repo root
- [ ] Verify `mise.toml` generated with correct tool versions
- [ ] Verify CI workflow files created in `.github/workflows/`
- [ ] Verify git hooks installed (pre-commit, commit-msg)
- [ ] Verify `pheno-cli validate` passes all checks

## Expected Outputs Per Repo

### AgilePlus (TypeScript, Private)
- `mise.toml` with node/bun version pins
- `.github/workflows/release.yml` (no publish step)
- `.github/workflows/ci.yml`
- Git hooks: pre-commit lint + type-check
- No registry publish (private project)

### tokenledger (Rust)
- `mise.toml` with rust toolchain version
- `.github/workflows/release.yml` targeting crates.io alpha
- Expected pre-release format: `0.1.0-alpha.1`
- `Cargo.toml` version management via pheno-cli

### thegent (Python)
- `mise.toml` with python version pin
- `.github/workflows/release.yml` targeting PyPI alpha
- Expected pre-release format: `0.2.0a1`
- `pyproject.toml` version management via pheno-cli

### agentapi-plusplus (Go)
- `mise.toml` with go version pin
- `.github/workflows/release.yml` using git tag for go proxy
- Expected pre-release format: `v1.0.0-alpha.1`
- `go.mod` version management via pheno-cli

## Validation Criteria

A pilot repo is considered passing when:

1. `pheno-cli validate` exits 0
2. `mise.toml` exists with at least one tool entry
3. At least one CI workflow exists under `.github/workflows/`
4. Git hooks directory is populated
5. Version format matches expected registry convention

## Pilot Guides

- [AgilePlus](./agilePlus.md) — TypeScript, private npm
- [tokenledger](./tokenledger.md) — Rust, crates.io
- [thegent](./thegent.md) — Python, PyPI
- [agentapi-plusplus](./agentapi-plusplus.md) — Go, go proxy
