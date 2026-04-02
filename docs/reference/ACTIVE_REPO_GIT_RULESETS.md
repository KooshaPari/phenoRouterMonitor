# Active Repo Git Rulesets

This shelf-level note defines the target merge policy for active Phenotype repos currently being
stabilized. GitHub rulesets and branch protections are still configured externally, but these are
the intended invariants to apply consistently.

## Active Repo Set

- `AgilePlus`
- `agentapi-plusplus`
- `cliproxyapi-plusplus`
- `cloud`
- `forgecode`
- `heliosApp`
- `heliosCLI`
- `phenotype-infrakit`
- `thegent`

## Required Merge Invariants

1. No force-push to `main`, `master`, `canary`, or `release/*`.
2. Pull request required for protected branches.
3. Linear history required.
4. All review conversations resolved before merge.
5. No open `changes requested` review state at merge time.
6. All non-billing CI checks green before merge.
7. Stacked PRs preferred for multi-part work.
8. Up-to-date base branch required before merge.
9. Helper bots or review assistants do not replace approval, comment resolution, or green CI.

## Billing Exception

Only GitHub Actions billing failures are allowed as an exception path, and only when:

- the failure is clearly billing-related
- the exception is documented in the PR body
- all non-billing checks are green
- all review comments are resolved

## CI Backstops

When repo-local support exists, CI should backstop the GitHub ruleset by checking:

- PR template completeness
- stacked PR topology disclosure
- unresolved review thread count
- review decision state
- all relevant check runs except documented billing failures

## Live Ruleset Inventory

The current GitHub ruleset list endpoint shows active rulesets for:

- `AgilePlus`
- `agentapi-plusplus`
- `cliproxyapi-plusplus`
- `heliosCLI`
- `thegent`
- `cloud`

No active ruleset payload was visible from the current auth context for:

- `heliosApp`
- `forgecode`
- `phenotype-infrakit`

Per-ruleset detail fetches are currently returning `404`, so this document should be treated as the
repo-tracked contract until higher-privilege verification is available.

## Current Active-Repo Snapshot

| Repo | Repo-tracked governance posture | Live PR pressure | Immediate action |
|---|---|---|---|
| `AgilePlus` | strong baseline, PR template, PR governance gate, CODEOWNERS | open PRs `#262` and `#261` have repeated `policy-gate` and audit failures | keep strict ruleset, fix failing checks before any merge |
| `agentapi-plusplus` | strong workflow surface, but helper-heavy and thin core CI | open PR `#398` has `CHANGES_REQUESTED`, `policy-gate`, `lint-test`, and build failures | pin only real merge-blocking checks in rulesets |
| `cliproxyapi-plusplus` | strong policy and check manifest but missing ownership backstop | open PR `#942` exists, but check metadata is sparse from current query | add CODEOWNERS and validate required-check names |
| `cloud` | mature CI and deploy flows, weak PR governance surface | open PRs are mostly review-waiting, not CI-driven from current sample | add CODEOWNERS and explicit policy gate |
| `forgecode` | weak before this lane, now being bootstrapped | no sampled open PR pressure | add CI and PR governance before enabling blocking rulesets |
| `heliosApp` | weak before this lane, now being bootstrapped | open PRs `#361` and `#360` already show red CI and review pressure | add PR governance surface and make secret scanning real |
| `heliosCLI` | strong branch-policy and stage-gate posture | open PR `#179` has `CHANGES_REQUESTED`, `policy-gate`, spelling, and Bazel failures | keep ruleset strict and ensure helper automations stay non-required |
| `phenotype-infrakit` | new bootstrap files present in this lane | no current ruleset payload visible | finish baseline and then enable server-side protection |
| `thegent` | documented ruleset baseline but sparse ownership and review backstops | open PR `#912` has `CHANGES_REQUESTED` and failing CodeQL analysis | add CODEOWNERS and keep policy checks required |

Repo-local backstops are now present in:

- [github-rulesets.md](/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/docs/process/github-rulesets.md)
- [pr-governance-gate.yml](/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/.github/workflows/pr-governance-gate.yml)
- [RULESET_BASELINE.md](/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI/.github/RULESET_BASELINE.md)
- [pr-governance-gate.yml](/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI/.github/workflows/pr-governance-gate.yml)
- [RULESET_BASELINE.md](/Users/kooshapari/CodeProjects/Phenotype/repos/thegent/.github/RULESET_BASELINE.md)
- [pr-governance-gate.yml](/Users/kooshapari/CodeProjects/Phenotype/repos/thegent/.github/workflows/pr-governance-gate.yml)
- [RULESET_BASELINE.md](/Users/kooshapari/CodeProjects/Phenotype/repos/agentapi-plusplus/.github/RULESET_BASELINE.md)
- [pr-governance-gate.yml](/Users/kooshapari/CodeProjects/Phenotype/repos/agentapi-plusplus/.github/workflows/pr-governance-gate.yml)

## 2026-04-02 Progress Note

- `AgilePlus`, `heliosCLI`, `thegent`, and `agentapi-plusplus` now have repo-tracked PR governance backstops for stacked-PR disclosure, unresolved-thread rejection, `CHANGES_REQUESTED` rejection, and billing-only CI exceptions.
- GitHub-hosted rulesets still need to be aligned in the server-side admin UI or API. The repo-local workflows enforce posture, but they do not replace branch protection configuration.
