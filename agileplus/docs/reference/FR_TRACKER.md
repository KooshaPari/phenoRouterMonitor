# AgilePlus: FR Implementation Tracker

**Version:** 2.2 | **Last Updated:** 2026-03-28 | **Total FRs:** 63

## Summary

| Category | Total | Implemented | Partial | Missing |
|----------|-------|-------------|---------|---------|
| FR-DOMAIN (Domain Model) | 16 | 15 | 1 | 0 |
| FR-AUDIT (Immutable Audit Trail) | 6 | 5 | 1 | 0 |
| FR-CLI (Command-Line Interface) | 11 | 8 | 2 | 1 |
| FR-API (HTTP REST API) | 8 | 7 | 1 | 0 |
| FR-GRPC (gRPC Service Layer) | 5 | 4 | 1 | 0 |
| FR-STORAGE (Persistence Layer) | 4 | 3 | 1 | 0 |
| FR-EVENTS (Event Bus) | 3 | 2 | 1 | 0 |
| FR-GRAPH (Graph Storage) | 5 | 3 | 2 | 0 |
| FR-IMPORT (Import Subsystem) | 4 | 2 | 2 | 0 |
| FR-TRIAGE (Triage Engine) | 4 | 2 | 2 | 0 |
| FR-PLANE (Plane.so Integration) | 3 | 2 | 1 | 0 |
| FR-GIT (Git VCS Integration) | 3 | 2 | 1 | 0 |
| FR-GOVERN (Governance Engine) | 4 | 2 | 2 | 0 |
| FR-P2P (Peer-to-Peer Replication) | 6 | 2 | 4 | 0 |
| FR-AGENT (Agent Dispatch) | 5 | 3 | 2 | 0 |
| FR-CONTENT (Content Storage) | 3 | 2 | 1 | 0 |
| **TOTAL** | **63** | **46** | **16** | **1** |

---

## FR-DOMAIN: Domain Model (16 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-DOMAIN-001 | Feature entity with all fields | Implemented | `crates/agileplus-api/tests/` | feature_crud | Maps to E1.1; struct defined in domain |
| FR-DOMAIN-002 | FeatureState enum with ordinals | Implemented | `crates/agileplus-domain/tests/` | state_enum_ordinals | E1.2; forward-only transitions |
| FR-DOMAIN-003 | Forward-only state transitions | Implemented | `crates/agileplus-api/tests/` | transition_rules | E1.2; enforced via state_machine |
| FR-DOMAIN-004 | StateTransition recording skipped states | Implemented | `crates/agileplus-api/tests/` | accelerated_transition | E1.2; tracks intermediate states |
| FR-DOMAIN-005 | WorkPackage entity definition | Implemented | `crates/agileplus-api/tests/` | work_package_crud | E1.3; all fields present |
| FR-DOMAIN-006 | WpState enum and transitions | Implemented | `crates/agileplus-api/tests/` | wp_state_validation | E1.3; Planned, Doing, Review, Done, Blocked |
| FR-DOMAIN-007 | WorkPackageDependency with cycle detection | Partial | `crates/agileplus-integration-tests/tests/` | dependency_cycles | E1.4; topological sort defined but needs test coverage |
| FR-DOMAIN-008 | Module entity | Implemented | `crates/agileplus-api/tests/` | module_crud | E1.5; scoped to project |
| FR-DOMAIN-009 | Project entity with scoping | Implemented | `crates/agileplus-api/tests/` | project_scoping | E1.5; features/modules scoped via project_id |
| FR-DOMAIN-010 | Backlog entity with priority | Implemented | `crates/agileplus-api/tests/` | backlog_priority | E1.5; priority and added_at fields |
| FR-DOMAIN-011 | Cycle entity and state machine | Implemented | `crates/agileplus-api/tests/api_integration/` | cycle_state_transitions | E1.6; Draft, Active, Completed, Archived |
| FR-DOMAIN-012 | Snapshot entity | Partial | `crates/agileplus-domain/tests/` | snapshot_materialization | E3.3; structure defined, test coverage limited |
| FR-DOMAIN-013 | Metric entity for telemetry | Implemented | `crates/agileplus-telemetry/` | metrics_recording | E12.1; command execution telemetry |
| FR-DOMAIN-014 | ServiceHealth entity | Implemented | `crates/agileplus-api/tests/` | health_check | E12.3; Healthy, Degraded, Unhealthy states |
| FR-DOMAIN-015 | ApiKey entity (SHA-256 hashed) | Implemented | `crates/agileplus-api/tests/` | api_key_hashing | E5.5; never stored in plaintext |
| FR-DOMAIN-016 | DeviceNode entity for P2P | Implemented | `crates/agileplus-p2p/` | device_registration | E11.1; UUID, hostname, address fields |

---

## FR-AUDIT: Immutable Audit Trail (6 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-AUDIT-001 | AuditEntry entity definition | Implemented | `crates/agileplus-api/tests/` | audit_entry_structure | E3.1; all fields including hash chain |
| FR-AUDIT-002 | Hash computation (SHA-256) | Implemented | `crates/agileplus-api/tests/` | audit_hash_chain | E3.1; genesis uses [0u8;32] |
| FR-AUDIT-003 | Chain verification function | Implemented | `crates/agileplus-api/tests/` | verify_audit_chain | E3.4; detects hash and prev_hash mismatches |
| FR-AUDIT-004 | EvidenceRef entity | Implemented | `crates/agileplus-api/tests/` | evidence_refs | E2.2; links audit to evidence by FR ID |
| FR-AUDIT-005 | SQLite persistence (immutable) | Partial | `crates/agileplus-sqlite/src/lib/tests.rs` | audit_persistence | E3.1; no in-place modification enforced at DB layer, test coverage varies |
| FR-AUDIT-006 | MinIO archiving support | Implemented | `crates/agileplus-api/tests/` | audit_archiving | E3.5; archived_to field populated |

---

## FR-CLI: Command-Line Interface (11 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-CLI-001 | feature create command | Implemented | `crates/agileplus-cli/tests/` | feature_create_cli | E1.1; creates in Created state |
| FR-CLI-002 | feature list command | Implemented | `crates/agileplus-cli/tests/` | feature_list_cli | E1.1; tabular format with filters |
| FR-CLI-003 | feature transition command | Implemented | `crates/agileplus-cli/tests/` | feature_transition_cli | E1.2; emits audit entry |
| FR-CLI-004 | wp create/list commands | Partial | `crates/agileplus-cli/tests/` | wp_cli_commands | E1.3; implementation incomplete for list |
| FR-CLI-005 | status update command | Implemented | `crates/agileplus-cli/tests/` | status_update_cli | E1.3; wp state transitions |
| FR-CLI-006 | specify command | Implemented | `crates/agileplus-cli/tests/` | specify_command | E4.1; AI-assisted spec generation |
| FR-CLI-007 | triage command | Partial | `crates/agileplus-triage/` | triage_classification | E4.9; classifier/router pipeline incomplete |
| FR-CLI-008 | sync command | Implemented | `crates/agileplus-sync/` | sync_command_integration | E7.1; bidirectional Plane sync |
| FR-CLI-009 | dashboard command | Implemented | `crates/agileplus-dashboard/tests/` | dashboard_htmx | E4.18; htmx-driven web dashboard |
| FR-CLI-010 | import command | Implemented | `crates/agileplus-import/` | import_manifest | E4.15; manifest-based import, idempotent |
| FR-CLI-011 | validate command | Missing | N/A | N/A | E4.6; governance contract validation not implemented |

---

## FR-API: HTTP REST API (8 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-API-001 | Feature CRUD endpoints | Implemented | `crates/agileplus-api/tests/api_integration/` | features_crud | E1.1; POST, GET, PUT, DELETE all covered |
| FR-API-002 | Transition endpoint with audit | Implemented | `crates/agileplus-api/tests/api_integration/` | feature_transition_endpoint | E1.2; returns audit entry |
| FR-API-003 | Work package endpoints | Implemented | `crates/agileplus-api/tests/api_integration/` | features_work_packages | E1.3; GET/POST both present |
| FR-API-004 | Audit endpoint (hash-chained) | Implemented | `crates/agileplus-api/tests/api_integration/` | audit_governance | E3.1; returns full trail sorted by timestamp |
| FR-API-005 | Health endpoint | Implemented | `crates/agileplus-api/tests/api_integration/` | health_check | E12.3; returns status and version |
| FR-API-006 | Metrics endpoint (Prometheus) | Implemented | `crates/agileplus-api/tests/` | metrics_exposition | E12.1; text exposition format |
| FR-API-007 | API key auth (Bearer token) | Implemented | `crates/agileplus-api/tests/api_integration/` | auth_middleware | E5.5; rejects 401 on missing/invalid key |
| FR-API-008 | Server-Sent Events (SSE) | Partial | `crates/agileplus-api/tests/` | sse_stream | E5.4; streaming implemented, feature_id filter incomplete |

---

## FR-GRPC: gRPC Service Layer (5 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-GRPC-001 | Feature CRUD RPC endpoints | Implemented | `proto/agileplus/v1/core.proto` | N/A | E1.1; Create, Get, Update, Delete, List in protobuf |
| FR-GRPC-002 | TransitionFeature RPC | Implemented | `proto/agileplus/v1/core.proto` | N/A | E1.2; with audit entry response |
| FR-GRPC-003 | Agent dispatch RPCs | Partial | `proto/agileplus/v1/agents.proto` | N/A | E6.1; SpawnAgent, MonitorAgent defined, RequestReview incomplete |
| FR-GRPC-004 | Multi-language codegen (Rust + Python) | Implemented | `rust/`, `python/` | N/A | E8.6; both compile and install |
| FR-GRPC-005 | TLS support (conditional) | Implemented | `crates/agileplus-grpc/` | tls_config | E5.5; uses cert/key paths from config |

---

## FR-STORAGE: Persistence Layer (4 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-STORAGE-001 | SQLite adapter implements ports | Implemented | `crates/agileplus-sqlite/` | port_trait_impl | E9.1; all repo traits implemented |
| FR-STORAGE-002 | WAL + PRAGMA sync | Implemented | `crates/agileplus-sqlite/` | wal_configuration | E9.1; checked in initialization |
| FR-STORAGE-003 | Embedded migrations (sqlx::migrate!) | Implemented | `crates/agileplus-sqlite/` | migration_embedding | E9.1; auto-applied on startup |
| FR-STORAGE-004 | LRU cache layer | Partial | `crates/agileplus-cache/` | lru_caching | E9.2; cache structure defined, TTL configuration incomplete |

---

## FR-EVENTS: Event Bus (3 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-EVENTS-001 | NATS JetStream publishing | Implemented | `crates/agileplus-nats/src/bus/tests.rs` | nats_event_publish | E7.3; follows agileplus.features.{id}.{type} pattern |
| FR-EVENTS-002 | DomainEvent entity | Implemented | `crates/agileplus-events/` | domain_event_structure | E3.2; all fields including JSON payload |
| FR-EVENTS-003 | NATS reconnect with exponential backoff | Partial | `crates/agileplus-nats/` | reconnect_strategy | E7.3; backoff implemented, buffering incomplete |

---

## FR-GRAPH: Graph Storage and Dependency Queries (5 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-GRAPH-001 | Typed Neo4j node structs | Implemented | `crates/agileplus-graph/src/nodes.rs` | N/A | E10.1; FeatureNode, WorkPackageNode, ModuleNode, CycleNode, DeviceNode |
| FR-GRAPH-002 | Relationship types with attributes | Implemented | `crates/agileplus-graph/src/relationships.rs` | N/A | E10.2; DependsOn, BelongsTo, AssignedTo, PartOf, SyncsWith |
| FR-GRAPH-003 | Neo4j Bolt + parameterized Cypher | Partial | `crates/agileplus-graph/src/store.rs` | graph_connectivity | E10.3; Bolt connection present, parameterization incomplete |
| FR-GRAPH-004 | Query functions (topo sort, cycle detection, critical path) | Partial | `crates/agileplus-graph/src/queries.rs` | graph_queries | E10.4; topo sort present, critical path incomplete |
| FR-GRAPH-005 | Health check function | Implemented | `crates/agileplus-graph/src/health.rs` | graph_health | E10.5; verifies Neo4j connectivity |

---

## FR-IMPORT: Import Subsystem (4 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-IMPORT-001 | ImportManifest struct | Implemented | `crates/agileplus-import/src/manifest.rs` | manifest_schema | E7.4; supports field mappings |
| FR-IMPORT-002 | Validation without aborting | Partial | `crates/agileplus-import/src/importer/` | partial_import | E7.4; collects errors, incomplete test coverage |
| FR-IMPORT-003 | ImportReport serialization | Implemented | `crates/agileplus-import/src/report.rs` | import_report | E7.4; JSON serializable with counts |
| FR-IMPORT-004 | Idempotent import (upsert) | Implemented | `crates/agileplus-import/src/importer/` | idempotent_import | E7.4; re-importing updates non-key fields |

---

## FR-TRIAGE: Triage Engine (4 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-TRIAGE-001 | Classifier intent detection | Partial | `crates/agileplus-triage/src/classifier.rs` | classify_bug_feature | E4.9; Bug, Feature, Idea, Task classification incomplete |
| FR-TRIAGE-002 | Priority scoring (0-100) | Partial | `crates/agileplus-triage/src/classifier.rs` | priority_scoring | E4.9; scoring heuristics incomplete |
| FR-TRIAGE-003 | Router with configurable rules | Partial | `crates/agileplus-triage/src/router.rs` | route_to_backlog | E4.9; routes classified items, policy configuration incomplete |
| FR-TRIAGE-004 | Backlog appending with priority | Partial | `crates/agileplus-triage/src/backlog.rs` | backlog_append | E4.9; operates on Backlog entity, test coverage limited |

---

## FR-PLANE: Plane.so Integration (3 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-PLANE-001 | State mapping (Plane state IDs) | Implemented | `crates/agileplus-plane/` | sync_mapping | E7.1; SyncMapping entity defined |
| FR-PLANE-002 | Feature sync update | Implemented | `crates/agileplus-sync/` | plane_sync_update | E7.1; updates plane_issue_id and plane_state_id |
| FR-PLANE-003 | Idempotent issue creation | Partial | `crates/agileplus-plane/` | create_plane_issue | E7.1; creation logic present, idempotency test incomplete |

---

## FR-GIT: Git VCS Integration (3 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-GIT-001 | Commit SHA resolution | Implemented | `crates/agileplus-git/` | current_commit_sha | E1.1; resolves and records in Feature |
| FR-GIT-002 | Worktree management (add/delete) | Implemented | `crates/agileplus-api/tests/api_integration/` | worktree_lifecycle | E1.3; uses git worktree add/remove |
| FR-GIT-003 | GitHub PR creation | Partial | `crates/agileplus-github/` | pr_creation | E7.2; creates PRs with metadata, test coverage incomplete |

---

## FR-GOVERN: Governance Engine (4 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-GOVERN-001 | GovernanceContract definition | Implemented | `crates/agileplus-domain/src/domain/governance.rs` | governance_contract | E2.1; specifies required evidence per transition |
| FR-GOVERN-002 | Evidence blocking (HTTP 422) | Implemented | `crates/agileplus-api/tests/api_integration/` | governance_evidence_required | E2.2; returns 422 with missing evidence list |
| FR-GOVERN-003 | Triage classification | Partial | `crates/agileplus-triage/` | triage_severity | E4.9; severity/effort estimation incomplete |
| FR-GOVERN-004 | Prometheus metrics (counters) | Implemented | `crates/agileplus-telemetry/` | prometheus_counters | E12.1; exposes all required counters |

---

## FR-P2P: Peer-to-Peer Replication (6 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-P2P-001 | VectorClock assignment and merge | Partial | `crates/agileplus-p2p/src/vector_clock.rs` | vector_clock_merge | E11.2; clock structure defined, merge logic incomplete |
| FR-P2P-002 | Device discovery (mDNS) | Partial | `crates/agileplus-p2p/src/discovery.rs` | device_mdns_discovery | E11.1; discovery framework incomplete |
| FR-P2P-003 | State archive export | Partial | `crates/agileplus-p2p/src/export.rs` | export_state_archive | E11.3; JSON/binary export, filtering incomplete |
| FR-P2P-004 | Conflict detection and reporting | Partial | `crates/agileplus-p2p/src/import.rs` | conflict_detection | E11.3; vector clock comparison, conflict report incomplete |
| FR-P2P-005 | Git merge for metadata | Partial | `crates/agileplus-p2p/src/git_merge.rs` | git_metadata_merge | E11.4; git merge framework, test coverage limited |
| FR-P2P-006 | Replication authentication | Partial | `crates/agileplus-p2p/src/replication.rs` | psk_authentication | E11.5; auth framework incomplete |

---

## FR-AGENT: Agent Dispatch and Review Ports (5 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-AGENT-001 | AgentPort trait (spawn, status, collect_result) | Implemented | `crates/agileplus-domain/src/ports/agent.rs` | agent_port_trait | E6.1; all methods defined |
| FR-AGENT-002 | AgentConfig struct with all fields | Implemented | `crates/agileplus-domain/src/ports/agent.rs` | agent_config | E6.1; backend, prompt, worktree_path, context_files, timeout, etc. |
| FR-AGENT-003 | AgentResult struct | Implemented | `crates/agileplus-domain/src/ports/agent.rs` | agent_result | E6.2; pr_url, commit_sha, stdout, stderr, exit_code, duration_ms |
| FR-AGENT-004 | ReviewPort trait (request_review, submit_review) | Partial | `crates/agileplus-domain/src/ports/review.rs` | review_port_trait | E6.3; submit_review incomplete |
| FR-AGENT-005 | ReviewComment with severity and actionable flag | Implemented | `crates/agileplus-domain/src/ports/review.rs` | review_comment | E6.3; all fields including actionable flag |

---

## FR-CONTENT: Content Storage Port (3 FRs)

| FR ID | Requirement | Status | Test File | Test Name | Notes |
|-------|-------------|--------|-----------|-----------|-------|
| FR-CONTENT-001 | ContentStoragePort trait (store, retrieve, delete) | Implemented | `crates/agileplus-domain/src/ports/content.rs` | content_port_trait | E9.4; all methods defined |
| FR-CONTENT-002 | MinIO adapter implementation | Implemented | `crates/agileplus-domain/src/ports/content.rs` | minio_adapter | E9.4; follows {entity_type}/{id}/{artifact_type} pattern |
| FR-CONTENT-003 | Spec/plan/output storage via port | Partial | `crates/agileplus-sqlite/` | content_storage_integration | E9.4; integration incomplete for all artifact types |

---

## Key Test Locations

| Crate | Path | Categories Covered |
|-------|------|-------------------|
| agileplus-api | `tests/api_integration/` | FR-API, FR-DOMAIN (core CRUD), FR-AUDIT |
| agileplus-cli | `tests/` | FR-CLI (primary) |
| agileplus-domain | `tests/` | FR-DOMAIN (state machines), FR-AGENT, FR-CONTENT |
| agileplus-nats | `src/bus/tests.rs` | FR-EVENTS |
| agileplus-p2p | `src/*/tests.rs` | FR-P2P |
| agileplus-integration-tests | `tests/` | FR-DOMAIN (complex scenarios), FR-GRAPH |
| agileplus-dashboard | `tests/` | FR-CLI-009 |
| agileplus-sync | (integration tests) | FR-PLANE, FR-EVENTS |
| agileplus-import | (integration tests) | FR-IMPORT |
| agileplus-triage | (integration tests) | FR-TRIAGE, FR-GOVERN |

---

## Traceability Notes

1. **Proto definitions** (FR-GRPC-*): Defined in `proto/agileplus/v1/*.proto`; implementation tests in `crates/agileplus-grpc/tests/`
2. **Missing implementation** (FR-CLI-011): `validate` command for governance contract checking not yet implemented
3. **Partial implementations**: Marked where core functionality exists but test coverage or specific features are incomplete
4. **Vector clock and P2P**: Complex distributed system features with frameworks in place but incomplete test coverage
5. **Triage system**: Classifier and router structures defined; full pipeline and policy configuration incomplete

---

## How to Use This Tracker

1. **To implement a missing FR**: Reference the Code Location and check the corresponding domain/crate structure
2. **To add tests for a Partial FR**: Check the Test File location and add test cases for uncovered functionality
3. **To verify a category**: Sum the Implemented + Partial counts for that category to gauge completion
4. **To find test examples**: See Key Test Locations table for where similar FRs are tested

---

## Maintenance

- Update Status when implementing new FRs or adding test coverage
- Add Test Name when new tests are written referencing specific FRs
- Sync with FUNCTIONAL_REQUIREMENTS.md version when requirements change
