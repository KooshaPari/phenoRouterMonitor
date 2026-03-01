---
audience: [agents]
---

# Governance Constraints

Rules and boundaries that AI agents must follow when working within AgilePlus.

## Core Rules

1. **Spec is source of truth** — never contradict the specification
2. **Work in worktrees** — never commit directly to main
3. **One WP at a time** — complete and review before starting the next
4. **No force pushes** — all history must be preserved
5. **No secret commits** — every change must be traceable to a WP

## File Boundaries

Agents may only modify files listed in their work package deliverables. Exceptions:

- Lock files (e.g., `bun.lockb`) may be updated as a side effect of dependency changes
- Configuration files may be updated if the WP explicitly requires it

## Forbidden Actions

| Action | Reason |
|--------|--------|
| `git push --force` | Destroys history |
| `git reset --hard` | Loses uncommitted work |
| `rm -rf` on project dirs | Irreversible data loss |
| Modifying `CLAUDE.md` | Governance file — requires human approval |
| Skipping pre-commit hooks | Bypasses quality gates |

## Review Protocol

Before submitting for review, agents must verify:

- [ ] All deliverable files exist
- [ ] Tests pass locally
- [ ] No files outside WP scope were modified
- [ ] Commit messages reference the WP ID
- [ ] Worktree has commits beyond main

## Escalation

If a governance rule conflicts with the task requirements, the agent must:

1. Stop work
2. Document the conflict
3. Wait for human resolution
