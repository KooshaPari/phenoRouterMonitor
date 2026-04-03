# AgilePlus: Code Entity Map

Maps key code entities (structs, traits, modules, functions) to their corresponding Functional Requirements (FRs).

---

## Domain Entities (FR-DOMAIN-*)

### Feature and State Management

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `Feature` | `crates/agileplus-domain/src/domain/feature.rs` | Struct | FR-DOMAIN-001 | Core feature entity with all metadata fields |
| `FeatureState` | `crates/agileplus-domain/src/domain/state_machine.rs` | Enum | FR-DOMAIN-002 | Feature state: Created, Specified, Researched, Planned, Implementing, Validated, Shipped, Retrospected |
| `StateMachine<Feature>` | `crates/agileplus-domain/src/domain/state_machine.rs` | Trait | FR-DOMAIN-003 | Enforces forward-only state transitions |
| `StateTransition` | `crates/agileplus-domain/src/domain/state_machine.rs` | Struct | FR-DOMAIN-004 | Captures from, to, and skipped states |

### Work Packages and Dependencies

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `WorkPackage` | `crates/agileplus-domain/src/domain/work_package/` | Struct | FR-DOMAIN-005 | Work package with ordinal, title, description, state, file scope |
| `WpState` | `crates/agileplus-domain/src/domain/work_package/` | Enum | FR-DOMAIN-006 | Work package state: Planned, Doing, Review, Done, Blocked |
| `WorkPackageDependency` | `crates/agileplus-domain/src/domain/work_package/` | Struct | FR-DOMAIN-007 | Dependency with from_wp_id, to_wp_id, dep_type; cycle detection via topological sort |

### Organization

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `Module` | `crates/agileplus-domain/src/domain/module.rs` | Struct | FR-DOMAIN-008 | Module with slug, name, owner, scoped to project |
| `Project` | `crates/agileplus-domain/src/domain/project.rs` | Struct | FR-DOMAIN-009 | Project entity; features and modules are scoped via project_id |
| `Backlog` | `crates/agileplus-domain/src/domain/backlog.rs` | Struct | FR-DOMAIN-010 | Prioritized feature collection with priority (i32) and added_at |
| `Cycle` | `crates/agileplus-domain/src/domain/cycle/` | Struct | FR-DOMAIN-011 | Cycle with state machine (Draft, Active, Completed, Archived) |

### Cross-cutting

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `Snapshot` | `crates/agileplus-domain/src/domain/snapshot.rs` | Struct | FR-DOMAIN-012 | Materialized state at event sequence number |
| `Metric` | `crates/agileplus-domain/src/domain/metric.rs` | Struct | FR-DOMAIN-013 | Telemetry: command, feature_id, duration_ms, agent_runs, review_cycles |
| `ServiceHealth` | `crates/agileplus-domain/src/domain/service_health.rs` | Struct | FR-DOMAIN-014 | Status (Healthy, Degraded, Unhealthy), message, timestamp |
| `ApiKey` | `crates/agileplus-domain/src/domain/api_key.rs` | Struct | FR-DOMAIN-015 | key_hash (SHA-256), actor, created_at, expires_at (never plaintext) |
| `DeviceNode` | `crates/agileplus-domain/src/domain/device_node.rs` | Struct | FR-DOMAIN-016 | device_id (UUID), hostname, address, last_seen for P2P |

---

## Audit and Evidence (FR-AUDIT-*)

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `AuditEntry` | `crates/agileplus-domain/src/domain/audit.rs` | Struct | FR-AUDIT-001 | Immutable audit log entry with hash chain fields |
| `hash_entry()` | `crates/agileplus-domain/src/domain/audit.rs` | Function | FR-AUDIT-002 | SHA-256 hash computation; genesis uses [0u8;32] |
| `verify_chain()` | `crates/agileplus-domain/src/domain/audit.rs` | Function | FR-AUDIT-003 | Chain verification; returns AuditChainError variants |
| `EvidenceRef` | `crates/agileplus-domain/src/domain/audit.rs` | Struct | FR-AUDIT-004 | evidence_id and fr_id (links audit to evidence) |
| `FeatureRepository` | `crates/agileplus-sqlite/` | Trait | FR-AUDIT-005 | Persist audit immutably via SQLite port |
| `audit::archive()` | `crates/agileplus-domain/src/domain/audit.rs` | Function | FR-AUDIT-006 | Archive entries to MinIO; populate archived_to field |

---

## API Routes (FR-API-*, FR-CLI-*)

### HTTP REST API (Axum)

| Route | Path | Handler | Maps To FR | Purpose |
|-------|------|---------|-----------|---------|
| `POST /features` | `crates/agileplus-api/src/routes/features.rs` | `create_feature` | FR-API-001 | Create feature |
| `GET /features` | `crates/agileplus-api/src/routes/features.rs` | `list_features` | FR-API-001 | List features |
| `GET /features/{id}` | `crates/agileplus-api/src/routes/features.rs` | `get_feature` | FR-API-001 | Get feature by ID |
| `PUT /features/{id}` | `crates/agileplus-api/src/routes/features.rs` | `update_feature` | FR-API-001 | Update feature |
| `DELETE /features/{id}` | `crates/agileplus-api/src/routes/features.rs` | `delete_feature` | FR-API-001 | Delete feature |
| `POST /features/{id}/transition` | `crates/agileplus-api/src/routes/features.rs` | `transition_feature` | FR-API-002 | Transition feature state; emit audit |
| `GET /features/{id}/work-packages` | `crates/agileplus-api/src/routes/features.rs` | `list_work_packages` | FR-API-003 | List work packages for feature |
| `POST /features/{id}/work-packages` | `crates/agileplus-api/src/routes/features.rs` | `create_work_package` | FR-API-003 | Create work package |
| `GET /features/{id}/audit` | `crates/agileplus-api/src/routes/audit.rs` | `get_audit_trail` | FR-API-004 | Audit trail (hash-chained) |
| `GET /health` | `crates/agileplus-api/src/routes/core.rs` | `health` | FR-API-005 | Health check |
| `GET /metrics` | `crates/agileplus-api/src/routes/core.rs` | `metrics` | FR-API-006 | Prometheus metrics |
| `GET /events` | `crates/agileplus-api/src/routes/events.rs` | `events_sse` | FR-API-008 | Server-sent events stream |

### API Key Authentication

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `ApiKeyMiddleware` | `crates/agileplus-api/src/middleware/auth.rs` | Middleware | FR-API-007 | Validates API key from Authorization: Bearer header; returns 401 if missing/invalid |

### CLI Commands (src/cli or subcommands)

| Command | Path | Maps To FR | Purpose |
|---------|------|-----------|---------|
| `feature create` | `crates/agileplus-cli/` | FR-CLI-001 | Create feature in Created state |
| `feature list` | `crates/agileplus-cli/` | FR-CLI-002 | List features (filtered by state/project) |
| `feature transition` | `crates/agileplus-cli/` | FR-CLI-003 | Transition feature state |
| `wp create` | `crates/agileplus-cli/` | FR-CLI-004 | Create work package |
| `wp list` | `crates/agileplus-cli/` | FR-CLI-004 | List work packages for feature |
| `status` | `crates/agileplus-cli/` | FR-CLI-005 | Update work package state |
| `specify` | `crates/agileplus-subcmds/` | FR-CLI-006 | Create spec with AI assistance |
| `triage` | `crates/agileplus-triage/` | FR-CLI-007 | Triage incoming issues |
| `sync` | `crates/agileplus-sync/` | FR-CLI-008 | Sync with Plane.so |
| `dashboard` | `crates/agileplus-dashboard/` | FR-CLI-009 | Launch htmx dashboard |
| `import` | `crates/agileplus-import/` | FR-CLI-010 | Import from manifest |
| `validate` | `crates/agileplus-subcmds/` | FR-CLI-011 | Validate governance (NOT IMPLEMENTED) |

---

## gRPC Service (FR-GRPC-*)

### Proto Definitions

| Service/Message | Proto File | Maps To FR | Purpose |
|-----------------|-----------|-----------|---------|
| `AgilePlusCoreService` | `proto/agileplus/v1/core.proto` | FR-GRPC-001, FR-GRPC-002 | Feature CRUD + transition RPCs |
| `AgentDispatchService` | `proto/agileplus/v1/agents.proto` | FR-GRPC-003 | SpawnAgent, MonitorAgent, RequestReview, SubmitReview RPCs |
| `Feature` message | `proto/agileplus/v1/common.proto` | FR-GRPC-001 | Common feature proto message |
| Code Generation | `buf.gen.yaml` | FR-GRPC-004 | Generates Rust (via tonic/prost) and Python (via grpcio) |
| gRPC Server Config | `crates/agileplus-grpc/src/` | FR-GRPC-005 | TLS config (uses cert/key paths from config) |

---

## Storage and Persistence (FR-STORAGE-*, FR-CONTENT-*)

### SQLite Adapter

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `SqliteRepository` | `crates/agileplus-sqlite/` | Struct | FR-STORAGE-001 | Implements all RepositoryPort traits |
| `init_db()` | `crates/agileplus-sqlite/src/lib.rs` | Function | FR-STORAGE-002, FR-STORAGE-003 | WAL mode, PRAGMA sync, embedded migrations |
| `migrations/` | `crates/agileplus-sqlite/src/migrations/` | Files | FR-STORAGE-003 | Embedded via sqlx::migrate!; auto-applied on startup |

### Cache Layer

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `LruCache` | `crates/agileplus-cache/` | Struct | FR-STORAGE-004 | LRU caching for features/work packages with TTL |

### Content Storage

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `ContentStoragePort` | `crates/agileplus-domain/src/ports/content.rs` | Trait | FR-CONTENT-001 | store, retrieve, delete operations |
| `MinioAdapter` | `crates/agileplus-domain/src/ports/content.rs` | Struct | FR-CONTENT-002 | Implements ContentStoragePort for MinIO; {entity_type}/{id}/{artifact_type} keys |
| `store_spec()`, `store_plan()` | `crates/agileplus-sqlite/` | Functions | FR-CONTENT-003 | Store via ContentStoragePort |

---

## Event Bus and Messaging (FR-EVENTS-*)

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `DomainEvent` | `crates/agileplus-events/` | Struct | FR-EVENTS-002 | Event with id, event_type, feature_id, wp_id, payload, timestamp, actor |
| `NatsEventBus` | `crates/agileplus-nats/src/bus/` | Struct | FR-EVENTS-001 | Publishes to NATS JetStream (agileplus.features.{id}.{type}) |
| `NatsEventBus::reconnect()` | `crates/agileplus-nats/src/bus/` | Method | FR-EVENTS-003 | Exponential backoff (max 30s, 10 retries); buffer pending events |

---

## Graph Storage and Dependency Management (FR-GRAPH-*)

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `FeatureNode` | `crates/agileplus-graph/src/nodes.rs` | Struct | FR-GRAPH-001 | Neo4j node for Feature |
| `WorkPackageNode` | `crates/agileplus-graph/src/nodes.rs` | Struct | FR-GRAPH-001 | Neo4j node for WorkPackage |
| `ModuleNode` | `crates/agileplus-graph/src/nodes.rs` | Struct | FR-GRAPH-001 | Neo4j node for Module |
| `CycleNode` | `crates/agileplus-graph/src/nodes.rs` | Struct | FR-GRAPH-001 | Neo4j node for Cycle |
| `DeviceNode` | `crates/agileplus-graph/src/nodes.rs` | Struct | FR-GRAPH-001 | Neo4j node for Device (P2P) |
| `DependsOn`, `BelongsTo`, `AssignedTo`, `PartOf`, `SyncsWith` | `crates/agileplus-graph/src/relationships.rs` | Enums | FR-GRAPH-002 | Relationship types with weight/dep_type attributes |
| `GraphStore` | `crates/agileplus-graph/src/store.rs` | Struct | FR-GRAPH-003 | Neo4j Bolt connection; parameterized Cypher queries |
| `topological_sort()` | `crates/agileplus-graph/src/queries.rs` | Function | FR-GRAPH-004 | Order work packages by dependency |
| `detect_cycles()` | `crates/agileplus-graph/src/queries.rs` | Function | FR-GRAPH-004 | Detect dependency cycles |
| `identify_blocked_wps()` | `crates/agileplus-graph/src/queries.rs` | Function | FR-GRAPH-004 | Find WPs with incomplete DependsOn predecessors |
| `critical_path_to()` | `crates/agileplus-graph/src/queries.rs` | Function | FR-GRAPH-004 | Calculate critical path to target WP |
| `health()` | `crates/agileplus-graph/src/health.rs` | Function | FR-GRAPH-005 | Neo4j connectivity check; returns ServiceHealth |

---

## Import Subsystem (FR-IMPORT-*)

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `ImportManifest` | `crates/agileplus-import/src/manifest.rs` | Struct | FR-IMPORT-001 | Feature/WP list with optional field mappings |
| `Importer` | `crates/agileplus-import/src/importer/` | Struct | FR-IMPORT-002 | Validates and imports manifest entries; collects errors without aborting |
| `ImportReport` | `crates/agileplus-import/src/report.rs` | Struct | FR-IMPORT-003 | Outcome counts (imported, skipped, failed, total); JSON serializable |
| `Importer::import_idempotent()` | `crates/agileplus-import/src/importer/` | Method | FR-IMPORT-004 | Re-import upserts non-key fields; emits audit entry |

---

## Triage Engine (FR-TRIAGE-*, FR-GOVERN-003)

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `Classifier` | `crates/agileplus-triage/src/classifier.rs` | Struct | FR-TRIAGE-001 | Categorizes items into Bug, Feature, Idea, Task |
| `Classifier::compute_priority()` | `crates/agileplus-triage/src/classifier.rs` | Method | FR-TRIAGE-002 | Score (0-100) based on heuristics |
| `Router` | `crates/agileplus-triage/src/router.rs` | Struct | FR-TRIAGE-003 | Routes classified items to backlogs by project |
| `BacklogAppender` | `crates/agileplus-triage/src/backlog.rs` | Struct | FR-TRIAGE-004 | Appends classified items to Backlog with priority |

---

## Plane.so Integration (FR-PLANE-*)

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `SyncMapping` | `crates/agileplus-domain/src/domain/sync_mapping.rs` | Struct | FR-PLANE-001 | Maps Feature.state to Plane issue state IDs |
| `PlaneAdapter` | `crates/agileplus-plane/` | Struct | FR-PLANE-001, FR-PLANE-003 | Plane API integration |
| `sync_feature()` | `crates/agileplus-sync/` | Function | FR-PLANE-002 | Updates Feature.plane_issue_id and plane_state_id |
| `create_plane_issue()` | `crates/agileplus-plane/` | Function | FR-PLANE-003 | Idempotent issue creation for new features |

---

## Git VCS Integration (FR-GIT-*)

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `GitAdapter` | `crates/agileplus-git/` | Struct | FR-GIT-001 | Resolves current commit SHA |
| `GitAdapter::resolve_commit()` | `crates/agileplus-git/` | Method | FR-GIT-001 | Resolves and records in Feature.created_at_commit / last_modified_commit |
| `GitAdapter::worktree_add()` | `crates/agileplus-git/` | Method | FR-GIT-002 | Creates git worktree at WorkPackage.worktree_path |
| `GitAdapter::worktree_remove()` | `crates/agileplus-git/` | Method | FR-GIT-002 | Removes git worktree |
| `GitHubAdapter` | `crates/agileplus-github/` | Struct | FR-GIT-003 | GitHub API integration |
| `GitHubAdapter::create_pr()` | `crates/agileplus-github/` | Method | FR-GIT-003 | Creates PR; stores URL in WorkPackage.pr_url |

---

## Governance (FR-GOVERN-*)

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `GovernanceContract` | `crates/agileplus-domain/src/domain/governance.rs` | Struct | FR-GOVERN-001 | Specifies required evidence per state transition |
| `validate_governance()` | `crates/agileplus-api/src/routes/features.rs` | Function | FR-GOVERN-002 | Blocks transition if evidence missing; returns HTTP 422 |
| `TelemetryCollector` | `crates/agileplus-telemetry/` | Struct | FR-GOVERN-004 | Exposes Prometheus counters: features_total, transitions_total, audit_entries_total, governance_violations_total |

---

## Peer-to-Peer Replication (FR-P2P-*)

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `VectorClock` | `crates/agileplus-p2p/src/vector_clock.rs` | Struct | FR-P2P-001 | Logical clock per entity; merge via component-wise max |
| `DeviceDiscovery` | `crates/agileplus-p2p/src/discovery.rs` | Struct | FR-P2P-002 | mDNS (`_agileplus._tcp`) or static address config |
| `StateArchive` | `crates/agileplus-p2p/src/export.rs` | Struct | FR-P2P-003 | JSON/binary archive with features, WPs, audit, vector clocks |
| `StateArchive::filter()` | `crates/agileplus-p2p/src/export.rs` | Method | FR-P2P-003 | Configurable entity filtering |
| `ConflictResolver` | `crates/agileplus-p2p/src/import.rs` | Struct | FR-P2P-004 | Merge incoming state; flag conflicts (incomparable clocks) |
| `ConflictReport` | `crates/agileplus-p2p/src/import.rs` | Struct | FR-P2P-004 | Lists conflicts for manual resolution |
| `GitMetadataMerge` | `crates/agileplus-p2p/src/git_merge.rs` | Struct | FR-P2P-005 | Merge branch names, commit SHAs, worktree paths using git semantics |
| `ReplicationSession` | `crates/agileplus-p2p/src/replication.rs` | Struct | FR-P2P-006 | PSK or certificate-based authentication; rejects unauthenticated requests |

---

## Agent Dispatch and Review (FR-AGENT-*)

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `AgentPort` | `crates/agileplus-domain/src/ports/agent.rs` | Trait | FR-AGENT-001 | spawn(), status(), collect_result() methods |
| `AgentConfig` | `crates/agileplus-domain/src/ports/agent.rs` | Struct | FR-AGENT-002 | backend, prompt, worktree_path, context_files, max_review_cycles, timeout_secs, extra_args |
| `AgentHandle` | `crates/agileplus-domain/src/ports/agent.rs` | Type | FR-AGENT-001, FR-AGENT-002 | Handle to spawned agent |
| `AgentResult` | `crates/agileplus-domain/src/ports/agent.rs` | Struct | FR-AGENT-003 | pr_url, commit_sha, stdout, stderr, exit_code, duration_ms |
| `ReviewPort` | `crates/agileplus-domain/src/ports/review.rs` | Trait | FR-AGENT-004 | request_review(), submit_review() methods |
| `ReviewComment` | `crates/agileplus-domain/src/ports/review.rs` | Struct | FR-AGENT-005 | severity, file_path, line, body, actionable; only actionable dispatched to agent |

---

## Telemetry and Observability

| Entity | Path | Type | Maps To FR | Purpose |
|--------|------|------|-----------|---------|
| `TelemetryCollector` | `crates/agileplus-telemetry/` | Struct | FR-DOMAIN-013, FR-GOVERN-004 | Records metrics and Prometheus counters |
| `Metric` | `crates/agileplus-domain/src/domain/metric.rs` | Struct | FR-DOMAIN-013 | command, feature_id, duration_ms, agent_runs, review_cycles, recorded_at |

---

## Reverse Index: FR to Primary Code Location

For quick lookup of where each FR is primarily implemented:

| FR Range | Primary Crate | Key Files |
|----------|--------------|-----------|
| FR-DOMAIN-* | agileplus-domain | `src/domain/*.rs` |
| FR-AUDIT-* | agileplus-domain + agileplus-sqlite | `src/domain/audit.rs` + `agileplus-sqlite/` |
| FR-CLI-* | agileplus-cli + subcommand crates | `crates/agileplus-cli/`, `agileplus-subcmds/`, others |
| FR-API-* | agileplus-api | `src/routes/*.rs`, `src/middleware/auth.rs` |
| FR-GRPC-* | agileplus-grpc + proto | `proto/agileplus/v1/`, `crates/agileplus-grpc/` |
| FR-STORAGE-* | agileplus-sqlite + agileplus-cache | `crates/agileplus-sqlite/`, `agileplus-cache/` |
| FR-EVENTS-* | agileplus-nats + agileplus-events | `crates/agileplus-nats/`, `agileplus-events/` |
| FR-GRAPH-* | agileplus-graph | `src/nodes.rs`, `relationships.rs`, `store.rs`, `queries.rs` |
| FR-IMPORT-* | agileplus-import | `src/manifest.rs`, `importer/`, `report.rs` |
| FR-TRIAGE-* | agileplus-triage | `src/classifier.rs`, `router.rs`, `backlog.rs` |
| FR-PLANE-* | agileplus-plane + agileplus-sync | `crates/agileplus-plane/`, `agileplus-sync/` |
| FR-GIT-* | agileplus-git + agileplus-github | `crates/agileplus-git/`, `agileplus-github/` |
| FR-GOVERN-* | agileplus-api + agileplus-telemetry | `src/routes/`, `crates/agileplus-telemetry/` |
| FR-P2P-* | agileplus-p2p | `src/vector_clock.rs`, `discovery.rs`, `export.rs`, `import.rs`, `git_merge.rs`, `replication.rs` |
| FR-AGENT-* | agileplus-domain | `src/ports/agent.rs`, `ports/review.rs` |
| FR-CONTENT-* | agileplus-domain + agileplus-sqlite | `src/ports/content.rs` + SQLite integration |

---

## Notes

1. **Port-Based Architecture**: Domain defines traits (AgentPort, ReviewPort, ContentStoragePort); implementations in domain itself or adapters (SQLite, Plane, etc.)
2. **Crate Dependency Graph**: CLI -> API/Domain; API -> Domain + Storage + Ports; Storage -> Domain; all -> Telemetry
3. **Test Locations**: Tests co-located with crates (e.g., `agileplus-api/tests/`, `src/*/tests.rs` inline modules)
4. **Proto Codegen**: buf.gen.yaml produces both Rust (rust/) and Python (python/) bindings from proto definitions
5. **Cross-Crate Traceability**: Use grep for "Traces to:" or "// FR-" comments to find all references
