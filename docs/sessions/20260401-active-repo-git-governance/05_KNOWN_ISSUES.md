# Known Issues

## GitHub Ruleset Detail API Visibility

- `gh api repos/<owner>/<repo>/rulesets` returns list data from the current auth context.
- `gh api repos/<owner>/<repo>/rulesets/<id>` is currently returning `404`.
- Impact: this lane can confirm ruleset presence, but not reliably dump the exact server-side rule
  payload for every repo.

## Active PRs Are Not Merge-Ready

- `AgilePlus` open PRs sampled in this lane are failing `policy-gate` and other checks.
- `agentapi-plusplus` open PR `#398` has `CHANGES_REQUESTED` plus failing checks.
- `heliosCLI` open PR `#179` has `CHANGES_REQUESTED`, `policy-gate`, and CI failures.
- `heliosApp` open PRs sampled in this lane already have red CI and review pressure.
- `thegent` open PR `#912` has `CHANGES_REQUESTED` and failing analysis jobs.

## Repo Coverage Gaps Still Remaining

- `cloud` still lacks a repo-local policy gate and `CODEOWNERS` backstop.
- `thegent` still lacks repo-local `CODEOWNERS`.
- `cliproxyapi-plusplus` still lacks ownership enforcement from a visible `CODEOWNERS` surface.
