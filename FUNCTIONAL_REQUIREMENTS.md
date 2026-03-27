# AgilePlus: Functional Requirements

**Version:** 2.1 | **Status:** Active | **Updated:** 2026-03-27
**Traces to:** PRD.md v2.0
**Source:** Derived from codebase analysis of 24 Rust crates in `crates/`

---

## FR-DOMAIN: Domain Model

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-DOMAIN-001 | System SHALL define a `Feature` entity with fields: `id` (i64), `slug` (String), `friendly_name` (String), `state` (FeatureState), `spec_hash` ([u8;32] hex-encoded), `target_branch` (String), `plane_issue_id` (Option<String>), `plane_state_id` (Option<String>), `labels` (Vec<String>), `module_id` (Option<i64>), `project_id` (Option<i64>), `created_at_commit` (Option<String>), `last_modified_commit` (Option<String>), `created_at`, `updated_at` | E1.1 | `crates/agileplus-domain/src/domain/feature.rs` |
| FR-DOMAIN-002 | System SHALL define `FeatureState` as an ordered enum: `Created`, `Specified`, `Researched`, `Planned`, `Implementing`, `Validated`, `Shipped`, `Retrospected`; each state SHALL have a monotonically increasing ordinal | E1.2 | `crates/agileplus-domain/src/domain/state_machine.rs` |
| FR-DOMAIN-003 | System SHALL enforce that state transitions are forward-only unless explicitly allowed; any attempt to transition to a state with a lower ordinal than the current state SHALL return `DomainError::InvalidTransition` | E1.2 | `crates/agileplus-domain/src/domain/state_machine.rs` |
| FR-DOMAIN-004 | System SHALL record `StateTransition { from, to, skipped: Vec<FeatureState> }` capturing all intermediate states that were skipped during an accelerated transition | E1.2 | `crates/agileplus-domain/src/domain/state_machine.rs` |
| FR-DOMAIN-005 | System SHALL define a `WorkPackage` entity with fields: `id`, `feature_id`, `ordinal`, `title`, `description`, `state` (WpState), `file_scope` (Vec<String>), `acceptance_criteria` (String), `assigned_agent` (Option<String>), `pr_url` (Option<String>), `worktree_path` (Option<String>), `base_commit` (Option<String>), `head_commit` (Option<String>), `created_at`, `updated_at` | E1.3 | `crates/agileplus-domain/src/domain/work_package/` |
| FR-DOMAIN-006 | System SHALL define `WpState` as: `Planned`, `Doing`, `Review`, `Done`, `Blocked`; transitions SHALL be validated and logged | E1.3 | `crates/agileplus-domain/src/domain/work_package/` |
| FR-DOMAIN-007 | System SHALL define `WorkPackageDependency` with fields `from_wp_id`, `to_wp_id`, `dep_type`; the graph SHALL be validated for cycles using topological sort before persistence | E1.4 | `crates/agileplus-domain/src/domain/work_package/` |
| FR-DOMAIN-008 | System SHALL define a `Module` entity with `id`, `slug`, `name`, `description`, `owner`, `project_id`; each `Feature` SHALL be optionally owned by exactly one `Module` | E1.5 | `crates/agileplus-domain/src/domain/module.rs` |
| FR-DOMAIN-009 | System SHALL define a `Project` entity with `id`, `slug`, `name`, `description`, `owner`, `created_at`; Features and Modules SHALL be scoped to a Project | E1.5 | `crates/agileplus-domain/src/domain/project.rs` |
| FR-DOMAIN-010 | System SHALL define a `Backlog` entity representing a prioritized collection of features within a project; backlog items SHALL have `priority` (i32) and `added_at` fields | E1.5 | `crates/agileplus-domain/src/domain/backlog.rs` |

---

## FR-AUDIT: Immutable Audit Trail

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-AUDIT-001 | System SHALL define `AuditEntry` with fields: `id`, `feature_id`, `wp_id` (Option), `timestamp`, `actor`, `transition` (String), `evidence_refs` (Vec<EvidenceRef>), `prev_hash` ([u8;32]), `hash` ([u8;32]), `event_id` (Option), `archived_to` (Option<String>) | E3.1 | `crates/agileplus-domain/src/domain/audit.rs` |
| FR-AUDIT-002 | System SHALL compute each `AuditEntry.hash` as SHA-256 over: `feature_id`, `wp_id`, `unix_timestamp_nanos`, `actor`, `transition`, and `prev_hash`; the genesis entry SHALL use `[0u8;32]` as `prev_hash` | E3.1 | `crates/agileplus-domain/src/domain/audit.rs::hash_entry` |
| FR-AUDIT-003 | System SHALL expose a `verify_chain(entries: &[AuditEntry])` function that returns `AuditChainError::EmptyChain`, `AuditChainError::HashMismatch { index, expected, actual }`, or `AuditChainError::PrevHashMismatch { index }` on any integrity violation | E3.1 | `crates/agileplus-domain/src/domain/audit.rs` |
| FR-AUDIT-004 | System SHALL define `EvidenceRef { evidence_id: i64, fr_id: String }` linking audit entries to evidence records indexed by functional requirement ID | E3.2 | `crates/agileplus-domain/src/domain/audit.rs` |
| FR-AUDIT-005 | System SHALL persist audit entries to SQLite via the `agileplus-sqlite` crate with no in-place modification; all mutations to audit records are forbidden after initial write | E3.1 | `crates/agileplus-sqlite/` |
| FR-AUDIT-006 | System SHALL support archiving audit entries to MinIO object storage; the `archived_to` field SHALL contain the object key after archiving | E3.3 | `crates/agileplus-domain/src/domain/audit.rs` |

---

## FR-CLI: Command-Line Interface

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-CLI-001 | The `agileplus` CLI SHALL implement `feature create --slug <slug> --name <name> [--target-branch <branch>]` that creates a Feature in `Created` state and prints the assigned ID | E1.1 | `crates/agileplus-cli/` |
| FR-CLI-002 | The `agileplus` CLI SHALL implement `feature list [--state <state>] [--project <project>]` that lists features with their state, slug, and ID in tabular format | E1.1 | `crates/agileplus-cli/` |
| FR-CLI-003 | The `agileplus` CLI SHALL implement `feature transition <slug> <target-state>` that advances a feature through the state machine and emits an audit entry | E1.2 | `crates/agileplus-cli/` |
| FR-CLI-004 | The `agileplus` CLI SHALL implement `wp create --feature <slug> --title <title> [--acceptance <criteria>]` and `wp list --feature <slug>` | E1.3 | `crates/agileplus-cli/` |
| FR-CLI-005 | The `agileplus` CLI SHALL implement `status <feature-id> --wp <wp-id> --state <state>` to update work package state | E1.3 | `crates/agileplus-cli/` |
| FR-CLI-006 | The `agileplus` CLI SHALL implement `specify --title <title> --description <desc>` to create a new specification with AI-assisted content generation | E2.1 | `crates/agileplus-subcmds/` |
| FR-CLI-007 | The `agileplus` CLI SHALL implement `triage` to read incoming issues/tickets and produce prioritized backlog items | E4.1 | `crates/agileplus-triage/` |
| FR-CLI-008 | The `agileplus` CLI SHALL implement `sync` to bidirectionally synchronize features and work packages with the configured Plane.so workspace | E5.3 | `crates/agileplus-sync/` |
| FR-CLI-009 | The `agileplus` CLI SHALL implement `dashboard` to launch an htmx-driven terminal dashboard showing feature/WP state, agent activity, and audit events | E6.1 | `crates/agileplus-dashboard/` |

---

## FR-API: HTTP REST API

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-API-001 | The Axum HTTP server SHALL expose `POST /features`, `GET /features`, `GET /features/{id}`, `PUT /features/{id}`, `DELETE /features/{id}` | E1.1 | `crates/agileplus-api/` |
| FR-API-002 | The Axum HTTP server SHALL expose `POST /features/{id}/transition` accepting `{ "target_state": "<state>" }` and returning the resulting audit entry | E1.2 | `crates/agileplus-api/` |
| FR-API-003 | The Axum HTTP server SHALL expose `GET /features/{id}/work-packages` and `POST /features/{id}/work-packages` | E1.3 | `crates/agileplus-api/` |
| FR-API-004 | The Axum HTTP server SHALL expose `GET /features/{id}/audit` returning the full hash-chained audit trail for a feature, sorted by `timestamp` ascending | E3.1 | `crates/agileplus-api/` |
| FR-API-005 | The Axum HTTP server SHALL expose `GET /health` returning `{ "status": "ok", "version": "<semver>" }` with HTTP 200 | E6.2 | `crates/agileplus-api/` |
| FR-API-006 | The Axum HTTP server SHALL expose `GET /metrics` in Prometheus text exposition format via the `agileplus-telemetry` crate | E6.3 | `crates/agileplus-telemetry/` |

---

## FR-GRPC: gRPC Service Layer

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-GRPC-001 | System SHALL define gRPC endpoints for feature CRUD (Create, Get, Update, Delete, List) via protobuf definitions in `proto/agileplus/v1/` | E1.1 | `proto/agileplus/v1/`, `crates/agileplus-grpc/` |
| FR-GRPC-002 | System SHALL define `TransitionFeature` RPC with `TransitionRequest { feature_id, target_state }` and `TransitionResponse { audit_entry }` | E1.2 | `proto/agileplus/v1/` |
| FR-GRPC-003 | System SHALL define RPCs for agent dispatch: `SpawnAgent`, `MonitorAgent`, `RequestReview`, `SubmitReview` | E2.1 | `proto/agileplus/v1/` |
| FR-GRPC-004 | System SHALL generate Rust bindings via `tonic`/`prost` (stored in `rust/`) and Python stubs via `grpcio` (stored in `python/`) from the same proto definitions | E5.1 | `rust/`, `python/`, `buf.gen.yaml` |
| FR-GRPC-005 | The gRPC server SHALL use TLS when `config.grpc.tls_cert_path` and `config.grpc.tls_key_path` are set; plaintext SHALL be allowed only in development mode | E6.4 | `crates/agileplus-grpc/` |

---

## FR-STORAGE: Persistence Layer

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-STORAGE-001 | The SQLite adapter (`agileplus-sqlite`) SHALL implement all repository port traits defined in `crates/agileplus-domain/src/ports/`; no other crate SHALL depend directly on SQLite | E1.1 | `crates/agileplus-sqlite/` |
| FR-STORAGE-002 | The SQLite adapter SHALL use WAL journal mode and `PRAGMA synchronous=NORMAL` for local-first performance | E6.5 | `crates/agileplus-sqlite/` |
| FR-STORAGE-003 | All schema migrations SHALL be embedded in the binary via `sqlx::migrate!` and applied automatically on startup | E6.5 | `crates/agileplus-sqlite/` |
| FR-STORAGE-004 | The cache layer (`agileplus-cache`) SHALL implement in-memory LRU caching for frequently-accessed features and work packages with configurable TTL | E6.6 | `crates/agileplus-cache/` |

---

## FR-EVENTS: Event Bus

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-EVENTS-001 | System SHALL publish domain events to NATS JetStream when features or work packages change state; event subjects SHALL follow the pattern `agileplus.features.{feature_id}.{event_type}` | E3.3 | `crates/agileplus-nats/`, `crates/agileplus-events/` |
| FR-EVENTS-002 | System SHALL define `DomainEvent` with fields: `id`, `event_type`, `feature_id`, `wp_id` (Option), `payload` (JSON), `timestamp`, `actor` | E3.3 | `crates/agileplus-events/` |
| FR-EVENTS-003 | The NATS adapter SHALL reconnect with exponential backoff (max 30 s, max 10 retries) on connection loss; pending events SHALL be buffered in SQLite during disconnect | E6.7 | `crates/agileplus-nats/` |

---

## FR-PLANE: Plane.so Integration

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-PLANE-001 | The Plane adapter SHALL map `Feature.state` to Plane issue state IDs via `SyncMapping { agileplus_state, plane_state_id }` stored in `sync_mappings` table | E5.3 | `crates/agileplus-plane/`, `crates/agileplus-domain/src/domain/sync_mapping.rs` |
| FR-PLANE-002 | The sync process SHALL update `Feature.plane_issue_id` and `Feature.plane_state_id` on successful Plane sync; sync failures SHALL be logged and retried | E5.3 | `crates/agileplus-sync/` |
| FR-PLANE-003 | The Plane adapter SHALL create Plane issues for newly-created features that lack a `plane_issue_id`; duplicate creation SHALL be idempotent | E5.3 | `crates/agileplus-plane/` |

---

## FR-GIT: Git VCS Integration

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-GIT-001 | The Git adapter SHALL resolve the current commit SHA for any working directory and record it as `Feature.created_at_commit` or `Feature.last_modified_commit` | E2.3 | `crates/agileplus-git/` |
| FR-GIT-002 | The Git adapter SHALL create and delete worktrees at the path specified in `WorkPackage.worktree_path` using `git worktree add` and `git worktree remove` | E2.2 | `crates/agileplus-git/` |
| FR-GIT-003 | The GitHub adapter (`agileplus-github`) SHALL create pull requests with title, body, base branch, and head branch; PR URL SHALL be stored in `WorkPackage.pr_url` | E2.3 | `crates/agileplus-github/` |

---

## FR-GOVERN: Governance Engine

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-GOVERN-001 | The governance engine SHALL define `GovernanceContract` specifying required evidence types (test output, CI log, security scan, review approval) per state transition | E3.2 | `crates/agileplus-domain/src/domain/governance.rs` |
| FR-GOVERN-002 | Any state transition SHALL be blocked if the required evidence for that transition is not attached to the feature or work package; the API SHALL return HTTP 422 with a list of missing evidence IDs | E3.2 | `crates/agileplus-api/`, `crates/agileplus-domain/src/domain/governance.rs` |
| FR-GOVERN-003 | The triage engine (`agileplus-triage`) SHALL classify incoming tickets by severity, estimate effort, and assign to a backlog with a priority score | E4.1 | `crates/agileplus-triage/` |
| FR-GOVERN-004 | The telemetry crate SHALL expose Prometheus counters for: `agileplus_features_total`, `agileplus_transitions_total`, `agileplus_audit_entries_total`, `agileplus_governance_violations_total` | E6.3 | `crates/agileplus-telemetry/` |
