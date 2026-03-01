---
audience: [developers, pms, agents]
---

# Status & Dashboard

Monitor project progress across features and work packages.

## CLI Status

```bash
agileplus status
```

Shows the kanban board for all active features:

```
Feature 001: checkout-upsell (Implementing)
  planned    │ doing      │ for_review │ done
  ───────────┼────────────┼────────────┼──────────
  WP04       │ WP03       │ WP02       │ WP01
             │            │            │
```

### Per-Feature Status

```bash
agileplus status 001
```

Shows detailed WP progress with subtask completion:

```
WP01  Models & migrations     ████████████ done      (6/6 subtasks)
WP02  API endpoints           ████████░░░░ for_review (5/6 subtasks)
WP03  UI components           ██████░░░░░░ doing     (3/6 subtasks)
WP04  Integration tests       ░░░░░░░░░░░░ planned   (blocked by WP02, WP03)
```

## Web Dashboard

```bash
agileplus dashboard
```

Opens an interactive browser dashboard with:
- Kanban board with drag-and-drop
- Feature timeline visualization
- Agent activity log
- Audit trail viewer

## Spec Index

View all specifications:

```bash
agileplus specs
```

```
# │ Feature              │ State        │ WPs  │ Last Updated
──┼──────────────────────┼──────────────┼──────┼─────────────
1 │ checkout-upsell      │ Implementing │ 4/4  │ 2 hours ago
2 │ user-auth            │ Planned      │ 0/3  │ 1 day ago
3 │ reporting-dashboard  │ Specified    │ —    │ 3 days ago
```
