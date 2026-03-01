---
audience: [developers, pms]
---

# Retrospectives

Structured reflection on completed features to improve future iterations.

## When to Run

After a feature is merged and shipped, run a retrospective to capture learnings.

## Process

### 1. Gather Data

Review the feature's lifecycle:

```bash
agileplus status 001 --history
```

This shows:
- Time spent in each phase
- Number of review cycles
- Blockers encountered
- Agent vs. human contribution ratio

### 2. Identify Patterns

| Category | Questions |
|----------|-----------|
| **Went well** | What saved time? What worked smoothly? |
| **Could improve** | Where were bottlenecks? What caused rework? |
| **Action items** | What should we change for next time? |

### 3. Update Constitution

If the retrospective reveals process gaps:

```bash
agileplus constitution --update
```

Encode learnings as new governance rules or quality standards.

## Metrics

Key metrics to track across features:

| Metric | What It Measures |
|--------|-----------------|
| **Cycle time** | Specify → Shipped duration |
| **Review rounds** | How many review cycles per WP |
| **Spec accuracy** | Requirements changed during implementation |
| **Agent effectiveness** | WPs completed by agents vs. humans |
| **Rework rate** | WPs sent back from review |

## Storing Retrospectives

Retrospective notes are stored in:

```
kitty-specs/001-feature/
└── retrospective.md
```
