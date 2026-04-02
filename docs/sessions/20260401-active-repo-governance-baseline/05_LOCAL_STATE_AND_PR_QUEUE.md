# Local State And PR Queue Audit

## Canonical Active Repos

### AgilePlus

- Branch: `main`
- Local status: heavily dirty beyond governance work
- Stashes: none
- Worktrees:
  - canonical checkout on `main`
  - `AgilePlus-phase2` on `phase2-decomposition`, marked prunable
- Open PRs:
  - PR #262 `gt/shadow/590c50cb` -> `main`
  - PR #261 `gt/birch/78fe54bb` -> `main`
- Assessment:
  - not a good immediate PR-prep target for this governance wave
  - CI is broadly failing on PR #262 and the checkout contains many unrelated local mutations

### heliosCLI

- Branch: `refactor/decouple-harness-crates`
- Local status: dirty but coherent for governance/CI work
- Stashes:
  - `stash@{0}` from `ci/trigger-workflows`
- Worktrees:
  - canonical checkout on `refactor/decouple-harness-crates`
  - `.worktrees/chore-govern-pi` on `chore/governance-migration-hc`
- Open PRs:
  - PR #179 `refactor/decouple-harness-crates` -> `main`
- Assessment:
  - best next execution target
  - live PR already exists
  - merge blockers are concrete: branch behind main, changes requested, repeated `policy-gate` failures, overlapping CI surfaces, and several failing experimental/non-canonical jobs

### heliosApp

- Branch: `feat/fix-typescript-vite-federation`
- Local status: dirty and governance files are largely untracked additions
- Stashes: none
- Worktrees: only canonical checkout
- Open PRs:
  - PR #361 docs/spec lane
  - PR #360 methodology lane
- Assessment:
  - local active branch is not the same lane as the open PRs
  - governance baseline is now in place, but PR-prep should wait until the branch/PR relationship is clarified or a new PR is opened intentionally

### thegent

- Branch: `refactor/cleanup-error-variants`
- Local status: dirty with both modified tracked files and untracked governance files
- Stashes: none
- Worktrees: only canonical checkout
- Open PRs:
  - PR #908 `refactor/cleanup-error-variants` -> `main`
  - other unrelated open PRs also exist (#909-#912)
- Assessment:
  - second-best next execution target
  - live PR already exists
  - blockers are broad CI/security workflow failures plus tracked-vs-untracked governance drift

## Shelf-Level Worktree State

The shelf root itself is also a git repository with additional worktrees under `.worktrees/`.

### `.worktrees/feat/http-client-core-fixes`

- Branch: `feat/http-client-core-fixes`
- Dirty:
  - `crates/phenotype-http-client-core/src/retry.rs`
  - `crates/phenotype-http-client-core/src/transport.rs`
- No stash
- This is a real active lane and should not be mixed into the active-repo governance PR wave.

### `.worktrees/feat/phenotype-crypto-complete`

- Branch: `feat/phenotype-crypto-complete-v2`
- Dirty with staged/unstaged crypto crate implementation work
- Shares the shelf-level stash stack below
- Not part of the current four-repo governance wave

### `.worktrees/feat/cache-adapter-impl`

- Detached HEAD
- Clean working tree
- Shares a very large shelf-level stash stack
- Treat as a recovery/archive candidate rather than a current execution lane

## Shelf-Level Stash Risk

The shelf root repo has a large stash backlog (39 entries observed through the `.worktrees` inventory), including:

- `tmp-pr-545`
- `pr-544`
- `phenotype-infrakit-main`
- `fix/http-client-core-simplify`
- multiple `main` WIP snapshots

This stash stack is separate from the active repo-local governance work and should be triaged in a dedicated shelf-level recovery pass. It is too risky to mix into the current PR prep lane.

## Temp Clone Audit

- `temp-PRODVERCEL` currently does not expose live git checkouts in the scanned paths.
- `/Users/kooshapari/temp-PRODVERCEL/485/kush/thegent` exists as a directory but is not a git repo in its current state.
- No temp clone produced a better PR-prep candidate than the canonical active repo branches.

## PR-Prep Priority

1. `heliosCLI` PR #179
2. `thegent` PR #908
3. `heliosApp` only after branch/PR lane alignment is made explicit
4. `AgilePlus` after separate workspace cleanup or isolation into a fresh branch/worktree
