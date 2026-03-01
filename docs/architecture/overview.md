---
audience: [developers, sdk]
---

# Architecture Overview

AgilePlus uses a clean architecture with port-based adapters.

## Crate Dependency Graph

```mermaid
graph TD
    CLI[agileplus-cli] --> Engine[agileplus-engine]
    CLI --> Domain[agileplus-domain]
    Engine --> Domain
    Engine --> Sync[agileplus-sync]
    Engine --> Agents[agileplus-agents]
    CLI --> SQLite[agileplus-sqlite]
    CLI --> Git[agileplus-git]
    SQLite --> Domain
    Git --> Domain
    Sync --> Domain
    Agents --> Domain

    style Domain fill:#7ebab5,color:#131517
    style CLI fill:#353a40,color:#f6f5f5
```

Text representation:

```
agileplus-cli
  ├── agileplus-domain    (domain entities, FSM, governance)
  ├── agileplus-sqlite    (StoragePort adapter)
  ├── agileplus-git       (VcsPort adapter)
  ├── agileplus-triage    (classifier, backlog, router)
  ├── agileplus-plane     (Plane.so sync)
  ├── agileplus-github    (GitHub issue sync)
  └── agileplus-subcmds   (sub-command registry, audit)
```

## Port Traits

The domain defines port traits; adapters implement them:

| Port | Trait | Adapters |
|------|-------|----------|
| Storage | `StoragePort` | `SqliteStorageAdapter` |
| VCS | `VcsPort` | `GitVcsAdapter` |
| Agent | `AgentPort` | `StubAgentAdapter` (future: Claude, Codex) |

## Data Flow

```
CLI Input → Command Handler → Domain Service → Port Trait → Adapter → External System
                                    ↓
                              Audit Chain (SHA-256 linked)
```

## Key Design Decisions

- **Rust 2024 edition** workspace with 8 crates
- **SQLite** for local-first storage (no server required)
- **Rule-based triage** (keyword matching with weighted scoring)
- **Token bucket rate limiting** for external API clients
- **SHA-256 content hashing** for sync conflict detection
- **Append-only JSONL** audit trail for sub-command dispatch
