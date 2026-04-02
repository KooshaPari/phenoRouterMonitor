# Specifications

## Objective

Define one shelf-level merge-policy baseline for active repos and bootstrap missing repo-local
governance surfaces so GitHub rulesets can be made strict without relying on tribal knowledge.

## Required Outcomes

- active repos have a repo-tracked statement of protected-branch expectations
- PR templates disclose stack topology, validation, governance, and billing-only exceptions
- CI includes at least one merge-policy gate in addition to build, test, or security jobs
- red CI, unresolved comments, or `CHANGES_REQUESTED` review state block merge
- the only tolerated non-green merge exception is documented GitHub Actions billing or quota
  failure

## Repo Classes

### Strong posture, mostly ruleset-alignment work

- `AgilePlus`
- `agentapi-plusplus`
- `cliproxyapi-plusplus`
- `heliosCLI`

### Medium posture, needs review-policy backstops

- `cloud`
- `thegent`

### Weak posture, needs bootstrap before strict rulesets

- `forgecode`
- `heliosApp`
- `phenotype-infrakit`

## Non-Goals

- replacing all existing CI with one universal workflow
- auto-editing live GitHub rulesets from this lane without exact rule-detail visibility
- waiving policy-gate, review, or CI failures for convenience

## Acceptance Criteria

1. Shelf docs identify active repos, current ruleset visibility, and immediate enforcement gaps.
2. Weak repos gain a minimum governance surface:
   - `CODEOWNERS`
   - PR template
   - ruleset baseline note
   - `policy-gate` workflow
3. Secret scanning is not soft-failed in repos touched by this lane when it is intended to be a
   merge gate.
4. Session docs are complete enough for another lane to continue from the current state without
   re-auditing from scratch.
