# DAG / WBS

## Phase Order

| Phase | Task ID | Description | Depends On |
|---|---|---|---|
| 1 | AUD-01 | Audit active repo status, upstream divergence, and open-PR branches | - |
| 1 | AUD-02 | Audit worktrees, temp clones, and stash inventory | - |
| 2 | CLS-01 | Classify repos into `PR-ready`, `needs-fix`, `blocked`, `archive` | AUD-01, AUD-02 |
| 2 | CLS-02 | Identify safe structural cleanup items | AUD-02 |
| 3 | FIX-01 | Prune broken linked-worktree metadata in `heliosCLI` | CLS-02 |
| 3 | FIX-02 | Build canonical PR-prep queue for active repos | CLS-01 |
| 4 | PRP-01 | Narrow `agentapi-plusplus` branch to governance-only update set | FIX-02 |
| 4 | PRP-02 | Narrow `thegent` branch to governance-only update set | FIX-02 |
| 4 | PRP-03 | Narrow `heliosCLI` branch to governance-and-doc update set | FIX-01, FIX-02 |
| 5 | PRP-04 | Re-evaluate `cliproxyapi-plusplus` and `heliosApp` after branch narrowing on higher-priority repos | PRP-01, PRP-02, PRP-03 |
| 5 | REC-01 | Start `phenotype-infrakit` stash/worktree recovery lane | FIX-02 |

## Execution Queue

1. `agentapi-plusplus`
2. `heliosCLI`
3. `cliproxyapi-plusplus`
4. `thegent`
5. `heliosApp`
6. `forgecode`
7. `cloud`
8. `AgilePlus`
9. `phenotype-infrakit`

## Rationale

- `agentapi-plusplus` and `heliosCLI` already have open PR branches and the governance updates are coherent enough to isolate first.
- `cliproxyapi-plusplus` and `thegent` also have active PR branches, but current local drift is riskier.
- `heliosApp` is still viable after lane clarification.
- `forgecode` and `cloud` are not PR lanes until they move off `main`.
- `AgilePlus` and `phenotype-infrakit` are currently too entangled with local runtime, generated data, and stash/worktree debt to treat as immediate PR lanes.
