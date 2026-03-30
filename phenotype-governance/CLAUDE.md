# CLAUDE.md - phenotype-governance

## Purpose

This repository contains reusable CI/CD workflows, quality configurations, and policy files for the Phenotype organization.

## For Agents

### Adding a New Workflow

1. Create workflow file in `.github/workflows/`
2. Add inputs/outputs documentation
3. Update `ci.yml` to include the new workflow
4. Test with `workflow_dispatch` trigger
5. Tag release for consumers

### Updating Quality Configs

When updating linting configs (clippy.toml, deny.toml, etc.):
1. Changes are consumed by all phenotype-* repos
2. Consider backward compatibility
3. Update version tag after changes
4. Document breaking changes in CHANGELOG

### Policy Files

OPA/Rego policies in `policy/` are evaluated by policy-engine crates.
Changes require:
- Policy syntax validation
- Integration tests
- Version bump

## Quality Gates

All workflows must pass:
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test --all`
- `cargo deny check`

## Stack

- GitHub Actions (reusable workflows)
- cargo (Rust quality)
- ruff/mypy (Python quality)
- buf (protobuf validation)
- ggshield (secret scanning)
