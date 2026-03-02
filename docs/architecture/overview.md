---
audience: [developers, sdk]
---

# Architecture Overview

AgilePlus is built on **hexagonal architecture** (ports and adapters) with clear separation of concerns. The domain is at the center, surrounded by adapters that handle external systems (git, SQLite, agent APIs, etc.).

## Crate Structure

The workspace contains 11 crates organized by responsibility:

```mermaid
graph TD
    CLI["agileplus-cli<br/>(Command-line interface)"]
    API["agileplus-api<br/>(REST API, gRPC)"]

    Engine["agileplus-engine<br/>(Orchestration)"]
    Domain["agileplus-domain<br/>(Core entities, FSM)"]

    SQLite["agileplus-sqlite<br/>(Storage adapter)"]
    Git["agileplus-git<br/>(VCS adapter)"]
    Agent["agileplus-agent*<br/>(Agent dispatch)"]

    SubCmds["agileplus-subcmds<br/>(Audit logging)"]
    Triage["agileplus-triage<br/>(Classification)"]
    Telemetry["agileplus-telemetry<br/>(Observability)"]

    Sync["agileplus-plane<br/>(External sync)"]

    CLI -->|depends| Domain
    CLI -->|depends| Engine
    API -->|depends| Domain
    API -->|depends| Engine

    Engine -->|depends| Domain
    Engine -->|depends| SQLite
    Engine -->|depends| Git
    Engine -->|depends| Agent

    SQLite -->|implements StoragePort| Domain
    Git -->|implements VcsPort| Domain
    Agent -->|implements AgentPort| Domain

    SubCmds -->|depends| Domain
    Triage -->|depends| Domain
    Telemetry -->|depends| Domain
    Sync -->|depends| Domain

    style Domain fill:#4a9b7f,color:#fff
    style CLI fill:#2c3e50,color:#fff
    style API fill:#2c3e50,color:#fff
    style Engine fill:#34495e,color:#fff
```

## Crate Descriptions

| Crate | Purpose | Type | Stability |
|-------|---------|------|-----------|
| **agileplus-domain** | Core entities (Feature, WP, Audit), FSM, port traits | Library | Stable (WP01-04) |
| **agileplus-cli** | Command-line interface; dispatches commands to engine | Binary | MVP |
| **agileplus-api** | REST API and gRPC endpoints for remote access | Library | WP15+ |
| **agileplus-engine** | Orchestration logic; coordinates ports; implements FSM transitions | Library | In Progress |
| **agileplus-sqlite** | Storage adapter; SQLite schema, migrations, queries | Library | WP05-06 |
| **agileplus-git** | VCS adapter; git worktrees, branches, merges, artifacts | Library | WP07 |
| **agileplus-subcmds** | Agent sub-command registry and audit logging | Library | WP04 |
| **agileplus-triage** | Rule-based classification and backlog management | Library | Prototype |
| **agileplus-telemetry** | Observability: logging, tracing, metrics | Library | Planning |
| **agileplus-plane** | Sync with Plane.so for external issue tracking | Library | Future |
| **agileplus-grpc** | Protocol buffer definitions and gRPC service stubs | Library | Future |

## Dependency Rules

Strict hierarchy ensures clean dependencies:

```
Adapters (SQLite, Git, Agent) ← Domain (entities, ports, FSM)
                               ↑
Engine ← Domain + Adapters
        ↑
CLI / API ← Engine + Domain
```

**Rules**:
- Adapters **only** depend on Domain (they implement port traits)
- Engine **only** depends on Domain and adapters (it orchestrates)
- CLI/API **only** depend on Engine and Domain
- **No circular dependencies**
- **No adapter-to-adapter dependencies** (they only talk via Domain)

## Port Traits

The domain defines three primary port traits in `crates/agileplus-domain/src/ports/`:

### StoragePort (`storage.rs`)

Abstracts persistence:
- Feature CRUD (create, read, update, list by state)
- Work package CRUD
- Audit trail append and retrieval
- Evidence storage and queries
- Policy rule management
- Governance contract storage

**Implemented by**: `SqliteStorageAdapter` (crates/agileplus-sqlite)

**Key methods**:
```rust
// Features
fn create_feature(&self, feature: &Feature) -> Result<i64>
fn get_feature_by_slug(&self, slug: &str) -> Result<Option<Feature>>
fn update_feature_state(&self, id: i64, state: FeatureState) -> Result<()>
fn list_features_by_state(&self, state: FeatureState) -> Result<Vec<Feature>>

// Work packages
fn create_work_package(&self, wp: &WorkPackage) -> Result<i64>
fn list_wps_by_feature(&self, feature_id: i64) -> Result<Vec<WorkPackage>>
fn get_ready_wps(&self, feature_id: i64) -> Result<Vec<WorkPackage>>

// Audit trail
fn append_audit_entry(&self, entry: &AuditEntry) -> Result<i64>
fn get_audit_trail(&self, feature_id: i64) -> Result<Vec<AuditEntry>>
```

### VcsPort (`vcs.rs`)

Abstracts version control:
- Worktree creation and cleanup
- Branch operations (create, checkout, merge)
- Artifact read/write (spec.md, plan.md, etc.)
- Conflict detection
- History scanning

**Implemented by**: `GitVcsAdapter` (crates/agileplus-git)

**Key methods**:
```rust
fn create_worktree(&self, feature_slug: &str, wp_id: &str) -> Result<PathBuf>
fn create_branch(&self, branch_name: &str, base: &str) -> Result<()>
fn merge_to_target(&self, source: &str, target: &str) -> Result<MergeResult>
fn read_artifact(&self, feature_slug: &str, path: &str) -> Result<String>
fn write_artifact(&self, feature_slug: &str, path: &str, content: &str) -> Result<()>
```

### AgentPort (`agent.rs`)

Abstracts agent dispatch:
- Synchronous and asynchronous dispatch
- Status polling
- Job cancellation
- Sub-command execution
- Result collection

**Implemented by**: Adapter layer (Claude Code CLI in MVP; Codex batch API in WP08)

**Key methods**:
```rust
fn dispatch(&self, task: AgentTask, config: &AgentConfig) -> Result<AgentResult>
fn dispatch_async(&self, task: AgentTask, config: &AgentConfig) -> Result<String>
fn query_status(&self, job_id: &str) -> Result<AgentStatus>
fn send_instruction(&self, job_id: &str, instruction: &str) -> Result<()>
```

### Observability Port (`observability.rs`)

Abstracts logging and tracing:
- Structured logging
- Span-based tracing
- Metrics recording
- Error reporting

**Implemented by**: `tracing` crate adapters

### Review Port (`review.rs`)

Abstracts code review and governance:
- PR submission
- Review polling
- Evidence validation
- Approval tracking

**Implemented by**: GitHub API adapter (planned WP12)

## Data Flow

A typical command flow through the system:

```
User Command (CLI)
    ↓
[Command Parser] (agileplus-cli)
    ↓
Engine Dispatch (agileplus-engine)
    ├→ Load Feature from StoragePort
    │   (SQLite implementation)
    ├→ Validate state machine preconditions
    ├→ Perform operation (e.g., decompose into WPs)
    ├→ Persist results via StoragePort
    ├→ Create git branch via VcsPort
    │   (Git implementation)
    ├→ Dispatch agent via AgentPort
    │   (Claude Code implementation)
    ├→ Record audit entry
    │   (with hash chain)
    └→ Return result to CLI

CLI Output
    ↓
User Terminal
```

All operations are **transactional** at the domain level:
1. Validate preconditions
2. Load domain state
3. Apply state transition (FSM)
4. Persist to storage
5. Record audit entry
6. Return success or error

If any step fails, the entire operation fails and state is rolled back (at the storage layer).

## Technology Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Language | Rust 2024 edition | Type safety, zero-cost abstractions, fearless concurrency |
| Storage | SQLite + sqlx | Local-first, no server, type-safe queries, migrations |
| VCS | git2-rs | Native git operations, stable API |
| Async | tokio | Industry standard, full-featured |
| Serialization | serde + JSON | Rust ecosystem standard |
| Hashing | SHA-256 (sha2 crate) | Cryptographic integrity |
| Error Handling | thiserror | Ergonomic error types |
| CLI | clap (planned) | Argument parsing and help |
| Observability | tracing | Structured logging and spans |

## Design Principles

### 1. Domain at the Center

Domain logic (FSM, state transitions, validation) lives in `agileplus-domain` and knows **nothing** about:
- Git implementation details
- SQLite schema
- HTTP/gRPC protocols
- Agent backend specifics

### 2. Port-Driven Testing

Tests can mock ports:
```rust
#[test]
fn feature_transition_requires_evidence() {
    let storage = MockStoragePort::new();
    let vcs = MockVcsPort::new();
    let engine = Engine::new(storage, vcs);

    // Test FSM logic without touching real git/SQLite
}
```

### 3. Append-Only Audit Trail

Every domain change produces an audit entry, creating an **immutable history**. This:
- Enables compliance audits
- Provides debugging context
- Prevents tampering (via hash chain)
- Supports forensics (who did what, when)

### 4. Async-First

All I/O operations (storage, VCS, agent dispatch) are async:
```rust
pub trait StoragePort: Send + Sync {
    fn create_feature(
        &self,
        feature: &Feature,
    ) -> impl Future<Output = Result<i64>> + Send;
}
```

This allows:
- Parallel WP implementation
- Concurrent governance checks
- Scalable agent dispatch

### 5. Error Handling

All domain operations return `Result<T, DomainError>`:

```rust
pub enum DomainError {
    InvalidTransition { from: String, to: String, reason: String },
    NotFound { entity: String, id: String },
    Conflict { details: String },
    Governance { requirement: String, evidence: String },
    // ... more variants
}
```

Errors are **semantic** (not just "database error") so callers know what went wrong.

## File Layout

```
crates/
├── agileplus-domain/
│   src/
│   ├── lib.rs                      # Public API, module exports
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── state_machine.rs        # FeatureState, transitions
│   │   ├── feature.rs              # Feature entity
│   │   ├── work_package.rs         # WorkPackage, DependencyGraph
│   │   ├── governance.rs           # GovernanceRule, PolicyRule
│   │   ├── audit.rs                # AuditEntry, hash chain
│   │   └── metric.rs               # Metric for observability
│   ├── ports/
│   │   ├── mod.rs
│   │   ├── storage.rs              # StoragePort trait
│   │   ├── vcs.rs                  # VcsPort trait
│   │   ├── agent.rs                # AgentPort trait
│   │   ├── observability.rs        # ObservabilityPort trait
│   │   └── review.rs               # ReviewPort trait
│   ├── error.rs                    # DomainError enum
│   └── config.rs                   # Configuration types
├── agileplus-sqlite/
│   src/
│   ├── lib.rs
│   ├── repository/
│   │   ├── mod.rs
│   │   ├── features.rs
│   │   ├── work_packages.rs
│   │   └── audit.rs
│   ├── migrations/
│   │   ├── 001_initial_schema.sql
│   │   ├── 002_audit_chain.sql
│   │   └── ... more migrations
│   └── adapter.rs                  # StoragePort implementation
├── agileplus-git/
│   src/
│   ├── lib.rs
│   ├── repository/
│   │   └── mod.rs
│   ├── worktree/
│   │   └── mod.rs
│   ├── artifact/
│   │   └── mod.rs
│   └── adapter.rs                  # VcsPort implementation
└── ... other crates
```

## Workspace Configuration

The workspace is defined in the root `Cargo.toml`:

```toml
[workspace]
resolver = "3"
members = [
    "crates/agileplus-domain",
    "crates/agileplus-cli",
    "crates/agileplus-api",
    "crates/agileplus-grpc",
    "crates/agileplus-sqlite",
    "crates/agileplus-git",
    "crates/agileplus-telemetry",
    "crates/agileplus-triage",
    "crates/agileplus-subcmds",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[workspace.dependencies]
# Shared versions across all crates
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

## Related Pages

- [Domain Model](domain-model.md) — Entity relationships and ER diagrams
- [Port Traits](ports.md) — Detailed port interface documentation
- [Dependency Resolution](../guides/dependencies.md) — WP dependency management
- [Crate Development](../guides/crate-development.md) — Adding new crates
