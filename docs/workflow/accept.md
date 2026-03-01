---
audience: [developers, pms]
---

# Accept

Final validation that a feature meets all specification requirements.

## What It Does

1. Verifies all WPs are in `done` lane
2. Runs the acceptance checklist from the spec
3. Validates success criteria are met
4. Confirms the feature is ready to merge

## Usage

```bash
agileplus accept 001
```

## Acceptance Criteria

The accept phase checks:

- Every functional requirement from the spec has a corresponding implementation
- All success criteria are demonstrably met
- No open `[NEEDS CLARIFICATION]` markers remain
- All work packages passed review

## After Acceptance

Once accepted, the feature moves to [Merge](/workflow/merge):

```bash
agileplus merge 001
```
