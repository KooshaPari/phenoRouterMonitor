---
audience: [developers, agents]
---

# Research

Phase 0 investigation that produces evidence-based technical decisions before planning.

## What It Does

1. Scans the codebase for relevant patterns and existing code
2. Evaluates feasibility of the spec requirements
3. Identifies dependencies, risks, and integration points
4. Produces research artifacts in the feature directory

## Usage

```bash
agileplus research 001
```

## Output

```
kitty-specs/001-feature/
└── research/
    ├── codebase-scan.md     # Relevant existing code
    ├── feasibility.md       # Can we build this? Risks?
    └── decisions.md         # Technical decisions with rationale
```

## Research Questions

The research phase answers:

- What existing code can we reuse?
- What patterns does the codebase already follow?
- Are there dependency conflicts?
- What are the integration boundaries?
- What's the estimated complexity?

## When to Use

Research is most valuable for:

- Features touching unfamiliar parts of the codebase
- Integration with external systems
- Performance-sensitive features
- Features with unclear technical feasibility

For simple features with well-understood scope, you can skip directly to [Plan](/workflow/plan).
