# Worklogs Index

**Version:** 1.0 | **Date:** 2026-03-29 | **Status:** Active

---

## Overview

Worklogs are organized by category for easy aggregation into project-level worklogs. Each category contains worklog entries that can be reviewed independently or aggregated.

## Category Structure

```
worklogs/
├── README.md                    # This index
├── ARCHITECTURE.md              # Architecture decisions & reviews
├── DUPLICATION.md               # Cross-project duplication findings
├── INTEGRATION.md               # Integration worklogs
├── PERFORMANCE.md               # Performance analysis & optimizations
├── RESEARCH.md                  # Research findings & starred repo analysis
└── GOVERNANCE.md                # Governance & policy worklogs
```

## Category Summaries

### ARCHITECTURE.md
- Hexagonal architecture reviews
- ADR decisions
- Library extraction candidates
- System design patterns

### DUPLICATION.md
- Cross-crate duplication findings
- Intra-repo duplication audits
- Library libification candidates
- Code smell analysis

### INTEGRATION.md
- External system integrations
- MCP server worklogs
- gRPC/proto worklogs
- Plane.so, GitHub, NATS sync

### PERFORMANCE.md
- Performance analysis
- Benchmark results
- Optimization opportunities
- Resource utilization

### RESEARCH.md
- Starred repo analysis
- Tool evaluation
- Gap analysis
- Technology recommendations

### GOVERNANCE.md
- Policy implementations
- Compliance tracking
- Evidence collection
- Quality gates

---

## Usage

### Reading Worklogs

```bash
# Read all worklogs
cat worklogs/*.md

# Read specific category
cat worklogs/DUPLICATION.md

# Read recent entries
grep -A 20 "## 2026-03-29" worklogs/*.md
```

### Aggregating by Project

Each entry includes project tags for filtering:
- `[AgilePlus]` - Core AgilePlus work
- `[thegent]` - TheGent work
- `[heliosCLI]` - HeliosCLI work
- `[cross-repo]` - Cross-repo work

### Entry Format

```markdown
## YYYY-MM-DD - Entry Title

**Project:** [project-tag]
**Category:** category-name
**Status:** [in_progress|completed|blocked|pending]
**Priority:** [P0|P1|P2|P3]

### Summary
Brief description of work done.

### Tasks Completed
- [ ] Task 1
- [x] Task 2

### Next Steps
- [ ] Follow-up task

### Related
- Links to specs, PRs, sessions
```

---

## Maintenance

- Entries are added chronologically (newest first)
- Each entry should be self-contained
- Include links to relevant files/sessions
- Mark entries as complete when work is done
- Archive stale entries quarterly

---

## Aggregation Guide

To create a project-level worklog:

```bash
# For AgilePlus
grep -h "\[AgilePlus\]" worklogs/*.md | sort

# For thegent
grep -h "\[thegent\]" worklogs/*.md | sort

# For cross-repo
grep -h "\[cross-repo\]" worklogs/*.md | sort
```
