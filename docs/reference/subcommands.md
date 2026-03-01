---
audience: [agents, developers]
---

# Sub-commands

AgilePlus includes 25 hidden sub-commands across 8 categories for advanced agent workflows.

## Categories

| Category | Commands | Purpose |
|----------|----------|---------|
| `branch` | create, checkout, delete | Branch management |
| `commit` | create, amend, fixup | Commit operations |
| `diff` | show, stat | Diff inspection |
| `stash` | push, pop, list | Stash management |
| `worktree` | add, remove, list | Worktree operations |
| `artifact` | write, read, hash | Artifact management |
| `governance` | check, enforce | Governance validation |
| `audit` | log, query | Audit trail queries |

Sub-commands are hidden from `--help` but available for agent dispatch. Each invocation is recorded in the append-only JSONL audit log.
