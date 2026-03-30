# AgilePlus: Functional Requirements

**Version:** 2.2 | **Status:** Active | **Updated:** 2026-03-27
**Traces to:** PRD.md v2.1
**Source:** Derived from codebase analysis of Rust crates in `crates/`

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
| FR-DOMAIN-011 | System SHALL define a `Cycle` entity with `id`, `slug`, `name`, `state` (CycleState: Draft, Active, Completed, Archived), `start_date`, `end_date`, `module_id` (Option); cycle transitions SHALL follow the defined state machine | E1.6 | `crates/agileplus-domain/src/domain/cycle/` |
| FR-DOMAIN-012 | System SHALL define a `Snapshot` entity that records materialized state of a domain entity at a known event sequence number; snapshots SHALL include `entity_type`, `entity_id`, `sequence`, `state_json`, `created_at` | E3.3 | `crates/agileplus-domain/src/domain/snapshot.rs` |
| FR-DOMAIN-013 | System SHALL define a `Metric` entity capturing per-command execution telemetry: `command`, `feature_id` (Option), `duration_ms`, `agent_runs`, `review_cycles`, `recorded_at` | E12.1 | `crates/agileplus-domain/src/domain/metric.rs` |
| FR-DOMAIN-014 | System SHALL define `ServiceHealth` with fields `adapter_name`, `status` (Healthy, Degraded, Unhealthy), `message` (Option<String>), `checked_at`; all adapters SHALL implement health reporting | E12.3 | `crates/agileplus-domain/src/domain/service_health.rs` |
| FR-DOMAIN-015 | System SHALL define an `ApiKey` entity with `id`, `key_hash` (SHA-256), `actor` (String), `created_at`, `expires_at` (Option); API keys SHALL never be stored in plaintext | E5.5 | `crates/agileplus-domain/src/domain/api_key.rs` |
| FR-DOMAIN-016 | System SHALL define a `DeviceNode` entity for P2P replication with `id`, `device_id` (UUID), `hostname`, `address`, `last_seen`; device nodes SHALL be registered at startup and updated on reconnect | E11.1 | `crates/agileplus-domain/src/domain/device_node.rs` |

---

## FR-AUDIT: Immutable Audit Trail

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-AUDIT-001 | System SHALL define `AuditEntry` with fields: `id`, `feature_id`, `wp_id` (Option), `timestamp`, `actor`, `transition` (String), `evidence_refs` (Vec<EvidenceRef>), `prev_hash` ([u8;32]), `hash` ([u8;32]), `event_id` (Option), `archived_to` (Option<String>) | E3.1 | `crates/agileplus-domain/src/domain/audit.rs` |
| FR-AUDIT-002 | System SHALL compute each `AuditEntry.hash` as SHA-256 over: `feature_id`, `wp_id`, `unix_timestamp_nanos`, `actor`, `transition`, and `prev_hash`; the genesis entry SHALL use `[0u8;32]` as `prev_hash` | E3.1 | `crates/agileplus-domain/src/domain/audit.rs::hash_entry` |
| FR-AUDIT-003 | System SHALL expose a `verify_chain(entries: &[AuditEntry])` function that returns `AuditChainError::EmptyChain`, `AuditChainError::HashMismatch { index, expected, actual }`, or `AuditChainError::PrevHashMismatch { index }` on any integrity violation | E3.4 | `crates/agileplus-domain/src/domain/audit.rs` |
| FR-AUDIT-004 | System SHALL define `EvidenceRef { evidence_id: i64, fr_id: String }` linking audit entries to evidence records indexed by functional requirement ID | E2.2 | `crates/agileplus-domain/src/domain/audit.rs` |
| FR-AUDIT-005 | System SHALL persist audit entries to SQLite via the `agileplus-sqlite` crate with no in-place modification; all mutations to audit records are forbidden after initial write | E3.1 | `crates/agileplus-sqlite/` |
| FR-AUDIT-006 | System SHALL support archiving audit entries to MinIO object storage; the `archived_to` field SHALL contain the object key after archiving | E3.5 | `crates/agileplus-domain/src/domain/audit.rs` |

---

## FR-CLI: Command-Line Interface

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-CLI-001 | The `agileplus` CLI SHALL implement `feature create --slug <slug> --name <name> [--target-branch <branch>]` that creates a Feature in `Created` state and prints the assigned ID | E1.1 | `crates/agileplus-cli/` |
| FR-CLI-002 | The `agileplus` CLI SHALL implement `feature list [--state <state>] [--project <project>]` that lists features with their state, slug, and ID in tabular format | E1.1 | `crates/agileplus-cli/` |
| FR-CLI-003 | The `agileplus` CLI SHALL implement `feature transition <slug> <target-state>` that advances a feature through the state machine and emits an audit entry | E1.2 | `crates/agileplus-cli/` |
| FR-CLI-004 | The `agileplus` CLI SHALL implement `wp create --feature <slug> --title <title> [--acceptance <criteria>]` and `wp list --feature <slug>` | E1.3 | `crates/agileplus-cli/` |
| FR-CLI-005 | The `agileplus` CLI SHALL implement `status <feature-id> --wp <wp-id> --state <state>` to update work package state | E1.3 | `crates/agileplus-cli/` |
| FR-CLI-006 | The `agileplus` CLI SHALL implement `specify --title <title> --description <desc>` to create a new specification with AI-assisted content generation | E4.1 | `crates/agileplus-subcmds/` |
| FR-CLI-007 | The `agileplus` CLI SHALL implement `triage` to read incoming issues/tickets and produce prioritized backlog items via the classifier and router pipeline | E4.9 | `crates/agileplus-triage/` |
| FR-CLI-008 | The `agileplus` CLI SHALL implement `sync` to bidirectionally synchronize features and work packages with the configured Plane.so workspace | E7.1 | `crates/agileplus-sync/` |
| FR-CLI-009 | The `agileplus` CLI SHALL implement `dashboard` to launch an htmx-driven web dashboard showing feature/WP state, agent activity, and audit events | E4.18 | `crates/agileplus-dashboard/` |
| FR-CLI-010 | The `agileplus` CLI SHALL implement `import --manifest <path>` to import features and work packages from a manifest file; the import SHALL be idempotent and produce an import report | E4.15 | `crates/agileplus-import/` |
| FR-CLI-011 | The `agileplus` CLI SHALL implement `validate <feature-slug>` that evaluates governance contracts, checks evidence coverage, and outputs a gap report; exit code SHALL be non-zero if governance requirements are unmet | E4.6 | `crates/agileplus-subcmds/` |

---

## FR-API: HTTP REST API

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-API-001 | The Axum HTTP server SHALL expose `POST /features`, `GET /features`, `GET /features/{id}`, `PUT /features/{id}`, `DELETE /features/{id}` | E1.1 | `crates/agileplus-api/` |
| FR-API-002 | The Axum HTTP server SHALL expose `POST /features/{id}/transition` accepting `{ "target_state": "<state>" }` and returning the resulting audit entry | E1.2 | `crates/agileplus-api/` |
| FR-API-003 | The Axum HTTP server SHALL expose `GET /features/{id}/work-packages` and `POST /features/{id}/work-packages` | E1.3 | `crates/agileplus-api/` |
| FR-API-004 | The Axum HTTP server SHALL expose `GET /features/{id}/audit` returning the full hash-chained audit trail for a feature, sorted by `timestamp` ascending | E3.1 | `crates/agileplus-api/` |
| FR-API-005 | The Axum HTTP server SHALL expose `GET /health` returning `{ "status": "ok", "version": "<semver>" }` with HTTP 200 | E12.3 | `crates/agileplus-api/` |
| FR-API-006 | The Axum HTTP server SHALL expose `GET /metrics` in Prometheus text exposition format via the `agileplus-telemetry` crate | E12.1 | `crates/agileplus-telemetry/` |
| FR-API-007 | All REST API routes SHALL reject unauthenticated requests with HTTP 401; API key SHALL be passed in the `Authorization: Bearer <key>` header | E5.5 | `crates/agileplus-api/` |
| FR-API-008 | The Axum HTTP server SHALL expose `GET /events` as a server-sent events (SSE) stream delivering `DomainEvent` payloads as JSON; clients MAY filter by `feature_id` query parameter | E5.4 | `crates/agileplus-api/` |

---

## FR-GRPC: gRPC Service Layer

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-GRPC-001 | System SHALL define gRPC endpoints for feature CRUD (Create, Get, Update, Delete, List) via protobuf definitions in `proto/agileplus/v1/` | E1.1 | `proto/agileplus/v1/`, `crates/agileplus-grpc/` |
| FR-GRPC-002 | System SHALL define `TransitionFeature` RPC with `TransitionRequest { feature_id, target_state }` and `TransitionResponse { audit_entry }` | E1.2 | `proto/agileplus/v1/` |
| FR-GRPC-003 | System SHALL define RPCs for agent dispatch: `SpawnAgent`, `MonitorAgent`, `RequestReview`, `SubmitReview` | E6.1 | `proto/agileplus/v1/` |
| FR-GRPC-004 | System SHALL generate Rust bindings via `tonic`/`prost` (stored in `rust/`) and Python stubs via `grpcio` (stored in `python/`) from the same proto definitions | E8.6 | `rust/`, `python/`, `buf.gen.yaml` |
| FR-GRPC-005 | The gRPC server SHALL use TLS when `config.grpc.tls_cert_path` and `config.grpc.tls_key_path` are set; plaintext SHALL be allowed only in development mode | E5.5 | `crates/agileplus-grpc/` |

---

## FR-STORAGE: Persistence Layer

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-STORAGE-001 | The SQLite adapter (`agileplus-sqlite`) SHALL implement all repository port traits defined in `crates/agileplus-domain/src/ports/`; no other crate SHALL depend directly on SQLite | E9.1 | `crates/agileplus-sqlite/` |
| FR-STORAGE-002 | The SQLite adapter SHALL use WAL journal mode and `PRAGMA synchronous=NORMAL` for local-first performance | E9.1 | `crates/agileplus-sqlite/` |
| FR-STORAGE-003 | All schema migrations SHALL be embedded in the binary via `sqlx::migrate!` and applied automatically on startup | E9.1 | `crates/agileplus-sqlite/` |
| FR-STORAGE-004 | The cache layer (`agileplus-cache`) SHALL implement in-memory LRU caching for frequently-accessed features and work packages with configurable TTL | E9.2 | `crates/agileplus-cache/` |

---

## FR-EVENTS: Event Bus

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-EVENTS-001 | System SHALL publish domain events to NATS JetStream when features or work packages change state; event subjects SHALL follow the pattern `agileplus.features.{feature_id}.{event_type}` | E7.3 | `crates/agileplus-nats/`, `crates/agileplus-events/` |
| FR-EVENTS-002 | System SHALL define `DomainEvent` with fields: `id`, `event_type`, `feature_id`, `wp_id` (Option), `payload` (JSON), `timestamp`, `actor` | E3.2 | `crates/agileplus-events/` |
| FR-EVENTS-003 | The NATS adapter SHALL reconnect with exponential backoff (max 30 s, max 10 retries) on connection loss; pending events SHALL be buffered in SQLite during disconnect | E7.3 | `crates/agileplus-nats/` |

---

## FR-GRAPH: Graph Storage and Dependency Queries

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-GRAPH-001 | The graph crate (`agileplus-graph`) SHALL define `FeatureNode`, `WorkPackageNode`, `ModuleNode`, `CycleNode`, and `DeviceNode` as typed Neo4j node structs with all corresponding domain entity fields | E10.1 | `crates/agileplus-graph/src/nodes.rs` |
| FR-GRAPH-002 | The graph crate SHALL define relationship types: `DependsOn`, `BelongsTo`, `AssignedTo`, `PartOf`, `SyncsWith`; each relationship SHALL carry a `weight` or `dep_type` attribute where applicable | E10.2 | `crates/agileplus-graph/src/relationships.rs` |
| FR-GRAPH-003 | The `GraphStore` implementation SHALL use the Neo4j Bolt protocol (via `neo4rs` or equivalent); all Cypher queries SHALL be parameterized to prevent injection | E10.3 | `crates/agileplus-graph/src/store.rs` |
| FR-GRAPH-004 | The graph crate SHALL expose query functions for: topological ordering of WPs, detection of dependency cycles, identification of blocked WPs (those with incomplete `DependsOn` predecessors), and critical path to a target WP | E10.4 | `crates/agileplus-graph/src/queries.rs` |
| FR-GRAPH-005 | The graph crate SHALL expose a `health()` function returning `ServiceHealth` by verifying connectivity to the Neo4j instance; failures SHALL be reported as `Unhealthy` with a descriptive message | E10.5 | `crates/agileplus-graph/src/health.rs` |

---

## FR-IMPORT: Import Subsystem

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-IMPORT-001 | The import crate (`agileplus-import`) SHALL define an `ImportManifest` struct specifying a list of features and work packages to import, with optional field mappings from source system field names to AgilePlus field names | E7.4 | `crates/agileplus-import/src/manifest.rs` |
| FR-IMPORT-002 | The `Importer` SHALL validate each manifest entry against the domain model before persisting; validation errors SHALL be collected and returned in an `ImportReport` without aborting the remaining entries (partial import) | E7.4 | `crates/agileplus-import/src/importer/` |
| FR-IMPORT-003 | The `ImportReport` SHALL record per-entry outcomes: `imported`, `skipped` (already exists), `failed` (validation error), and `total`; the report SHALL be serializable to JSON | E7.4 | `crates/agileplus-import/src/report.rs` |
| FR-IMPORT-004 | Import operations SHALL be idempotent: re-importing an entity with the same slug SHALL update non-key fields and emit an audit entry for the update, not create a duplicate | E7.4 | `crates/agileplus-import/src/importer/` |

---

## FR-TRIAGE: Triage Engine

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-TRIAGE-001 | The triage crate (`agileplus-triage`) SHALL define a `Classifier` that categorizes incoming work items into intent types: `Bug`, `Feature`, `Idea`, `Task` based on title, description, and metadata | E4.9 | `crates/agileplus-triage/src/classifier.rs` |
| FR-TRIAGE-002 | The `Classifier` SHALL compute a priority score (0-100) for each classified item based on configurable heuristics (e.g., keyword weights, severity signals, staleness) | E4.9 | `crates/agileplus-triage/src/classifier.rs` |
| FR-TRIAGE-003 | The triage crate SHALL define a `Router` that routes classified items to the appropriate backlog by project; routing rules SHALL be configurable via policy | E4.9 | `crates/agileplus-triage/src/router.rs` |
| FR-TRIAGE-004 | The triage subsystem SHALL operate on the `Backlog` domain entity; classified items that pass routing SHALL be appended to the project backlog with the computed priority | E4.9 | `crates/agileplus-triage/src/backlog.rs` |

---

## FR-PLANE: Plane.so Integration

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-PLANE-001 | The Plane adapter SHALL map `Feature.state` to Plane issue state IDs via `SyncMapping { agileplus_state, plane_state_id }` stored in `sync_mappings` table | E7.1 | `crates/agileplus-plane/`, `crates/agileplus-domain/src/domain/sync_mapping.rs` |
| FR-PLANE-002 | The sync process SHALL update `Feature.plane_issue_id` and `Feature.plane_state_id` on successful Plane sync; sync failures SHALL be logged and retried | E7.1 | `crates/agileplus-sync/` |
| FR-PLANE-003 | The Plane adapter SHALL create Plane issues for newly-created features that lack a `plane_issue_id`; duplicate creation SHALL be idempotent | E7.1 | `crates/agileplus-plane/` |

---

## FR-GIT: Git VCS Integration

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-GIT-001 | The Git adapter SHALL resolve the current commit SHA for any working directory and record it as `Feature.created_at_commit` or `Feature.last_modified_commit` | E1.1 | `crates/agileplus-git/` |
| FR-GIT-002 | The Git adapter SHALL create and delete worktrees at the path specified in `WorkPackage.worktree_path` using `git worktree add` and `git worktree remove` | E1.3 | `crates/agileplus-git/` |
| FR-GIT-003 | The GitHub adapter (`agileplus-github`) SHALL create pull requests with title, body, base branch, and head branch; PR URL SHALL be stored in `WorkPackage.pr_url` | E7.2 | `crates/agileplus-github/` |

---

## FR-GOVERN: Governance Engine

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-GOVERN-001 | The governance engine SHALL define `GovernanceContract` specifying required evidence types (test output, CI log, security scan, review approval) per state transition | E2.1 | `crates/agileplus-domain/src/domain/governance.rs` |
| FR-GOVERN-002 | Any state transition SHALL be blocked if the required evidence for that transition is not attached to the feature or work package; the API SHALL return HTTP 422 with a list of missing evidence IDs | E2.2 | `crates/agileplus-api/`, `crates/agileplus-domain/src/domain/governance.rs` |
| FR-GOVERN-003 | The triage engine (`agileplus-triage`) SHALL classify incoming tickets by severity, estimate effort, and assign to a backlog with a priority score | E4.9 | `crates/agileplus-triage/` |
| FR-GOVERN-004 | The telemetry crate SHALL expose Prometheus counters for: `agileplus_features_total`, `agileplus_transitions_total`, `agileplus_audit_entries_total`, `agileplus_governance_violations_total` | E12.1 | `crates/agileplus-telemetry/` |

---

## FR-P2P: Peer-to-Peer Replication

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-P2P-001 | The P2P crate (`agileplus-p2p`) SHALL assign a unique `VectorClock` to each mutable domain entity; clocks SHALL be incremented on every local write and merged on sync using component-wise maximum | E11.2 | `crates/agileplus-p2p/src/vector_clock.rs` |
| FR-P2P-002 | The P2P crate SHALL implement device discovery via mDNS (`_agileplus._tcp`) or static address configuration; discovered devices SHALL be registered in the `DeviceNode` store | E11.1 | `crates/agileplus-p2p/src/discovery.rs` |
| FR-P2P-003 | The export subsystem SHALL produce a portable state archive (JSON or binary) containing features, work packages, audit entries, and vector clocks for a configurable entity filter | E11.3 | `crates/agileplus-p2p/src/export.rs` |
| FR-P2P-004 | The import subsystem SHALL merge an incoming state archive using vector clock comparison; entities with concurrent edits (clocks incomparable) SHALL be flagged in a conflict report for manual resolution | E11.3 | `crates/agileplus-p2p/src/import.rs` |
| FR-P2P-005 | The git-merge module SHALL merge git-adjacent metadata (branch names, commit SHAs, worktree paths) between devices using git object model semantics; non-conflicting git refs SHALL be merged automatically | E11.4 | `crates/agileplus-p2p/src/git_merge.rs` |
| FR-P2P-006 | The replication session SHALL authenticate devices using a pre-shared key or device certificate; unauthenticated replication requests SHALL be rejected | E11.5 | `crates/agileplus-p2p/src/replication.rs` |

---

## FR-AGENT: Agent Dispatch and Review Ports

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-AGENT-001 | The domain SHALL define an `AgentPort` trait with methods: `spawn(config: AgentConfig) -> AgentHandle`, `status(handle: &AgentHandle) -> AgentStatus`, `collect_result(handle: AgentHandle) -> AgentResult` | E6.1 | `crates/agileplus-domain/src/ports/agent.rs` |
| FR-AGENT-002 | `AgentConfig` SHALL include: `backend` (ClaudeCode | Codex), `prompt` (String), `worktree_path` (PathBuf), `context_files` (Vec<PathBuf>), `max_review_cycles` (u32), `timeout_secs` (u64), `extra_args` (Vec<String>) | E6.1 | `crates/agileplus-domain/src/ports/agent.rs` |
| FR-AGENT-003 | `AgentResult` SHALL include: `pr_url` (Option<String>), `commit_sha` (Option<String>), `stdout` (String), `stderr` (String), `exit_code` (i32), `duration_ms` (u64) | E6.2 | `crates/agileplus-domain/src/ports/agent.rs` |
| FR-AGENT-004 | The domain SHALL define a `ReviewPort` trait with methods: `request_review(wp_id: i64, pr_url: &str) -> ReviewRequest`, `submit_review(request_id: i64, comments: Vec<ReviewComment>) -> ReviewResult` | E6.3 | `crates/agileplus-domain/src/ports/review.rs` |
| FR-AGENT-005 | `ReviewComment` SHALL carry `severity` (Critical | Major | Minor | Informational), `file_path` (Option<String>), `line` (Option<u32>), `body` (String), `actionable` (bool); only `actionable` comments SHALL be dispatched back to the agent | E6.3 | `crates/agileplus-domain/src/ports/review.rs` |

---

## FR-CONTENT: Content Storage Port

| ID | Requirement | Traces To | Code Location |
|----|-------------|-----------|---------------|
| FR-CONTENT-001 | The domain SHALL define a `ContentStoragePort` trait with methods: `store(key: &str, content: &[u8]) -> Result<()>`, `retrieve(key: &str) -> Result<Vec<u8>>`, `delete(key: &str) -> Result<()>` | E9.4 | `crates/agileplus-domain/src/ports/content.rs` |
| FR-CONTENT-002 | The MinIO adapter SHALL implement `ContentStoragePort`; object keys SHALL follow the pattern `{entity_type}/{entity_id}/{artifact_type}` | E9.4 | `crates/agileplus-domain/src/ports/content.rs` |
| FR-CONTENT-003 | Spec content (spec.md), plan artifacts (plan.md), and agent output (stdout/stderr) SHALL be stored via `ContentStoragePort`; references to stored content SHALL use the object key | E9.4 | `crates/agileplus-sqlite/` |
