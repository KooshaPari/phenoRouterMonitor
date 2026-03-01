# Spec-Driven Development

Spec-driven development inverts the typical development workflow. Instead of jumping from idea to code, every feature begins as a **specification** — a structured document that defines what gets built, why, and what success looks like.

## The Problem

Most development workflows fail in predictable ways:

- **Scope creep** — features grow unbounded because nobody defined boundaries
- **Audit gaps** — no record of why decisions were made
- **Agent chaos** — AI agents generate code without governance
- **Integration friction** — work packages conflict because they weren't planned together

## The AgilePlus Approach

AgilePlus enforces a 7-stage pipeline where each transition is **governed** and **auditable**:

```
specify → research → plan → implement → validate → ship → retrospective
```

Every feature moves through a state machine. You can't skip stages. You can't ship unvalidated work. Every state transition is recorded with a SHA-256 hash chain.

## Key Principles

### Specifications First

A feature doesn't exist until it has a spec. The spec defines:
- What the feature does (functional requirements)
- Who it's for (actors and scenarios)
- How to verify it works (acceptance criteria)

### Work Packages

Large features are decomposed into **work packages** (WPs) — small, independently implementable units with clear dependencies. Each WP gets its own git branch and can be assigned to a different agent.

### Governance by Default

Every state transition requires preconditions. You can't move a feature to `implementing` without a plan. You can't `ship` without validation passing. The system enforces this — not the developer.

### Agent-Agnostic

AgilePlus doesn't care which agent writes the code. Claude Code, Cursor, Codex, Copilot — they all receive the same structured prompts and are held to the same governance standards.

## What This Means in Practice

A developer (or agent) working with AgilePlus never writes code without context:

1. The **spec** tells them what to build
2. The **plan** tells them how to decompose it
3. The **work package** tells them exactly which piece to implement
4. The **validation** tells them if they got it right
5. The **audit trail** tells everyone else what happened
