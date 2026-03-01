---
audience: [developers, sdk]
---

# Domain Model

## Feature

The central entity. Features move through a governed state machine:

```mermaid
erDiagram
    Feature ||--o{ WorkPackage : contains
    Feature ||--o{ AuditEntry : tracks
    Feature {
        string id
        string title
        FeatureState state
        datetime created_at
    }
    WorkPackage ||--o{ Task : contains
    WorkPackage {
        string id
        string title
        Lane lane
        string[] dependencies
    }
    Task {
        string id
        string description
        bool done
    }
    AuditEntry {
        string action
        string actor
        datetime timestamp
        string details
    }
```

```
Draft → Specified → Researched → Planned → Implementing → Validating → Shipped
```

Each feature has:
- Unique ID and slug
- Specification artifact (markdown)
- Work packages (1:N)
- Audit trail entries

## Work Package

A unit of work within a feature:
- Has its own git branch
- Contains subtasks
- Tracks dependencies on other WPs
- Moves through: `planned → doing → for_review → done`

## Audit Entry

Immutable record of every state transition:
- SHA-256 hash chain (each entry references previous)
- Actor identification (human or agent)
- Timestamp, from/to state, metadata

## Backlog Item

Triage output routed to the backlog:
- Intent classification (bug, feature, idea, task)
- Priority scoring (critical → low)
- Status tracking (new → triaged → in_progress → done)
