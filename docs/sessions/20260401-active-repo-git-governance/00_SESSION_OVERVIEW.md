# Session Overview

Session: `20260401-active-repo-git-governance`

## Goal

Add a live governance baseline for active repos in the shelf recovery wave so Git rulesets, PR
discipline, CI gates, and billing-only exception handling are explicit and resumable.

## Scope

- `AgilePlus`
- `agentapi-plusplus`
- `cliproxyapi-plusplus`
- `cloud`
- `forgecode`
- `heliosApp`
- `heliosCLI`
- `phenotype-infrakit`
- `thegent`

## Outputs

- `artifacts/active-repo-git-governance-baseline.md`
- `docs/reference/ACTIVE_REPO_GIT_RULESETS.md`
- repo-local governance bootstraps for weak active repos
- AgilePlus governance tracking under `002-org-wide-release-governance-dx-automation`

## Success Criteria

- active repos have one shared baseline for:
  - no force pushes on protected branches
  - no `--no-verify`
  - required green CI before merge
  - billing-only exception path
  - stacked PR discipline
  - minimum CI/tooling expectations
- weak repos gain repo-local policy files that can actually support those rulesets
