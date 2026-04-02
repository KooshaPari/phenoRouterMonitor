# Active Repo Governance Baseline Implementation

## 2026-04-01

- Added checked-in `required-checks.json` manifests for `AgilePlus`, `heliosCLI`, `heliosApp`, and `thegent`.
- Extended each repo's `.github/rulesets/main.json` manifest to require two stable status checks on the default branch:
  - `policy-gate`
  - `pr-governance-gate`
- Added `heliosApp/.github/workflows/pr-governance-gate.yml` so `heliosApp` now has the same merge-blocking governance gate pattern already present in the other active repos.
- Chose the stable two-check baseline intentionally:
  - `policy-gate` owns branch shape and merge-history policy.
  - `pr-governance-gate` dynamically fails if any other non-billing PR check is failing or still pending, which avoids ruleset churn from per-job CI naming drift.
- Reconciled duplicate default-branch rulesets on `heliosCLI` and `thegent` by standardizing the canonical checked-in manifest around the live `Main` ruleset before deleting the extra baseline clone.
- Left pre-existing untracked governance files in `thegent` untouched and additive-only.
