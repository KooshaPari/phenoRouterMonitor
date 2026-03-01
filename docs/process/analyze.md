---
audience: [developers, pms]
---

# Analyze

Non-destructive cross-artifact consistency and quality check.

## What It Does

Scans `spec.md`, `plan.md`, and `tasks.md` for:

- **Consistency** — do the plan and tasks match the spec?
- **Completeness** — are all requirements covered by work packages?
- **Dependency correctness** — are WP dependencies properly declared?
- **Quality** — do artifacts meet the constitution's standards?

## Usage

```bash
agileplus analyze 001
```

## Output

```
Feature 001: checkout-upsell

Consistency
  ✓ All spec requirements mapped to WPs
  ✓ Plan architecture matches task deliverables
  ⚠ WP03 mentions "email service" not in spec

Completeness
  ✓ 12/12 functional requirements covered
  ✗ Success criterion SC-3 has no corresponding test

Dependencies
  ✓ No circular dependencies
  ✓ All blocked WPs have valid blockers

Quality
  ✓ Spec passes checklist (14/14)
  ⚠ Plan missing "Build Sequence" section
```

## When to Use

Run analyze:
- After generating tasks (before implementation)
- After major spec changes
- Before acceptance
- As a health check at any point in the lifecycle

Analyze is **read-only** — it never modifies artifacts.
