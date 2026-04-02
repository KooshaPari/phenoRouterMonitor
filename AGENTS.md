# AGENTS.md — repos shelf agent rules

This is the repos shelf — a polyrepo containing many independent projects. Agents working in this shelf must understand the difference between shelf-level work and project-level work.

## Shelf Identity

This shelf is a collection of independent git repositories. Never treat the shelf as a single project. Always identify which project you're working in before taking action.

## AgilePlus Mandate

All work that uses AgilePlus MUST be tracked in the AgilePlus system when a project integrates with it. Reference locations (examples):
- AgilePlus repository: /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
- AgilePlus CLI usage: cd AgilePlus && agileplus <command>

Work Requirements (example guidance):
1. Check for an AgilePlus spec before implementing.
2. Create a spec for new work: agileplus specify --title "<feature>" --description "<desc>"
3. Update work package status: agileplus status <feature-id> --wp <wp-id> --state <state>
4. Prefer attaching work to an AgilePlus spec where the project uses it.

## Agent Self-Reference

| Agent | Role | Default location |
|-------|------|-----------------|
| Forge | Main coding/impl agent | `@shelf/*` |
| Muse  | Code review / quality | `@shelf/*` |
| Sage  | Research / investigation | `@shelf/*` |
| Helios| Runtime / testing specialist | `@shelf/*` |

Agents should identify themselves at the start of significant tasks:

⏺ [HH:MM:SS] AgentName working on <project>:<task>

## Project Navigation Rules

DO THIS FIRST when starting any task:
1. If the user mentions a project, use it.
2. If the current working directory contains a `.git/` subdirectory, assume project-level work.
3. If the task references a path like `heliosCLI/...` or `thegent/...`, use that project as the scope.
4. Otherwise, assume shelf-level work and ask for clarification.

Working in a project:
- Run all tests from inside the project directory.
- File paths in tasks should be project-relative.
- Respect project-specific dotfiles and tooling.

## File Size & Modularity Mandate

Hard constraints (shelf guidance):
- Target file size: <= 350 lines.
- Hard limit: <= 500 lines for source files.

If a file approaches the target limit, decompose it into smaller modules following documented patterns (service submodule, adapter extraction, route splitting, model split).

## Standard Operating Loop

For each task: Review → Research → Plan → Execute → Size-Check → Test → Review & Polish → Repeat.

Key points:
- Research first (search codebase, tests, docs).
- Implement in small, verifiable increments.
- Tests must precede or accompany implementation per project testing standards.
- Avoid placeholder TODOs or "AI slop".

## Branch Discipline

- Feature branches and worktrees: use worktrees where appropriate per project guidance (worktrees/<project>/...)
- Canonical repository should track `main` only; return to `main` for merge/integration checkpoints.
- Branch naming: feat/, fix/, chore/, docs/ prefixes.
- Avoid force-pushing shared branches. Personal feature branches may be force-pushed if appropriate.

## Read-Only Tools First

Before modifying code: read files, search the codebase, and verify state with non-destructive commands. Then make changes with full context.

## Quality Standards (high-level)

- Linters and type-checkers should pass per project standards.
- Tests must pass in CI.
- New suppressions require inline justification.

## Session Documentation

When performing multi-step or research work, maintain session documentation in docs/sessions/<session-id>/ with the recommended structure: 01_RESEARCH.md, 02_PLAN.md, 03_IMPLEMENTATION.md, 04_VALIDATION.md, 05_KNOWN_ISSUES.md.

## Where to look for canonical governance resources

- platforms/thegent/governance/AGENTS.base.md — canonical base for agent rules
- thegent/governance/standards/* — code-style.md, testing-standards.md, pr-standards.md
- governance/templates/NEW_PROJECT_CHECKLIST.md — new project checklist

---

_Last updated: 2026-04-02_
