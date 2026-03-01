---
audience: [developers, agents, pms]
---

# Plan

Generate an implementation blueprint from the specification and research artifacts.

## What It Does

1. Analyzes the spec, research, and existing codebase
2. Produces a `plan.md` with architecture decisions
3. Identifies files to create/modify
4. Defines the build sequence and dependency graph

## Usage

```bash
agileplus plan 001
```

## Plan Structure

```markdown
# Implementation Plan: Feature Name

## Architecture Decisions
- Decision 1: Rationale
- Decision 2: Rationale

## File Changes
| File | Action | Purpose |
|------|--------|---------|
| src/auth/login.rs | Create | Login endpoint handler |
| src/auth/mod.rs | Modify | Register login route |

## Build Sequence
1. Data models first
2. Business logic
3. API endpoints
4. Tests

## Dependencies
- External: jwt crate v0.16
- Internal: agileplus-domain Feature entity
```

## From Plan to Tasks

After the plan is approved, generate work packages:

```bash
agileplus tasks 001
```

This decomposes the plan into parallel-safe work packages with dependency tracking.

## Next Steps

[Tasks](/workflow/tasks) → [Implement](/workflow/implement) → [Review](/workflow/review)
