# Crate Map

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| `agileplus-domain` | Domain entities, FSM, governance | `Feature`, `WorkPackage`, `AuditEntry`, `StoragePort`, `VcsPort` |
| `agileplus-cli` | CLI entry point (clap) | `Cli`, `Commands`, command handlers |
| `agileplus-sqlite` | SQLite storage adapter | `SqliteStorageAdapter` |
| `agileplus-git` | Git VCS adapter | `GitVcsAdapter` |
| `agileplus-triage` | Triage classifier + backlog | `TriageClassifier`, `BacklogItem`, `RouterGenerator` |
| `agileplus-plane` | Plane.so sync | `PlaneClient`, `PlaneSyncAdapter` |
| `agileplus-github` | GitHub issue sync | `GitHubClient`, `GitHubSyncAdapter` |
| `agileplus-subcmds` | Sub-command registry + audit | `SubCommandRegistry`, `AuditLog` |

## Dependency Rules

- `agileplus-domain` has **zero** external dependencies (pure domain logic)
- Adapter crates depend on `agileplus-domain` for port traits
- `agileplus-cli` depends on all adapter crates
- No circular dependencies between crates
