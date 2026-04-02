# Research

## Remote state observed

- `BytePortfolio` was not archived at the start of execution and was archived in this wave.
- `router-docs` and `heliosBench` were already archived before execution began.
- `odin-*` course repos were already archived before the archive sweep was executed.

## Ruleset drift observed

- `thegent`, `helios-cli`, and `phenodocs` each had duplicate active default-branch rulesets.
- `cliproxyapi-plusplus` had duplicate active rulesets with no effective rules.
- `BytePort` currently has zero repository rulesets.

## Tooling coverage observed

- `heliosCLI` has the strongest sampled stack: Sentry, Snyk, CodeQL, Semgrep, cargo-deny, leak
  detection, and a repo-local security-guard workflow.
- `thegent` has CodeQL, Semgrep, TruffleHog, and strong PR governance gates but lacked a repo-local
  secret-scan guard workflow.
- `cliproxyapi-plusplus` had strong CodeQL/Semgrep/Trivy/CI guard coverage but its
  `security-guard.yml` was still a placeholder.
- `cloud` is the local reference source for `Chromatic`.
- no clear shelf-local reusable `Socket.dev` workflow source was found in the sampled repos.

## Local readiness observed

- `AgilePlus`, `cloud`, `heliosApp`, `heliosCLI`, `cliproxyapi-plusplus`, and `thegent` all have
  local dirtiness in the canonical shelf checkout.
- `heliosCLI` has a stash on `ci/trigger-workflows`.
- `agentapi-plusplus`, `cliproxyapi-plusplus`, and `thegent` have active open PRs on the current
  local branch.
- `heliosCLI` also has an active open PR on the current local branch.
- `heliosApp` is the sampled branch that still lacks a discovered PR in this pass.
- open PR readiness is uneven:
  - `agentapi-plusplus` current PR is under `CHANGES_REQUESTED` and has many failing checks
  - `cliproxyapi-plusplus` current PR appears closer to repairable, with a much narrower visible
    failing set
  - `thegent` current PR has broad failing governance/security/code-analysis surfaces
- `AgilePlus-phase2` and `heliosCLI/worktrees/chore/fix-dep-drift-python` are flagged prunable by
  `git worktree list`.
- root `.worktrees/*`, repo-local `.worktrees/*`, and `.archive/temp-directories/*` contain
  additional lane surfaces that should not be treated as cleanly disposable without ownership
  mapping.

## Governance baseline chosen

- protect `main`
- no force push
- PR only
- 1 approval minimum
- review thread resolution required
- merge/squash allowed for stacked PR friendliness
- no linear-history-only enforcement

## High-confidence rollout chosen

- create the missing `BytePort` remote ruleset immediately
- align checked-in ruleset baseline JSON files to the remote protected-main posture
- add or repair repo-local `security-guard` workflows where the state is obviously missing or stubbed
- defer `Chromatic`, `Sentry`, and `Socket.dev` to applicability-driven rollout instead of forcing
  them into repos that do not clearly need them
