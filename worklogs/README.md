# Worklogs

Agent-format worklogs organized by category for the Phenotype projects.

## Categories

| Category | File | Entries | Focus |
|----------|------|---------|-------|
| **Architecture** | `ARCHITECTURE.md` | 40 | Patterns, decisions, library extraction |
| **Duplication** | `DUPLICATION.md` | 40 | Cross-project duplication audits |
| **Dependencies** | `DEPENDENCIES.md` | 40 | 3rd party integration, fork candidates |
| **Integration** | `INTEGRATION.md` | 40 | MCP, external integrations |
| **Performance** | `PERFORMANCE.md` | 40 | Optimization, benchmarking |
| **Research** | `RESEARCH.md` | 40 | Technology radar, starred repos |
| **Governance** | `GOVERNANCE.md` | 40 | Policy, compliance, quality |

## Entry Format

```markdown
## YYYY-MM-DD - Entry Title

**Project:** [AgilePlus]|[heliosCLI]|[thegent]|[cross-repo]
**Category:** category-name
**Status:** [in_progress|completed|pending|blocked]
**Priority:** P0|P1|P2|P3

### Summary
Brief description of the work.

### Tasks Completed
- [x] Completed task
- [ ] Next task

### Related
- `docs/spec.md`
- `plans/plan.md`
```

## Aggregation

Use `aggregate.sh` to view worklogs by different dimensions:

```bash
# View by project
./worklogs/aggregate.sh projects

# View by priority
./worklogs/aggregate.sh priority

# View by category
./worklogs/aggregate.sh category

# View all entries
./worklogs/aggregate.sh all
```

## Priority Levels

| Priority | Description | SLA |
|----------|-------------|-----|
| **P0** | Critical/Blocking | 24 hours |
| **P1** | High priority | 1 week |
| **P2** | Medium priority | 2 weeks |
| **P3** | Low priority | 1 month |

## Project Tags

| Tag | Project |
|-----|---------|
| `[AgilePlus]` | AgilePlus platform |
| `[heliosCLI]` | heliosCLI terminal |
| `[thegent]` | thegent agent system |
| `[cross-repo]` | Cross-cutting work |

## Adding Worklog Entries

1. Identify the appropriate category file
2. Create entry with required format
3. Set priority and status
4. Add related links
5. Update aggregate if needed

## Workflow

```
Idea -> Research -> Plan -> Implement -> Document
   |         |        |         |         |
   v         v        v         v         v
Research   Research  Plan     Work     Worklog
           entry    entry    entry    entry
```

## Integration with Plans

Worklogs aggregate into plans:
- `plans/2026-03-29-CROSS_PROJECT_DUPLICATION_PLAN-v1.md`
- `plans/2026-03-29-FORK_EXECUTION_PLAN-v1.md`
- etc.

## Status Meanings

| Status | Meaning |
|--------|---------|
| `pending` | Not started, not blocked |
| `in_progress` | Currently being worked |
| `completed` | Done, no further action |
| `blocked` | Waiting on external dependency |
