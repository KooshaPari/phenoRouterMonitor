---
audience: [developers, pms]
---

# Triage Workflow Example

Walk through classifying incoming items and managing the backlog.

## Scenario

Your team receives several requests throughout the day. AgilePlus triages them automatically.

## Incoming Items

### Bug report

```bash
agileplus triage "users can't log in after password reset - getting 500 error"
```

```
Intent:      Bug
Confidence:  92%
Keywords:    error

Added to backlog as Bug item.
```

### Feature request

```bash
agileplus triage "would be great to have dark mode in the dashboard"
```

```
Intent:      Feature
Confidence:  78%
Keywords:    have

Added to backlog as Feature item.
```

### Vague idea

```bash
agileplus triage "what if we integrated with Slack for notifications"
```

```
Intent:      Idea
Confidence:  81%
Keywords:    what if

Added to backlog as Idea item.
```

### Maintenance task

```bash
agileplus triage "need to update Node from 18 to 22"
```

```
Intent:      Task
Confidence:  88%
Keywords:    update

Added to backlog as Task item.
```

## Review the Queue

```bash
agileplus queue list
```

```
Backlog queue:
  #1  [Bug]     users can't log in after password reset    Priority: High
  #2  [Feature] dark mode in the dashboard                 Priority: Medium
  #3  [Task]    update Node from 18 to 22                  Priority: Medium
  #4  [Idea]    integrate with Slack for notifications      Priority: Low
```

## Work the Queue

Pop the highest-priority item:

```bash
agileplus queue pop
```

```
Popped: #1 [Bug] users can't log in after password reset (Priority: High)
```

This item can now be fed into the spec pipeline:

```bash
agileplus specify --title "Fix post-reset login 500" --description "Users get 500 error when logging in after password reset"
```

## Filter by Type

```bash
agileplus queue list --type bug
agileplus queue list --status new --type feature
```

## Override Classification

If the classifier gets it wrong:

```bash
agileplus triage "refactor the auth module" --type task
```

The `--type` flag overrides automatic classification.
