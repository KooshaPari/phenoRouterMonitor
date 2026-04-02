# Research

## Cross-Repo Snapshot

- `AgilePlus`: rich local governance surface, weak live ruleset, no machine-readable required-check/ruleset manifest
- `heliosCLI`: active ruleset exists but is too weak; repo docs and policy surfaces are stale and drifted
- `heliosApp`: no live ruleset, no CODEOWNERS, no PR template, no policy-gate, and one dead Rust CI lane
- `thegent`: stronger live ruleset than the others, but repo-native `.github` governance surfaces are incomplete

## Implementation Direction

- add `.github/rulesets/main.json` in active repos as the checked-in GitHub ruleset source of truth
- add a reusable `gh api` apply script
- add missing `.github/CODEOWNERS` and PR templates where absent
- add `policy-gate.yml` where absent
- fix obvious workflow drift in `heliosApp`
