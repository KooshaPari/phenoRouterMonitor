# phenotype-governance

Canonical governance repository for Phenotype organization.

## Structure

```
phenotype-governance/
├── .github/workflows/     # Reusable GitHub Actions workflows
├── configs/              # Quality and linting configs
├── policy/               # OPA/Rego policies
└── templates/            # Repository templates
```

## Workflows

### Reusable Workflows

| Workflow | Purpose |
|----------|---------|
| `ci.yml` | Main orchestrator - combines all quality checks |
| `rust-quality.yml` | Rust: fmt, clippy, test |
| `python-quality.yml` | Python: ruff, mypy, test |
| `proto-contract.yml` | gRPC contract tests |
| `docs-quality.yml` | Markdown linting |
| `security-guard.yml` | Secret scanning (ggshield) |
| `release.yml` | Publish workflow |

## Usage

```yaml
# In any phenotype repo .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  rust-quality:
    uses: KooshaPari/phenotype-governance/.github/workflows/rust-quality.yml@v1
    secrets: inherit
```

## Rationale

Consolidates governance infrastructure from SAGE audit findings (2026-03-29):
- Previously scattered across `infra/` and root configs
- WP10 CI workflow spec implemented here
- Quality configs extracted from root `Cargo.toml`

## Related

- Evidence: `evidence_ledger.jsonl`
- Audit: `docs/worklogs/GOVERNANCE.md`
- WP10 Spec: `docs/specs/002-org-wide-release-governance-dx-automation/tasks/WP10-*.md`
