# Research

## Existing Governance Signals

- Shelf governance already forbids `--no-verify` and force-push on shared branches in
  `docs/agents/governance-constraints.md`.
- Existing branch-protection guidance exists in `.github/BRANCH_PROTECTION_SPECS_MAIN.md`, but it
  is SSOT/spec-branch oriented rather than a current active-repo execution baseline.

## Active Repo Workflow Surface

### AgilePlus

Observed workflow coverage includes:

- `ci.yml`
- `quality-gate.yml`
- `policy-gate.yml`
- `sast-quick.yml`
- `snyk-scan.yml`
- `self-merge-gate.yml`
- `evidence-capture.yml`

### heliosCLI

Observed workflow coverage includes:

- `ci.yml`
- `rust-ci.yml`
- `quality.yml`
- `policy-gate.yml`
- `sast-quick.yml`
- `snyk-scan.yml`
- docs, Bazel, release, and stage-gate workflows

### platforms/thegent

Observed repo-local workflow coverage is currently minimal:

- `codeql.yml`

This makes `thegent` the bootstrap case for CI/ruleset hardening.

### Additional active repos

- `agentapi-plusplus` already has broad workflow coverage: `ci`, `lint-test`, `policy-gate`,
  `quality-gate`, `security-guard`, `sast-quick`, docs, preview, SDK, and release workflows.
- `cliproxyapi-plusplus` already has strong workflow coverage: `ci`, `lint-test`, `policy-gate`,
  `required-check-names-guard`, docs, CodeQL, SAST, and auto-merge helpers.
- `cloud` has mature core CI and deploy flows plus `trufflehog`, but weak repo-local review policy
  surfaces.
- `heliosApp` has `CI` and security scans, but lacked repo-native PR governance files before this
  lane.
- `forgecode` had security scans only and no primary CI workflow before this lane.
- `phenotype-infrakit` had security scans only and now has bootstrap governance files in flight.

## Governance Tension

- Some historical spec/task docs still mention `--no-verify` as technically possible standard Git
  behavior.
- The live governance baseline for active repos needs to be stricter than those historical notes:
  no `--no-verify`, no convenience force-push on shared branches, and no merge with unresolved CI
  except the billing/quota no-jobs-started case.

## Live GitHub Ruleset Visibility

- Active rulesets are visible from the current auth context for:
  - `AgilePlus`
  - `agentapi-plusplus`
  - `cliproxyapi-plusplus`
  - `heliosCLI`
  - `thegent`
  - `cloud`
- No list payload was visible for:
  - `heliosApp`
  - `forgecode`
  - `phenotype-infrakit`
- Per-ruleset detail fetches are currently returning `404`, so exact server-side rule payloads are
  not available from the current token.

## Open PR Pressure

- `AgilePlus` open PRs sampled in this lane are failing `policy-gate` and additional checks.
- `agentapi-plusplus` open PR `#398` is in `CHANGES_REQUESTED` and also has red checks.
- `heliosCLI` open PR `#179` is in `CHANGES_REQUESTED` with `policy-gate` and CI failures.
- `heliosApp` open PRs sampled in this lane already show red CI.
- `thegent` open PR `#912` is in `CHANGES_REQUESTED` with failing analysis jobs.

That makes the governance stance straightforward: no sampled active PR is a candidate for bypass or
relaxed merge treatment right now.
