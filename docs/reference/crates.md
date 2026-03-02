---
audience: [developers, sdk]
---

# Crate Architecture Map

AgilePlus is organized into 13+ crates following hexagonal architecture. The domain layer has zero external dependencies; all I/O is abstracted via ports and adapters.

## Core Domain Layer

### agileplus-domain

The immutable core of the system: domain entities, state machines, port traits, and governance logic.

| Module | Responsibility |
|--------|-----------------|
| `domain/` | Feature, WorkPackage, AuditEntry, Evidence, Metric, PolicyRule |
| `state_machine/` | FSM transitions with governance gate validation |
| `ports/` | StoragePort, VcsPort, AgentPort trait definitions |
| `governance/` | GovernanceContract, GateViolation, rule enforcement |
| `error/` | DomainError enum with strong typing |

**Key Types:**
- `Feature { id, slug, state: FeatureState, ...}`
- `WorkPackage { id, feature_id, sequence, state: WpState, ...}`
- `AuditEntry { id, feature_slug, timestamp, actor, transition, hash, ... }` (immutable)
- `StoragePort`, `VcsPort`, `AgentPort` (async trait interfaces)

**Dependencies:** None (zero external crates)

**Stability:** Very stable. Changes here cascade to all adapters.

## CLI & Entry Point

### agileplus-cli

Command-line interface using `clap` for argument parsing and subcommand routing.

| Module | Commands |
|--------|----------|
| `commands/specify` | `agileplus specify` |
| `commands/research` | `agileplus research` |
| `commands/plan` | `agileplus plan` |
| `commands/implement` | `agileplus implement` + agent dispatch |
| `commands/validate` | `agileplus validate` |
| `commands/ship` | `agileplus ship` |
| `commands/retrospective` | `agileplus retrospective` |
| `commands/triage` | `agileplus triage` |
| `commands/queue` | `agileplus queue {add,list,show,pop}` |

**Key Types:**
- `Cli { command: Commands, verbose: u8, db: PathBuf, repo: Option<PathBuf> }`
- `Commands` enum (Specify, Research, Plan, Implement, Validate, Ship, Retrospective, Triage, Queue)
- Individual command args structs (SpecifyArgs, PlanArgs, ImplementArgs, etc.)

**Dependencies:** agileplus-domain, all adapter crates

**Stability:** Moderate. Command signatures can change with feature work.

## Storage Adapters

### agileplus-sqlite

SQLite adapter implementing `StoragePort`.

| Type | Purpose |
|------|---------|
| `SqliteStorageAdapter` | Connection pooling, async queries |
| `FeatureRepository` | Feature CRUD operations |
| `WorkPackageRepository` | WorkPackage CRUD + dependency tracking |
| `AuditRepository` | Append-only audit log with hash chaining |
| `EvidenceRepository` | Evidence/governance artifact storage |
| `MetricRepository` | Performance metrics recording |

**Key Features:**
- Tokio async runtime via `tokio-rusqlite`
- Connection pooling via `sqlx`
- Transaction support for atomicity
- Hash chaining for audit immutability

**Dependencies:** agileplus-domain, tokio, sqlx, rusqlite

**Stability:** Very stable. Schema changes are migrations only.

### agileplus-api

RESTful HTTP adapter for remote access.

**Key Routes:**
- `GET /features` — list features
- `POST /features` — create feature
- `GET /features/:slug` — get feature by slug
- `POST /features/:slug/transition` — trigger state transition
- `GET /features/:slug/audit` — fetch audit trail
- `GET /features/:slug/work-packages` — list work packages

**Dependencies:** agileplus-domain, agileplus-sqlite, axum (web framework), tokio

**Stability:** Moderate. API may expand with new endpoints.

### agileplus-grpc

gRPC service adapter (WP09) implementing `AgilePlusCoreService`.

**Key Services:**
- Feature operations: GetFeature, ListFeatures, GetFeatureState
- Work package operations: ListWorkPackages, GetWorkPackageStatus
- Governance: CheckGovernanceGate, GetAuditTrail, VerifyAuditChain
- Command dispatch: DispatchCommand, StreamAgentEvents

**Dependencies:** agileplus-domain, agileplus-sqlite, tonic (gRPC framework), prost (protobuf)

**Stability:** Moderate. Service definitions may expand.

## VCS Adapters

### agileplus-git

Git adapter implementing `VcsPort`.

| Type | Purpose |
|------|---------|
| `GitVcsAdapter` | Wraps `git2-rs` for CLI-independent operations |
| `WorktreeManager` | Creates/destroys isolated worktrees |
| `BranchManager` | Branch lifecycle (create, checkout, merge) |
| `ArtifactStore` | Read/write feature artifacts |
| `ConflictDetector` | Pre-merge conflict analysis |
| `HistoryScanner` | Audit trail reconstruction from git log |

**Key Features:**
- Async via `git2` (blocking) wrapped in `tokio::task::spawn_blocking`
- Worktree isolation for agent workspaces
- Merge conflict detection without applying
- Git history reconstruction for governance audits

**Dependencies:** agileplus-domain, git2, tokio

**Stability:** Very stable. Git behavior is standardized.

## Service Adapters

### agileplus-triage

Triage classifier and backlog queue.

| Type | Purpose |
|------|---------|
| `TriageClassifier` | NLP-based classification (bug, feature, idea, task) |
| `RouterGenerator` | Route classified items to appropriate queues |
| `BacklogQueue` | In-memory or persisted task queue |

**Key Features:**
- Keyword-based classification (can be replaced with ML model)
- Priority scoring
- Queue persistence to JSON/SQLite
- Batch processing support

**Dependencies:** agileplus-domain, regex, serde

**Stability:** Moderate. Classification logic may improve.

### agileplus-plane

[Plane.so](https://plane.so) integration for project management sync.

| Type | Purpose |
|------|---------|
| `PlaneClient` | HTTP API wrapper |
| `PlaneSyncAdapter` | Bidirectional sync: Feature ↔ Plane Issue |
| `PlaneEventListener` | Webhook handler for external updates |

**Key Features:**
- OAuth2 authentication
- Issue state mapping
- Automatic PR linking
- Rate limiting

**Dependencies:** agileplus-domain, reqwest, tokio

**Stability:** Moderate. API may change with Plane updates.

### agileplus-github

GitHub integration for PR/issue tracking.

| Type | Purpose |
|------|---------|
| `GitHubClient` | GraphQL + REST API wrapper |
| `GitHubSyncAdapter` | Feature ↔ GitHub Issue sync |
| `PrBuilder` | Programmatic PR creation from work packages |
| `ReviewAdapter` | PR review status polling (CodeRabbit fallback) |

**Key Features:**
- GraphQL for efficient querying
- OAuth App authentication
- PR template rendering
- Automated review via CodeRabbit bot
- Merge conflict detection

**Dependencies:** agileplus-domain, octocrab, reqwest, tokio

**Stability:** Moderate. GitHub API evolves regularly.

## Utility Crates

### agileplus-subcmds

Hidden subcommand registry and audit system.

| Category | Commands | Purpose |
|----------|----------|---------|
| `branch` | create, checkout, delete, list | Branch management |
| `commit` | create, amend, fixup, rebase | Commit operations |
| `diff` | show, stat | Diff inspection |
| `stash` | push, pop, list | Stash management |
| `worktree` | add, remove, list | Worktree operations |
| `artifact` | write, read, hash | Artifact file ops |
| `governance` | check, enforce | Governance validation |
| `audit` | log, query, verify | Audit trail queries |

**Key Features:**
- Append-only JSONL audit log
- Command signature tracking (args, exit code, duration)
- Agent-accessible via MCP
- Full traceability for compliance

**Dependencies:** agileplus-domain, serde, chrono

**Stability:** Very stable. Commands are immutable once added.

### agileplus-telemetry

Observability: logging, metrics, tracing.

| Type | Purpose |
|------|---------|
| `TraceLayer` | OpenTelemetry integration (optional) |
| `MetricsCollector` | Prometheus-compatible metrics |
| `LogFormatter` | Structured logging with context |

**Dependencies:** tracing, opentelemetry, prometheus

**Stability:** Very stable. Observability-only.

## Agents (Separate Workspace)

The `agileplus-agents/` directory contains AI agent orchestration.

### agileplus-agent-dispatch

Agent dispatch and lifecycle management (WP08).

| Type | Purpose |
|------|---------|
| `AgentDispatchAdapter` | Implements `AgentPort` from domain |
| `AgentTask` | Work package + context + prompt |
| `AgentConfig` | Kind, timeout, review cycles |
| `AgentResult` | Exit code, commits, PR URL |
| `JobState` | Pending, Running, Completed, Failed |

**Key Features:**
- Job ID tracking (UUID v4)
- Async dispatch with background tracking
- Status polling for long-running agents
- Session timeout with graceful termination
- Review loop integration

**Dependencies:** agileplus-domain, tokio, dashmap, uuid

**Stability:** Moderate. Agent orchestration is still evolving.

### agileplus-agent-service

gRPC service for agent-facing APIs (WP09).

**Key Services:**
- Agent registration
- Prompt delivery
- Output collection
- Session lifecycle

**Dependencies:** agileplus-agent-dispatch, tonic, prost

**Stability:** Moderate.

### agileplus-agent-review

Code review automation and fallback (WP09).

| Type | Purpose |
|------|---------|
| `CodeRabbitReviewer` | CodeRabbit bot integration |
| `CiStatusPolling` | GitHub Actions CI status |
| `FallbackReviewer` | Simple heuristic-based review |

**Key Features:**
- Wait for CodeRabbit reviews
- Poll GitHub Actions status
- Fallback to basic checks if CI/review unavailable

**Dependencies:** agileplus-domain, octocrab, reqwest, tokio

**Stability:** Moderate. Review logic may expand.

## Dependency Graph

```
agileplus-domain (0 external deps)
  ↑
  ├─ agileplus-cli
  ├─ agileplus-sqlite
  ├─ agileplus-git
  ├─ agileplus-api
  ├─ agileplus-grpc
  ├─ agileplus-triage
  ├─ agileplus-plane
  ├─ agileplus-github
  ├─ agileplus-subcmds
  ├─ agileplus-telemetry
  └─ agileplus-agents/*
        ├─ agileplus-agent-dispatch
        ├─ agileplus-agent-service
        └─ agileplus-agent-review
```

**Key Principle:** All arrows point UP to domain. No downward dependencies.

## Compilation & Workspace

Build the full workspace:

```bash
cargo build --release
cargo test
cargo test --all --doc
```

Build individual crates:

```bash
cargo build -p agileplus-cli
cargo build -p agileplus-sqlite --release
```

Run CLI:

```bash
cargo run -p agileplus-cli -- specify 001-login
```

## Stability Matrix

| Crate | Stability | Frequency | Impact |
|-------|-----------|-----------|--------|
| agileplus-domain | Very High | Rare | High (affects all) |
| agileplus-cli | Moderate | Regular | High |
| agileplus-sqlite | Very High | Rare | High (data) |
| agileplus-git | Very High | Rare | Medium |
| agileplus-api | Moderate | Regular | Medium (external) |
| agileplus-grpc | Moderate | Regular | Medium (external) |
| agileplus-github | Moderate | Regular | Medium (integration) |
| agileplus-agents/* | Low | Frequent | Medium (WIP) |
