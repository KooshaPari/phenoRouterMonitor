---
audience: [developers, agents, pms]
---

# Specify

The entry point for every feature. `specify` transforms a natural language description into a structured specification.

## What It Does

1. Runs a **discovery interview** — scoped to feature complexity
2. Generates a `spec.md` with requirements, user scenarios, success criteria
3. Creates `meta.json` with feature metadata
4. Commits to the target branch

## Usage

```bash
agileplus specify "Add checkout upsell flow"
```

Or interactively (no arguments):

```bash
agileplus specify
# → Enters discovery interview mode
```

## Spec Structure

```
kitty-specs/001-checkout-upsell/
├── meta.json       # Feature metadata
└── spec.md         # Specification document
```

### spec.md Sections

| Section | Purpose |
|---------|---------|
| Overview | What and why |
| User Scenarios | Who uses it, how |
| Functional Requirements | Testable requirements |
| Success Criteria | Measurable outcomes |
| Assumptions | Documented defaults |
| Key Entities | Data model (if applicable) |

## Guidelines

- Focus on **what** and **why** — never how
- Write for non-technical stakeholders
- Every requirement must be testable
- Success criteria must be measurable and technology-agnostic
- Maximum 3 `[NEEDS CLARIFICATION]` markers for truly ambiguous items

## Next Steps

After specifying: [Clarify](/workflow/clarify) → [Research](/workflow/research) → [Plan](/workflow/plan)
