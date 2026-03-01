---
audience: [developers, pms]
---

# Triage & Queue

AgilePlus includes a rule-based triage system that classifies incoming items and routes them to a priority backlog.

## Classifying Items

```bash
agileplus triage "login page crashes on mobile Safari"
```

Output:
```
Intent:      Bug
Confidence:  85%
Keywords:    crashes

Added to backlog as Bug item.
```

The classifier uses **weighted keyword matching** across four intent categories:

| Intent | Keywords (sample) |
|--------|-------------------|
| Bug | crash, error, fail, broken, fix |
| Feature | add, new, implement, support |
| Idea | what if, could we, brainstorm |
| Task | update, migrate, refactor, clean |

### Override Classification

```bash
agileplus triage "update dependencies" --type task
```

### Dry Run

Classify without adding to backlog:

```bash
agileplus triage "add dark mode support" --dry-run
```

### JSON Output

```bash
agileplus triage "crash on startup" --output json
```

```json
{
  "intent": "Bug",
  "confidence": 0.9,
  "matched_keywords": ["crash"]
}
```

## Managing the Queue

### Add Items

```bash
agileplus queue add --title "Fix Safari crash" --type bug
agileplus queue add --title "Add dark mode" --description "Support system-level dark mode preference"
```

If `--type` is omitted, the title is auto-classified.

### List Items

```bash
agileplus queue list
agileplus queue list --status new --type bug
agileplus queue list --output json
```

### Show Details

```bash
agileplus queue show 42
```

### Pop Next Item

Pop the highest-priority item from the queue:

```bash
agileplus queue pop
```

Priority order: **Critical > High > Medium > Low**. Within the same priority, older items come first (FIFO).

## Priority Assignment

Items are automatically assigned priority based on intent:

| Intent | Default Priority |
|--------|-----------------|
| Bug | High |
| Feature | Medium |
| Task | Medium |
| Idea | Low |
