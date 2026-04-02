# GitHub Rulesets

This document is the canonical repo-tracked contract for GitHub merge policy on active AgilePlus
lanes. GitHub rulesets remain an external settings surface, but this file defines the intended
configuration and the CI-side checks that should backstop it.

## Branch Classes

### Protected Branches

- `main`
- `canary`
- `release/*`

### Layered PR Branches

Preferred branch prefixes for stacked and incremental delivery:

- `stack/`
- `layer/`
- `feat/`
- `fix/`
- `docs/`
- `refactor/`
- `ci/`
- `chore/`

## Required GitHub Ruleset Posture

Apply these rules in GitHub for every protected branch above:

1. Require pull requests before merge.
2. Require linear history.
3. Block force pushes.
4. Block branch deletion.
5. Require all conversations resolved before merge.
6. Require status checks to pass.
7. Require the branch to be up to date before merge.
8. Require CODEOWNERS review on owned paths.
9. Dismiss stale approvals when new commits are pushed.
10. Restrict bypass actors to explicit emergency maintainers only.

## Approval Policy

- Standard changes: at least 1 approval.
- Constitution or governance-path changes: 2 approvals.
- No merge while review state is `CHANGES_REQUESTED`.
- No merge with unresolved review threads.

## CI Policy

Required checks for protected branches should include:

- `policy-gate`
- `pr-governance-gate`
- `quality-gate`
- core CI workflow checks from `.github/workflows/ci.yml`
- security checks that are intended to block merge for the branch

Recommended security checks:

- `code-scanning-results`
- `sast-quick`
- `snyk-scan`

## Billing Exception Rule

The only tolerated merge exception is a documented GitHub Actions billing failure.

Requirements:

1. The failing check name must clearly be billing-related.
2. The PR must carry the `ci-billing-exception` label.
3. All non-billing checks must still be green.
4. All review threads must still be resolved.
5. Any merge under this exception must be called out in the PR body.

This is an exception for platform billing failure only, not for flaky test suites, lint failures,
or broken workflow logic.

## Stacked PR Policy

- Stacked PRs are the preferred path for multi-part active work.
- Each PR in a stack must be independently reviewable.
- Base branches may target another layer branch instead of `main`.
- `fix/*` or `feat/*` PRs targeting `main` directly require either:
  - no stack is needed, or
  - the `layered-pr-exception` label with rationale in the PR body.

## PR Body Requirements

Every PR must include:

- summary of change and rationale
- validation performed
- stack topology
- governance declaration

The canonical format is enforced through `.github/pull_request_template.md` and
`.github/workflows/pr-governance-gate.yml`.

## Active Repo Baseline

This policy should be mirrored for active Phenotype repos currently under stabilization:

- `AgilePlus`
- `heliosCLI`
- `thegent`
- `agentapi-plusplus`

When repo-specific workflows differ, preserve the same merge invariants:

- no force push to protected branches
- no merge with unresolved comments
- no merge with red CI except the billing exception rule
- stacked PRs preferred for multi-part work
