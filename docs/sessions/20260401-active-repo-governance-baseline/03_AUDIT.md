# Active Repo Governance Baseline Audit

## Audit Summary

The four active repos now have live GitHub rulesets enforcing:

- no branch deletion
- no force push / non-fast-forward updates
- PR-only merge flow
- 1 approval
- stale review dismissal on push
- resolved review threads
- code-owner review
- required checks: `policy-gate` and `pr-governance-gate`

## Findings

### AgilePlus

- Live ruleset is clean and aligned with the checked-in baseline.
- `pr-governance-gate.yml` had an event-handling bug: it only read `pull_request` payloads and would fail on `pull_request_review` events.
- Local worktree still carries many deleted workflows not yet reconciled into a clean governance posture.

### heliosCLI

- Live ruleset is now canonical and explicit.
- Workflow surface remains fragmented: `ci.yml`, `quality.yml`, `rust-ci.yml`, `bazel.yml`, `docs-site.yml`, `pages.yml`, and multiple release/security workflows still overlap.
- `pages.yml` is locally modified and should be reviewed before broader CI normalization.

### heliosApp

- Ruleset and governance gates are now present and live.
- CI surface is still thin relative to governance expectations: the repo mainly exposes `ci.yml`, `policy-gate`, `pr-governance-gate`, `sast-*`, and `security-guard`.
- The repo likely needs a clearer split between required build-quality checks and advisory security/deep-scan workflows.

### thegent

- Live ruleset is canonical and duplicate default-branch rulesets were removed.
- Repo-native governance files are still mostly untracked in the current worktree.
- The workflow surface is smaller than earlier governance docs imply, so tracked workflow reality and policy docs still need reconciliation.

## Priority Order

1. Fix broken governance gates that can silently fail enforcement.
2. Reduce workflow/check-name drift in the active repos with the heaviest CI overlap.
3. Reconcile checked-in repo governance surfaces with actual tracked workflows.
4. Only then widen into deeper CI consolidation and optional advisory scan cleanup.
