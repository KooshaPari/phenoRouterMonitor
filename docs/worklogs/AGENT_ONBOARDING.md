# Agent Onboarding: Worklog System

**Version:** 1.0 | **Date:** 2026-03-29 | **Audience:** All Agents (Forge, Muse, Sage, Helios)

---

## Overview

The **repos shelf** uses a categorized worklog system for tracking research, decisions, and work across multiple projects. All agents MUST use this system when working on tasks that produce findings, decisions, or work products.

## Why Worklogs?

1. **Aggregation** - Work can be aggregated by project, priority, or category
2. **Discoverability** - Other agents can find relevant context quickly
3. **Continuity** - Agents can resume work from previous sessions
4. **Transparency** - Humans can see what agents are working on

## Worklog Categories

| Category | File | Purpose |
|----------|------|---------|
| **ARCHITECTURE** | `worklogs/ARCHITECTURE.md` | ADRs, library extraction, system design |
| **DUPLICATION** | `worklogs/DUPLICATION.md` | Cross-project duplication audits |
| **DEPENDENCIES** | `worklogs/DEPENDENCIES.md` | External deps, forks, package modernization |
| **INTEGRATION** | `worklogs/INTEGRATION.md` | External integrations, MCP work |
| **PERFORMANCE** | `worklogs/PERFORMANCE.md` | Optimization, benchmarking |
| **RESEARCH** | `worklogs/RESEARCH.md` | Starred repo analysis, technology radar |
| **GOVERNANCE** | `worklogs/GOVERNANCE.md` | Policy, evidence, quality gates |

## Entry Format

Every worklog entry MUST follow this format:

```markdown
## YYYY-MM-DD - Entry Title

**Project:** [AgilePlus]|[thegent]|[heliosCLI]|[heliosApp]|[cross-repo]
**Category:** category-name
**Status:** [in_progress|completed|blocked|pending]
**Priority:** [P0|P1|P2|P3]

### Summary
Brief description of the work.

### Key Findings
- Finding 1
- Finding 2

### Tasks Completed
- [x] Task 1
- [ ] Task 2

### Next Steps
- [ ] Follow-up task

### Related
- Links to specs, PRs, sessions, or files
```

## When to Write Worklogs

Write a worklog entry when you:

1. **Complete research** - Any starred repo analysis, technology evaluation
2. **Make decisions** - ADRs, architectural choices, fork decisions
3. **Find issues** - Duplication, performance problems, integration gaps
4. **Complete work** - Implementation milestones, feature completions
5. **Plan work** - Fork candidates, migration plans, enhancement recommendations

## Project Tags

Always tag entries with the relevant project:

| Tag | Project | Examples |
|-----|---------|----------|
| `[AgilePlus]` | AgilePlus Rust monorepo | Feature work, CLI, API |
| `[thegent]` | TheGent dotfiles manager | Dotfiles, system setup |
| `[heliosCLI]` | HeliosCLI framework | CLI utilities, commands |
| `[heliosApp]` | HeliosApp application | Frontend, UI work |
| `[cross-repo]` | Cross-repo work | Audits, integrations, decisions |

## Priority Guidelines

| Priority | When to Use |
|----------|-------------|
| **P0** | Critical blockers, security issues, fundamental decisions |
| **P1** | Important work, high-value improvements |
| **P2** | Medium priority, nice-to-have improvements |
| **P3** | Low priority, future considerations |

## Status Guidelines

| Status | Meaning |
|--------|---------|
| **in_progress** | Currently working on this |
| **completed** | Work finished, all tasks done |
| **blocked** | Waiting on dependencies or decisions |
| **pending** | Planned but not started |

## Quick Commands

```bash
# Read all worklogs
cat worklogs/*.md

# Read by category
cat worklogs/DUPLICATION.md

# Read by priority
./worklogs/aggregate.sh priority

# Read by project
./worklogs/aggregate.sh project

# Read all entries
./worklogs/aggregate.sh all
```

## Agent Responsibilities

### Forge (Implementation Agent)
- Write worklogs for implementation findings
- Update DUPLICATION.md when finding copy-paste code
- Update DEPENDENCIES.md for dependency changes

### Muse (Code Review Agent)
- Write worklogs for code quality findings
- Update ARCHITECTURE.md for design feedback
- Update GOVERNANCE.md for quality gate findings

### Sage (Research Agent)
- Write worklogs for all research activities
- Update RESEARCH.md with starred repo analysis
- Update PERFORMANCE.md with optimization research

### Helios (Testing Agent)
- Write worklogs for testing findings
- Update INTEGRATION.md for integration test results
- Update PERFORMANCE.md for benchmark results

## Example Worklog Entries

### Research Entry
```markdown
## 2026-03-29 - khoj-ai/khoj Analysis

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P1

### Summary
Analyzed Khoj AI knowledge base for semantic search capabilities.

### Key Findings
- Local-first with embeddings
- Multiple interfaces (web, Obsidian, Emacs)
- RAG pipeline support

### Next Steps
- [ ] Create knowledge-base repo spec
- [ ] Prototype Khoj integration
```

### Decision Entry
```markdown
## 2026-03-29 - Fork Decision: utils/pty

**Project:** [cross-repo]
**Category:** architecture
**Status:** completed
**Priority:** P0

### Decision
Fork `utils/pty` to `phenotype-process` for shared PTY/process management.

### Rationale
- Used in 3+ repos
- Cross-platform quirks need phenotype-specific fixes
- Adds worktree-specific utilities

### Risks
- Maintenance burden for forked code
- Mitigation: Keep fork minimal, contribute upstream
```

## Tips

1. **Be concise** - Entries should be scannable
2. **Link liberally** - Include file paths, PR URLs, session paths
3. **Update status** - Mark entries completed when done
4. **Follow format** - Use the template for consistency
5. **Check existing** - Before adding, check if similar entry exists

## Questions?

Ask in the current session or check:
- `worklogs/README.md` - Full worklog documentation
- `worklogs/aggregate.sh` - Aggregation script
- `PLAN.md` - Implementation plan with priorities
