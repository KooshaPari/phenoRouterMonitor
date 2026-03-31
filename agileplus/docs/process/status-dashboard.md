---
audience: [developers, pms, agents]
---

# Status & Dashboard

Monitor project progress across features and work packages in real-time. AgilePlus provides both CLI and web-based views for tracking development.

## CLI Status Commands

### All Features

```bash
agileplus status
```

Shows kanban board across all features:

```
┌─ FEATURE KANBAN ───────────────────────────────────┐
│                                                     │
│ Feature 001: Checkout Upsell (Implementing)        │
│                                                     │
│  planned    │ doing       │ for_review │ done       │
│  ──────────────────────────────────────────        │
│  WP04       │ WP03        │ WP02       │ WP01       │
│             │ (claude)    │ (awaiting) │ (merged)   │
│             │             │            │            │
│             │ 2 commits   │ 1 issue    │ 6/6 tests  │
│             │ 50% done    │ awaiting   │            │
│             │             │ review     │            │
│                                                     │
│                                                     │
│ Feature 002: Guest Auth (Specified)                │
│                                                     │
│  ✓ Spec ready for planning                         │
│    123 requirements, 12 success criteria           │
│                                                     │
│                                                     │
│ Feature 003: Payment Webhooks (Planned)            │
│                                                     │
│  ✓ Plan ready for implementation                   │
│    4 WPs, 12-day critical path                     │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Per-Feature Status

```bash
agileplus status 001
```

Shows detailed work package status with timeline:

```
Feature 001: Checkout Upsell

State: Implementing
Spec: Ready (2/2/2026)
Plan: Ready (2/3/2026)
Est. completion: 2/13/2026
Current date: 2/11/2026
Progress: 7/12 days (58%)

┌─ WP Progress ───────────────────┐
│                                  │
│ WP01  Models & migrations        │
│       ████████████░░ done        │
│       Approved: 2/4              │
│       Commits: 4                 │
│       Tests: 12/12 passing       │
│                                  │
│ WP02  API endpoints              │
│       ████████░░░░░░ for_review  │
│       Under review (2/11 9:30am) │
│       Commits: 3                 │
│       Tests: 18/18 passing       │
│       Issues: 0 blocking         │
│                                  │
│ WP03  UI components              │
│       ██████░░░░░░░░ doing       │
│       Assigned: claude-code      │
│       Session: sess_abc123       │
│       Started: 2/11 2:00pm       │
│       Estimated done: 2/12 5pm   │
│       Progress: 50%              │
│                                  │
│ WP04  Integration tests          │
│       ░░░░░░░░░░░░░░░░ planned   │
│       Blocked by: WP02, WP03     │
│       Will start when: 2/12      │
│       Est. duration: 2 days      │
│                                  │
└──────────────────────────────────┘

Dependency Timeline
  WP01 [██████████] (2/4 - 2/6)
    │
    ├─> WP02 [████████░░] (2/6 - 2/8)
    │     │
    │     └─> WP04 [░░░░░] (2/12 - 2/13)
    │
    └─> WP03 [██████░░░░] (2/6 - 2/12)
          │
          └─> WP04 [░░░░░] (2/12 - 2/13)

Critical Path: WP01 → WP02 → WP04 (8 days)
Parallelization: WP02 & WP03 running together
Slack: WP03 has 0 days slack (on critical path)

Agent Activity
  claude-code (WP01, WP02, WP03)
    ✓ WP01 approved (100% pass rate)
    ✓ WP02 approved (100% pass rate)
    ⏳ WP03 in progress (2.5 hours elapsed)

Recent Events
  2/11 9:30am  WP02 moved to for_review
  2/11 2:00pm  WP03 dispatched to claude-code
  2/11 10:00am WP01 moved to done
  2/10 3:15pm  WP02 dispatched to claude-code
```

### Watch Mode

Monitor progress in real-time:

```bash
agileplus status 001 --watch
```

Live-updating terminal UI (refreshes every 5 seconds):

```
Feature 001: Checkout Upsell
┌─────────────────────────────────────────┐
│ WP02: API endpoints          for_review │
│                                          │
│ Under review since: 9:30am (1h 15min)  │
│ Reviewer: @jane                         │
│ Status: Awaiting feedback               │
│                                          │
│ WP03: UI components                doing│
│                                          │
│ Assigned: claude-code                   │
│ Progress: 55% (↑ from 50% 10min ago)   │
│ ETA: 5:30pm (3h 30min remaining)       │
│ Current file: src/components/upsell.rs │
│                                          │
│ Commits since refresh: 1                │
│   feat(WP03): add upsell discount badge│
│                                          │
└─────────────────────────────────────────┘
```

### Filter & Search

```bash
# Show only WPs in "doing" state
agileplus status --lane doing

# Show only blockers
agileplus status --blocked

# Show WPs from a specific agent
agileplus status --agent claude-code

# Show features by state
agileplus status --state implementing --state planned
```

## Web Dashboard

Interactive browser-based dashboard:

```bash
agileplus dashboard
```

Opens at `http://localhost:9090` (default):

### Features View

```
┌─ AgilePlus Dashboard ────────────────────────────┐
│ [Features] [Timeline] [Agents] [Audit] [Settings]│
│                                                   │
│ FEATURES (3 active)                             │
│                                                   │
│ ┌─ 001: Checkout Upsell ──────────────────────┐ │
│ │ State: Implementing                         │ │
│ │ Progress: 58%                               │ │
│ │ [██████████░░░░░░░░░░░░] 7 of 12 days      │ │
│ │                                              │ │
│ │ Lanes: [planned] [doing] [for_review] [done] │ │
│ │                                              │ │
│ │  planned    │ doing    │ for_review │ done  │ │
│ │  ──────────────────────────────────────────  │ │
│ │  WP04      │ WP03     │ WP02       │ WP01   │ │
│ │  [   ]     │ [  ✎  ] │ [  👁️ ]    │ [ ✓ ]  │ │
│ │                                              │ │
│ │ Duration: 8 days (target: 9 days)          │ │
│ │ Status: On track                            │ │
│ │                                              │ │
│ │ [View Details] [Edit Spec] [View Logs]      │ │
│ └──────────────────────────────────────────────┘ │
│                                                   │
│ ┌─ 002: Guest Auth ────────────────────────────┐ │
│ │ State: Specified                            │ │
│ │ Progress: 0%                                │ │
│ │ [░░░░░░░░░░░░░░░░░░░░░░░░░] 0 of 8 days   │ │
│ │ Status: Ready for planning (2/13)           │ │
│ │                                              │ │
│ │ [View Details] [Start Planning]             │ │
│ └──────────────────────────────────────────────┘ │
│                                                   │
│ ┌─ 003: Payment Webhooks ──────────────────────┐ │
│ │ State: Planned                              │ │
│ │ Progress: 0%                                │ │
│ │ [░░░░░░░░░░░░░░░░░░░░░░░░░] 0 of 12 days  │ │
│ │ Status: Ready for implementation (2/14)     │ │
│ │                                              │ │
│ │ [View Details] [Dispatch Agent]             │ │
│ └──────────────────────────────────────────────┘ │
│                                                   │
└─────────────────────────────────────────────────┘
```

### Kanban Board

```
┌─ WP Kanban: Feature 001 ─────────────────────┐
│                                               │
│ Planned         Doing           For Review    │
│ (1 WP)          (1 WP)          (1 WP)       │
│                                               │
│ ┌──────────┐   ┌──────────┐   ┌──────────┐  │
│ │  WP04    │   │  WP03    │   │  WP02    │  │
│ │ Integration  │ UI         │   │ API        │  │
│ │ tests    │   │ components │   │ endpoints  │  │
│ │          │   │          │   │          │  │
│ │ Due: 2/13│   │ Agent:   │   │ Reviewer:│  │
│ │          │   │ claude   │   │ @jane    │  │
│ │ [Drag]   │   │          │   │          │  │
│ │          │   │ Progress:│   │ Awaiting:│  │
│ │          │   │ 55%      │   │ Changes  │  │
│ │          │   │          │   │          │  │
│ │          │   │ [Drag]   │   │ [Drag]   │  │
│ └──────────┘   └──────────┘   └──────────┘  │
│                                               │
│ Done (1 WP)                                   │
│                                               │
│ ┌──────────┐                                  │
│ │  WP01    │                                  │
│ │ Models & │                                  │
│ │ migrations                                  │
│ │          │                                  │
│ │ ✓ Done   │                                  │
│ │ 2/6/26   │                                  │
│ └──────────┘                                  │
│                                               │
└──────────────────────────────────────────────┘
```

Drag cards between lanes to manually move WPs:

```
# Drag WP02 from for_review to done:
✓ WP02 moved to done
  All dependent WPs are now unblocked
```

### Timeline View

```
┌─ Feature Timeline ──────────────────────────┐
│                                             │
│ 001: Checkout Upsell                       │
│ Feb 1 ─────────────────────── Feb 13       │
│ [Spec] [Plan] [WP01] [WP02→→] [WP03→→→]   │
│          └─────────────────────┘            │
│ WP04: Starting 2/12                        │
│                                             │
│ 002: Guest Auth                            │
│ Feb 13 ──────────────────────── Feb 21    │
│ [Spec] [Plan] [WP01] [WP02] [WP03]        │
│ Not yet started                            │
│                                             │
│ 003: Payment Webhooks                      │
│ Feb 14 ──────────────────────── Feb 26    │
│ [Plan] [WP01] [WP02] [WP03] [WP04]        │
│ Earliest start: 2/14                       │
│                                             │
└─────────────────────────────────────────────┘
```

### Agent Activity Log

```
┌─ Agent Activity ────────────────────────────┐
│                                             │
│ claude-code                                 │
│  Working on: WP03 (UI components)         │
│  Started: 2/11 2:00pm                     │
│  Duration: 3h 30m                         │
│  Progress: 55% (3.5 commits)              │
│  ETA: 5:30pm                              │
│                                             │
│ Recent commits:                            │
│  • feat(WP03): add upsell discount badge │
│  • feat(WP03): implement upsell panel     │
│  • test(WP03): add widget tests           │
│                                             │
│ Historical:                                │
│  ✓ WP01 completed (2/6) - 2 days         │
│  ✓ WP02 completed (2/8) - 2.5 days       │
│                                             │
│ Performance:                               │
│  Pass rate: 100% (3/3 WPs approved)      │
│  Avg speed: 2.3 days/WP                   │
│                                             │
└─────────────────────────────────────────────┘
```

## Feature Index

View all specifications:

```bash
agileplus specs
```

```
Feature Specification Index

# │ Feature                │ State        │ WPs  │ Created    │ Last Updated
──┼────────────────────────┼──────────────┼──────┼────────────┼──────────────
1 │ Checkout Upsell        │ Implementing │ 4/4  │ 2026-02-01 │ 2026-02-11 (1h)
2 │ Guest Auth             │ Specified    │ 0/3  │ 2026-02-05 │ 2026-02-10
3 │ Payment Webhooks       │ Planned      │ 4/4  │ 2026-02-08 │ 2026-02-11
4 │ Reporting Dashboard    │ Specified    │ 0/5  │ 2026-01-28 │ 2026-02-01
5 │ Mobile App             │ Idea         │ —    │ 2026-02-09 │ 2026-02-09

Total: 5 features
  Implementing: 1 (20%)
  Specified: 2 (40%)
  Planned: 1 (20%)
  Idea/Backlog: 1 (20%)
```

## Metrics View

Track key metrics across features:

```bash
agileplus metrics
```

```
Project Metrics

VELOCITY
  Avg cycle time: 8 days
  Throughput: 1 feature per 8 days
  Trends: ↑ Improving (last 3 features: 10, 9, 8 days)

QUALITY
  Test coverage: 87% avg
  Agent pass rate: 100% (8/8 WPs approved first review)
  Rework rate: 12.5% (1/8 WPs went back once)

TEAM
  Agent utilization: 2 agents (claude-code, codex)
  Active WPs: 1 (WP03 in progress)
  Blocked WPs: 1 (WP04 blocked by WP02, WP03)

SCHEDULE
  On-time delivery: 100% (0 missed deadlines)
  Variance: 0 days (all on estimate)
```

## Audit Trail

View all state transitions and changes:

```bash
agileplus audit 001 --format timeline
```

```
Feature 001: Audit Trail

2026-02-11 10:00  WP01 → done
  Agent: claude-code
  Approver: @jane
  Commits: 4
  Files: 3
  Hash: abc1234

2026-02-11 09:30  WP02 → for_review
  Agent: claude-code
  Status change initiated
  Commits: 3
  Hash: def5678

2026-02-11 14:00  WP03 → doing
  Agent: claude-code (session: sess_abc123)
  Dispatched via: agileplus implement 001 --wp WP03
  Hash: ghi9012

...
```

## Key Features

- **Real-time updates** — Refreshes automatically (5-second intervals in watch mode)
- **Drag-and-drop** — Web dashboard allows manual lane transitions
- **Agent tracking** — See which agent is working on what
- **Dependency visualization** — Understand blocking relationships
- **Metrics** — Velocity, quality, team metrics over time
- **Audit trail** — Complete history of all state changes
- **Search & filter** — Find features, WPs, and agents easily

## Dashboard Configuration

Customize dashboard appearance in `.kittify/config.yaml`:

```yaml
dashboard:
  port: 9090
  refresh_interval_seconds: 5
  show_completed_features: false
  metrics_retention_days: 90
  archive_threshold_days: 365
```

## Tips

1. **Use watch mode during implementation** — Monitor progress in real-time
2. **Check metrics weekly** — Catch trends early (velocity slowing, rework increasing)
3. **Review audit trail** — Understand what happened to completed features
4. **Customize dashboard** — Focus on metrics that matter for your team
