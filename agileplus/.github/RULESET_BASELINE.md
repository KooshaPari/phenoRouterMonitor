# AgilePlus Ruleset Baseline

This repository already has meaningful CI and policy workflows. GitHub rulesets
should require the following on protected branches:

- pull request required before merge
- no force push
- no branch deletion
- linear history
- CODEOWNERS review
- conversation resolution before merge
- required checks:
  - `policy-gate`
  - `pr-governance-gate`
  - `verify`
  - `semgrep`
  - `secrets`
  - `lint-rust`
  - `license-check`
  - `snyk-test` when Snyk is configured

## Branch Policy

- `stack/*`, `layer/*`, `release/*`, and similar integration branches are valid
  stacked PR lanes.
- `fix/*` must not target `main` or `master` unless the PR carries a documented
  exception label such as `layered-pr-exception`.
- Merge commits in PR branches are disallowed.
- Local `--no-verify` usage is not accepted as a reason to bypass server-side
  workflow checks.

## Exception Policy

- Only documented billing or quota failures may be excluded from required checks.
- Review threads and blocking comments must be resolved before merge.
