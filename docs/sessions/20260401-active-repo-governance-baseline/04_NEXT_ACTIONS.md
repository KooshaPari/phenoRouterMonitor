# Next Actions

## Recommended Execution Order

### Wave 1: Enforcement Correctness

1. Fix `AgilePlus/.github/workflows/pr-governance-gate.yml` event handling.
2. Validate the four repos still expose the required checks with the exact names:
   - `policy-gate`
   - `pr-governance-gate`

### Wave 2: CI Surface Normalization

1. `heliosCLI`
   - collapse the required path onto a small stable set of workflows
   - make `rust-ci.yml` the canonical required CI aggregator
   - demote or document overlapping non-required workflows
2. `thegent`
   - move the current untracked governance workflows into the tracked repo state
   - align tracked workflow inventory with governance docs
3. `heliosApp`
   - separate required build/type/lint checks from advisory security/deep-scan jobs
   - ensure the security guard mirrors local hook policy

### Wave 3: Documentation and Repo Hygiene

1. Normalize `RULESET_BASELINE.md`, PR templates, `required-checks.json`, and governance docs in each active repo.
2. Reconcile any stale policy docs that still describe older or copied governance models.
3. Record final required-check names and exception policy in one canonical per-repo governance surface.

## Decision

Proceed audit-first, then execute in waves. The next best implementation target is `heliosCLI` because it has the highest workflow overlap and therefore the highest chance of CI/check-name drift breaking merge policy.
