# phenotype-infrakit Ruleset Baseline

This repository had security tooling but no clear PR governance contract. GitHub
rulesets should require the following on protected branches:

- pull request required before merge
- no force push
- no branch deletion
- linear history
- CODEOWNERS review
- conversation resolution before merge
- required checks:
  - `policy-gate`
  - `pr-governance-gate`
  - `semgrep`
  - `secrets`
  - `lint-rust`
  - `license-check`
  - `snyk-test` when Snyk is configured

## Branch Policy

- Stacked PR lanes such as `stack/*`, `layer/*`, `preview/*`, and `release/*` are
  valid.
- `fix/*` must not target `main` or `master` unless the PR carries
  `layered-pr-exception`.
- Merge commits in PR branches are disallowed.
- Local `--no-verify` usage is not accepted as a reason to bypass server-side
  workflow checks.

## Exception Policy

- Only documented billing or quota failures may be excluded from required checks.
- Review threads and blocking comments must be resolved before merge.
