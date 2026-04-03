# GitHub Actions Reusable Workflows

This directory contains reusable workflow files for centralized CI/CD operations across the organization.

## Workflows

### publish.yml

Reusable workflow for publishing packages to package registries.

**Inputs:**
- `language` (required, string): Programming language (npm, pypi, crates.io)
- `registry` (required, string): Package registry (npm, pypi, crates.io)
- `version` (required, string): Package version to publish
- `dry_run` (optional, boolean): Run without publishing (default: false)

**Secrets:**
- `NPM_TOKEN` (optional): NPM registry token
- `PYPI_TOKEN` (optional): PyPI registry token
- `CARGO_TOKEN` (optional): Cargo/crates.io registry token

**Usage:**
```yaml
jobs:
  publish:
    uses: ./.github/workflows/publish.yml
    with:
      language: npm
      registry: npm
      version: 1.0.0
    secrets:
      NPM_TOKEN: ${{ secrets.NPM_TOKEN }}
```

### gate-check.yml

Reusable workflow for running quality gates (lint, test, audit).

**Inputs:**
- `language` (required, string): Programming language
- `channel` (required, string): Release channel (alpha, beta, stable)
- `risk_profile` (required, string): Risk profile (low, medium, high)

**Behavior:**
- Always runs: lint and test
- Conditional: audit runs for stable channel or high risk profile

**Usage:**
```yaml
jobs:
  gate:
    uses: ./.github/workflows/gate-check.yml
    with:
      language: npm
      channel: stable
      risk_profile: high
```

### promote.yml

Reusable workflow for promoting packages between release channels with gate checks.

**Inputs:**
- `language` (required, string): Programming language
- `registry` (required, string): Package registry
- `from_channel` (required, string): Source release channel
- `to_channel` (required, string): Target release channel
- `risk_profile` (required, string): Risk profile (low, medium, high)
- `version` (required, string): Package version to promote

**Secrets:**
- `NPM_TOKEN` (optional): NPM registry token
- `PYPI_TOKEN` (optional): PyPI registry token
- `CARGO_TOKEN` (optional): Cargo/crates.io registry token

**Behavior:**
- Runs gate checks (lint, test, audit if applicable)
- Publishes to registry if all gates pass

**Usage:**
```yaml
jobs:
  promote:
    uses: ./.github/workflows/promote.yml
    with:
      language: npm
      registry: npm
      from_channel: alpha
      to_channel: beta
      risk_profile: medium
      version: 1.0.0
    secrets:
      NPM_TOKEN: ${{ secrets.NPM_TOKEN }}
```

### changelog.yml

Reusable workflow for generating changelog entries using git-cliff.

**Inputs:**
- `version` (required, string): Version for changelog entry

**Behavior:**
- Uses git-cliff to generate changelog
- Commits changes with automated git bot account

**Usage:**
```yaml
jobs:
  changelog:
    uses: ./.github/workflows/changelog.yml
    with:
      version: 1.0.0
```

### audit.yml

Reusable workflow for running organization-wide audits using pheno audit.

**Inputs:**
- `repos_dir` (required, string): Directory containing repos to audit

**Behavior:**
- Runs `pheno audit` with JSON output
- Uploads results as artifact

**Usage:**
```yaml
jobs:
  audit:
    uses: ./.github/workflows/audit.yml
    with:
      repos_dir: ./repos
```

## Security Considerations

All workflows use environment variables to safely pass inputs to shell commands, following GitHub's security best practices for workflow injection prevention.

## Dependencies

These workflows assume:
- `mise` is available for task orchestration
- `git-cliff` is available for changelog generation (changelog.yml)
- `pheno` CLI is available (audit.yml)
- Languages/registries are properly configured (publish.yml, promote.yml)
