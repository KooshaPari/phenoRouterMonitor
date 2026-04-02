# Known Issues

## Blocking Local-State Issues

### `phenotype-infrakit`

- Branch `fix/http-client-core-simplify` is ahead of `main` by 1 commit, but the shelf root also carries cross-repo status bleed from `AgilePlus`, `cloud`, and shelf docs.
- `git stash list` contains 39 entries, including multiple `main`, `pr-*`, and feature-lane stashes.
- This repo should be treated as a recovery/consolidation lane before any PR work.

### `AgilePlus`

- The repo is heavily dirty on `main` with live DB changes, generated artifacts, deleted workflows, and many untracked spec directories.
- It is not suitable for immediate PR creation without first splitting runtime-state changes from governance/docs changes.

### `agentapi-plusplus`

- The branch is a valid PR lane, but the local dirty tree still mixes workflow guide, PR template,
  worklog, and new governance files.
- Narrow the local diff before updating the already-open PR.

### `cliproxyapi-plusplus`

- The branch is a valid PR lane, but local state still mixes security workflow edits, plan churn,
  and new ruleset files.
- Do not widen that PR before narrowing the local diff.

### `heliosCLI`

- A broken linked worktree entry existed at `worktrees/chore/fix-dep-drift-python`.
- `git worktree prune -v` removed the stale metadata on 2026-04-02.
- The repo still has one stash and mixed governance/doc updates on `refactor/decouple-harness-crates`.

### `thegent`

- PR branch `refactor/cleanup-error-variants` is behind `origin/main`.
- Current local state mixes cargo, worklog, and governance bootstrap files.

### `forgecode`

- Current governance bootstrap work lives on `main`, so it should not go straight to PR without a
  dedicated feature branch.

### `temp-PRODVERCEL`

- `/Users/kooshapari/temp-PRODVERCEL/485/kush/thegent` is not a git repo and should not be treated as a valid PR-prep surface.

## Open PR Branches With Local Drift

- `heliosCLI`: PR branch `refactor/decouple-harness-crates` is open as PR `#179`.
- `thegent`: PR branch `refactor/cleanup-error-variants` is open and locally drifted.
- `agentapi-plusplus`: PR branch `feat/chromatic-visual-testing` is open as PR `#398`.
- `cliproxyapi-plusplus`: PR branch `feat/kilo-gastown-spec-and-sast` is open as PR `#942`.

## Next Constraints

- Do not open new PRs for repos that already have an open branch PR until the local diff is narrowed and validated.
- Do not collapse stash history in `phenotype-infrakit` without a dedicated recovery pass.
