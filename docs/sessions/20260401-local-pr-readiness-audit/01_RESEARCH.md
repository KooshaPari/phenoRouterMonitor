# Research

## Audit Method

For each repo, inspect:

- current branch and tracking status
- `git status --short --branch`
- `git stash list`
- `git worktree list`
- shallow tmp/worktree/session directory sweep

## Key Finding

No scoped repo is clean enough for immediate PR creation. The blockers are local-state blockers:

- mixed governance plus unrelated runtime changes
- stale or prunable linked worktrees
- active stash layers
- duplicate or drifting canonical status surfaces
- governance bootstrap work still living on `main` in some repos

## Strongest PR-Prep Candidate

`agentapi-plusplus` and `heliosCLI` are the closest repos to PR-refresh-ready because:

- both already sit on active feature branches with existing PR lanes
- local drift is relatively contained compared with `AgilePlus` and `phenotype-infrakit`
- neither has the extreme stash debt seen in the shelf-root repo

## Highest Fragmentation Risk

`AgilePlus`, `thegent`, and `phenotype-infrakit` are the most fragmented:

- `AgilePlus` due to mixed workflow deletions, DB/runtime churn, and governance/spec additions on
  `main`
- `thegent` due to branch drift versus `origin/main` plus mixed code/docs/governance changes
- `phenotype-infrakit` due to a 39-entry stash stack and multiple linked worktrees
