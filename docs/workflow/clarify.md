---
audience: [developers, agents, pms]
---

# Clarify

Identify and resolve underspecified areas in a feature specification through targeted questions.

## What It Does

1. Scans the spec for ambiguity, missing edge cases, and unclear requirements
2. Generates up to **5 targeted clarification questions**
3. Encodes answers back into the spec, replacing `[NEEDS CLARIFICATION]` markers
4. Validates the updated spec against quality criteria

## Usage

```bash
agileplus clarify 001
```

## Question Categories

Questions are prioritized by impact:

1. **Scope** — What's included/excluded?
2. **Outcomes** — What does success look like?
3. **Risks & Security** — What could go wrong?
4. **User Experience** — How should edge cases feel?
5. **Technical constraints** — Any hard limits?

## Example

```
Q1: User Roles
Context: Spec mentions "users can manage their account"
Need: Which user roles have account management access?
Options: (A) All users · (B) Admin + Owner · (C) Role-based with config

> Answer: B — Admin and Owner roles only
```

The answer replaces the ambiguity in the spec with a concrete requirement.

## When to Skip

If the spec was generated with full context and no `[NEEDS CLARIFICATION]` markers remain, clarify will report the spec is ready and suggest moving to [Research](/workflow/research) or [Plan](/workflow/plan).
