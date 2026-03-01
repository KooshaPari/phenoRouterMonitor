---
audience: [agents]
---

# Agent Prompt Format

AgilePlus communicates with AI agents through structured prompts. This page documents the prompt format agents receive and the expected response format.

## Prompt Structure

Every agent prompt contains these sections:

```
# Task: [WP ID] — [Title]

## Context
- Feature: [feature name and number]
- Spec: [link to spec.md]
- Plan: [link to plan.md]

## Deliverables
1. [File or artifact to produce]
2. [Tests to write]

## Constraints
- [Governance rules]
- [Architecture boundaries]
- [Dependency restrictions]

## Completion
When done, run:
  spec-kitty agent tasks move-task WP## --to for_review
```

## Response Expectations

Agents should:

1. **Read the spec and plan** before writing code
2. **Work in the assigned worktree** — never write to the main repo
3. **Commit before moving to review** — uncommitted changes block the transition
4. **Follow existing patterns** — match the codebase's style and conventions

## Frontmatter Tags

Agents can read frontmatter from any docs page:

```yaml
---
audience: [agents]      # This page is for agents
---
```

When `audience` includes `agents`, the page contains information relevant to AI agent workflows.

## Error Handling

If an agent encounters a blocker:

1. Document the blocker in a task comment
2. Create a new task describing what needs to be resolved
3. Do **not** mark the task as completed
