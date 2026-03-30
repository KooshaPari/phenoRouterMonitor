# AgilePlus — Implementation Plan

**Version:** 2.0 | **Status:** In Progress | **Date:** 2026-03-25

This plan maps the PRD epics and stories to phased work packages (WPs), with explicit DAG dependencies and current implementation status. The system is being built as a 24-crate Rust monorepo with hexagonal architecture, a Python MCP server, and integration points for Plane.so, GitHub, and NATS.

---

## Phase 1: Foundation & Core Infrastructure

**Status:** Mostly Complete

Core domain model, storage, event sourcing, and basic CLI scaffolding.

| Task ID | Description | Depends On | Status |
|---------|-------------|------------|--------|
| P1.1 | Domain model: Feature, WorkPackage, Cycle, Module entities with state machines | — | Done |
| P1.2 | Event sourcing: append-only Event stream with hash chains and sequence numbers | — | Done |
| P1.3 | Audit trail: AuditEntry with SHA-256 hash chains for tamper detection | P1.2 | Done |
| P1.4 | SQLite adapter: StoragePort implementation with schema and migrations | P1.1, P1.2 | Done |
| P1.5 | Cache layer: in-process LRU cache for hot-path reads (Feature, WP lookups) | P1.4 | Done |
| P1.6 | Plugin registry: trait-object-based plugin discovery for Storage and VCS adapters | P1.1 | Done |
| P1.7 | Git VCS adapter: Git operations (branch create/delete, worktree management, merge) | — | Done |
| P1.8 | Telemetry: OpenTelemetry tracing, metrics, structured logging infrastructure | — | Done |
| P1.9 | gRPC protocol: core.proto, common.proto with Feature, WP, Cycle messages | P1.1 | Done |
| P1.10 | CLI scaffolding: clap argument parsing, command routing, error handling | — | Done |

**Deliverables:**
- 24 crates, workspace builds cleanly
- SQLite database with full schema
- gRPC proto files (buf lint clean, buf breaking enforced in CI)
- CLI entry point with subcommand routing

---

## Phase 2: Core API & REST Server

**Status:** In Progress

REST API server for feature and work package management, with authentication and streaming.

| Task ID | Description | Depends On | Status |
|---------|-------------|------------|--------|
| P2.1 | Axum REST server: listen on configurable port, health checks | P1.10 | Done |
| P2.2 | Feature CRUD routes: POST/GET/PATCH/DELETE /features | P1.4, P2.1 | Done |
| P2.3 | Work package CRUD routes: POST/GET/PATCH /features/{id}/work-packages | P1.4, P2.1 | Done |
| P2.4 | Cycle CRUD routes: POST/GET/PATCH /cycles | P1.4, P2.1 | Done |
| P2.5 | Module CRUD routes: POST/GET/PATCH /modules | P1.4, P2.1 | Done |
| P2.6 | API key auth middleware: validate keys, extract actor context | P2.1 | Done |
| P2.7 | OpenTelemetry middleware: trace propagation, span creation per request | P1.8, P2.1 | Done |
| P2.8 | Event stream (SSE): /events endpoint for real-time domain event streaming | P1.2, P2.1 | Done |
| P2.9 | Audit trail routes: GET /audit/{entity_type}/{entity_id} | P1.3, P2.1 | Done |
| P2.10 | Error response standardization: consistent JSON error envelopes with traces | P2.1 | Done |

**Deliverables:**
- agileplus-api crate with 10 major route groups
- All CRUD operations return 200/201/400/404/500 with proper semantics
- OpenAPI/Swagger docs generated from routes

---

## Phase 3: Feature Lifecycle State Machine

**Status:** In Progress

Full state machine enforcement: Created → Specified → Researched → Planned → Implementing → Validated → Shipped → Retrospected.

| Task ID | Description | Depends On | Status |
|---------|-------------|------------|--------|
| P3.1 | State transition validation: enum FeatureState with allowed transitions | P1.1 | Done |
| P3.2 | Transition RPC/endpoint: POST /features/{id}/transition with validation | P2.2, P3.1 | Done |
| P3.3 | Transition hooks: pre/post-transition callbacks for side effects | P3.2 | Done |
| P3.4 | Spec hash computation: SHA-256 of spec.md content, versioning | P1.1 | Done |
| P3.5 | Work package state machine: Planned → Doing → Review → Done (with Blocked) | P1.1 | Done |
| P3.6 | WP dependency DAG: cycle detection, topological sort for execution ordering | P1.1, P1.6 | Done |
| P3.7 | Skipped-state tracking: record which intermediate states were skipped (e.g., Specified→Planned) | P3.2 | Done |
| P3.8 | Invalid transition rejection: clear error messages with allowed next states | P3.1 | Done |

**Deliverables:**
- Feature state machine fully enforced
- WP DAG solver produces execution order
- All transitions logged to audit trail

---

## Phase 4: Governance & Evidence Collection

**Status:** Planned

Governance contracts, evidence types, policy rules, and validation engine.

| Task ID | Description | Depends On | Status |
|---------|-------------|------------|--------|
| P4.1 | Governance contract model: versioned contracts binding to features with rules | P1.1 | Partial |
| P4.2 | Evidence types enum: TestResults, CiOutput, SecurityScan, ReviewApproval, LintResults, ManualAttestation | P1.1 | Partial |
| P4.3 | Evidence collection RPC/endpoint: POST /features/{id}/evidence with artifact links | P2.9, P4.2 | Partial |
| P4.4 | Evidence linking to FR IDs: Evidence.fr_ids tracks which functional requirements are covered | P4.3 | Partial |
| P4.5 | Policy rule model: domain, severity (info/warning/error/critical), auto-enforce flag | P1.1 | Partial |
| P4.6 | Policy evaluation engine: assess policies against feature state and evidence | P4.5 | Planned |
| P4.7 | Validation command (CLI): `agileplus validate` runs governance checks, produces reports | P4.1-P4.6 | Planned |
| P4.8 | Validation API endpoint: GET /features/{id}/validate returns compliance status and gaps | P2.9, P4.6 | Planned |
| P4.9 | Governance gap report: list uncovered evidence types and policy failures | P4.6 | Planned |
| P4.10 | Batch evidence import: import test results, CI artifacts, security scans in bulk | P4.3 | Planned |

**Deliverables:**
- agileplus-domain models for Contract, Evidence, PolicyRule
- Governance evaluation engine (50-100 LOC)
- `agileplus validate` CLI with gap analysis output

---

## Phase 5: CLI Subcommands (Core Workflow)

**Status:** In Progress

Core CLI commands for the feature lifecycle: specify, research, plan, implement, validate, ship, retrospective.

| Task ID | Description | Depends On | Status |
|---------|-------------|------------|--------|
| P5.1 | `specify` command: create/update feature with kitty-specs structure | P1.10, P3.1 | Done |
| P5.2 | `research` command: agent-assisted codebase analysis (pre/post-spec) | P1.10 | Planned |
| P5.3 | `plan` command: parse WBS from plan.md, create work packages from tasks | P1.10, P3.5 | Done |
| P5.4 | `implement` command: dispatch agents to WPs in isolated worktrees (orchestrate) | P1.10, P6.1 | Partial |
| P5.5 | `validate` command: run governance checks, report compliance | P1.10, P4.6 | Planned |
| P5.6 | `ship` command: merge validated WP branches, update feature→Shipped | P1.10, P3.2 | Partial |
| P5.7 | `retrospective` command: generate metrics (duration, agent runs, review cycles) | P1.10 | Partial |
| P5.8 | `triage` command: classify incoming work items (bug/feature/idea/task) | P1.10 | Done |
| P5.9 | `queue` command: manage backlog, import items, list queue state | P1.10 | Partial |
| P5.10 | `cycle` command: create/list/show/transition/add-features-to cycles | P1.10, P3.1 | Done |
| P5.11 | `module` command: create/list/show/delete/assign-features/tag modules | P1.10, P3.1 | Done |

**Deliverables:**
- All 11 subcommands functional and tested
- Each command handles --help, --json output, error cases
- CLI integration tests pass

---

## Phase 6: Agent Dispatch & Review Orchestration

**Status:** In Progress

AI agent spawning, lifecycle management, review loops, and result collection.

| Task ID | Description | Depends On | Status |
|---------|-------------|------------|--------|
| P6.1 | Agent stub: mock agent for testing (simulates agent behavior) | P1.10 | Done |
| P6.2 | Agent dispatch RPC/endpoint: POST /agents/dispatch with config and worktree | P2.1, P6.1 | Partial |
| P6.3 | Agent lifecycle tracking: running, completed, failed, timed_out states | P6.2 | Partial |
| P6.4 | Agent result collection: PR URLs, commit SHAs, stdout/stderr, exit codes | P6.3 | Partial |
| P6.5 | Review comment model: severity-classified (critical/major/minor/info) | P1.1 | Partial |
| P6.6 | Review loop orchestration: agent submits, reviewer responds, loop until approved/rejected/max-cycles | P6.4, P6.5 | Partial |
| P6.7 | Review comment evaluation: parse comments, extract action items, route to agent | P6.6 | Planned |
| P6.8 | Review loop CLI: `agileplus implement` with --max-review-cycles and --timeout | P5.4, P6.6 | Partial |
| P6.9 | Max review cycles enforcement: reject further reviews if limit reached | P6.8 | Planned |
| P6.10 | Agent result archival: persist all agent artifacts to object storage (MinIO) | P6.4 | Planned |

**Deliverables:**
- agileplus-domain Agent and ReviewComment models
- Agent dispatch and result collection working end-to-end
- Review loop orchestration with max-cycle enforcement

---

## Phase 7: External Integrations

**Status:** In Progress

Plane.so bidirectional sync, GitHub PR/issue linking, NATS event bus.

| Task ID | Description | Depends On | Status |
|---------|-------------|------------|--------|
| P7.1 | Plane.so API client: authenticate, fetch/create/update issues and sub-issues | P1.10 | Partial |
| P7.2 | Plane.so sync mapping: track sync state with content hashes, last-synced, direction | P1.4, P7.1 | Partial |
| P7.3 | Plane.so bidirectional sync: push features→issues, pull issues→features, conflict detection | P7.2 | Partial |
| P7.4 | Plane.so conflict counting: tally conflicting updates, report in sync status | P7.3 | Planned |
| P7.5 | GitHub API client: authenticate, fetch/create/update PRs and issues | P1.10 | Partial |
| P7.6 | GitHub PR linking: link work packages to PRs, track merge status | P7.5 | Partial |
| P7.7 | GitHub issue linking: link features to issues, track issue state | P7.5 | Partial |
| P7.8 | NATS event bus: publish domain events to NATS subjects, subscribe to external events | P1.2, P1.10 | Partial |
| P7.9 | NATS event stream mapping: map domain events → NATS subjects, NATS messages → domain events | P7.8 | Planned |
| P7.10 | External import: import features/WPs from Plane.so, GitHub, or file | P1.4, P7.1, P7.5 | Partial |

**Deliverables:**
- agileplus-plane, agileplus-github, agileplus-nats crates functional
- Plane.so bidirectional sync working with conflict detection
- GitHub PR/issue linking automated
- NATS event publishing and subscription working

---

## Phase 8: MCP Server (AI Agent Integration)

**Status:** Planned

Python MCP server exposing AgilePlus as tools and resources for AI agents via Model Context Protocol.

| Task ID | Description | Depends On | Status |
|---------|-------------|------------|--------|
| P8.1 | MCP tools for feature management: create, list, update, get features | P1.10 | Planned |
| P8.2 | MCP tools for governance: check status, list evidence requirements, validate policies | P4.6 | Planned |
| P8.3 | MCP tools for status reporting: project status, cycle progress, agent activity | P1.10 | Planned |
| P8.4 | MCP resources for specs, plans, audit trails | P1.10 | Planned |
| P8.5 | MCP prompt templates: pre-built prompts for specify, implement, review workflows | P1.10 | Planned |
| P8.6 | gRPC client in Python: communicate with Rust backend | P1.9 | Partial |
| P8.7 | MCP sampling integration: agent-assisted decision making | P8.1-P8.5 | Planned |
| P8.8 | MCP server startup/shutdown: systemd integration or CLI wrapper | P8.1-P8.7 | Planned |

**Deliverables:**
- agileplus-mcp Python package with 10+ MCP tools
- gRPC client stubs generated and tested
- MCP server runs as standalone service or CLI subcommand

---

## Phase 9: Dashboard & Observability

**Status:** In Progress

Web-based dashboard for visualization, configuration, and evidence gallery. OpenTelemetry observability.

| Task ID | Description | Depends On | Status |
|---------|-------------|------------|--------|
| P9.1 | Dashboard server: Axum templates with HTMX/Alpine.js, seed data | P2.1 | Done |
| P9.2 | Feature status page: show all features with state, WP counts, module assignment | P9.1, P1.1 | Done |
| P9.3 | Cycle progress visualization: timeline view, feature completion %, WP breakdown | P9.1, P3.1 | Done |
| P9.4 | Module organization view: hierarchical module tree with feature counts | P9.1, P3.1 | Done |
| P9.5 | Metrics dashboard: duration per feature, agent runs, review cycles, cycle velocity | P9.1, P1.8 | Done |
| P9.6 | Agent activity monitor: real-time agent status (running/completed/failed), process detection | P9.1, P6.3 | Done |
| P9.7 | Evidence gallery: collapsible log viewer for test results, CI output, security scans | P9.1, P4.3 | Done |
| P9.8 | Clickable timeline: event timeline with links to agents, WPs, git commits, CI/CD jobs | P9.1, P1.3 | Done |
| P9.9 | Settings/config page: API key management, Plane.so/GitHub credentials, NATS broker URL | P9.1 | Done |
| P9.10 | Lightbox/asset viewer: rich preview UI with hover highlights, bottom-right controls | P9.1 | Done |
| P9.11 | Command metrics: OpenTelemetry metrics for CLI command duration, agent runs, review cycles | P1.8, P5.1-P5.11 | Done |
| P9.12 | Service health checks: health status for storage, VCS, external adapters (Plane.so, GitHub) | P2.1, P1.8 | Planned |
| P9.13 | Dashboard evidence persistence: store evidence logs after PR merge for retrospectives | P9.7, P4.3 | Planned |

**Deliverables:**
- agileplus-dashboard crate with all pages functional
- Real-time updates via SSE or polling
- Rich UI with hover-to-expand, gallery preview, timeline links
- Health check endpoints for all adapters

---

## Phase 10: Testing & Quality Infrastructure

**Status:** In Progress

Integration tests, contract tests (gRPC), BDD, property-based tests, benchmarks.

| Task ID | Description | Depends On | Status |
|---------|-------------|------------|--------|
| P10.1 | Integration tests: feature CRUD, state transitions, event sourcing round-trip | P1.4, P3.1 | Partial |
| P10.2 | Contract tests (gRPC): proto message serialization, RPC service stubs | P1.9 | Partial |
| P10.3 | BDD tests: Gherkin features for user workflows (specify, plan, implement, ship) | P5.1-P5.7 | Planned |
| P10.4 | Property-based tests: proptest for event sourcing, state machine invariants | P1.2, P3.1 | Partial |
| P10.5 | Benchmarks: criterion for CLI command performance (specify, plan, validate) | P5.1-P5.7 | Partial |
| P10.6 | Lint enforcement: clippy rules, deadcode detection, security audit (cargo-deny) | — | Done |
| P10.7 | Coverage gates: 80% coverage minimum for domain, CLI, API modules | P10.1-P10.5 | Planned |
| P10.8 | buf lint and breaking checks: proto quality enforcement in CI | P1.9 | Done |
| P10.9 | Snapshot tests: golden files for audit trails, event streams, reports | P1.3, P1.2 | Planned |
| P10.10 | Fuzz testing: fuzzing for proto parsing, event deserialization | P1.9, P1.2 | Planned |

**Deliverables:**
- 3 crates for tests (integration-tests, contract-tests, benchmarks)
- All test suites pass in CI (no failures, no skip/ignore)
- Coverage reports generated and tracked over time

---

## Phase 11: Production Hardening & Deployment

**Status:** Planned

Security hardening, performance optimization, documentation, deployment scripts.

| Task ID | Description | Depends On | Status |
|---------|-------------|------------|--------|
| P11.1 | Credential management: keychain integration (macOS, Linux), file-based fallback | P2.6, P7.1, P7.5 | Partial |
| P11.2 | Secret detection in CI: gitleaks pre-commit hook, GitHub secret scanning | — | Done |
| P11.3 | API rate limiting: per-key rate limits on REST endpoints | P2.1 | Planned |
| P11.4 | Request validation: size limits, input sanitization, schema validation | P2.1 | Planned |
| P11.5 | Audit log archival: move old entries to MinIO, maintain chain integrity | P1.3 | Planned |
| P11.6 | Snapshot retention: periodic snapshots with garbage collection | P1.3 | Planned |
| P11.7 | Performance optimization: query optimization, cache tuning, index analysis | P1.4 | Planned |
| P11.8 | Database backup/restore: schema export, data migration scripts | P1.4 | Planned |
| P11.9 | Deployment playbook: docker-compose.yml or systemd unit files | P2.1, P1.4 | Planned |
| P11.10 | Documentation: API docs (OpenAPI), CLI help system, deployment guide | P5.1-P5.11, P2.1 | Partial |
| P11.11 | Error classification: categorize errors for observability and debugging | P2.1, P1.8 | Planned |
| P11.12 | Graceful shutdown: drain in-flight requests, flush event buffers on stop | P2.1, P1.2 | Planned |

**Deliverables:**
- Secure credential storage (no plaintext in git)
- Rate limiting and request validation working
- Complete deployment documentation and scripts
- API documentation auto-generated from code

---

## Phase 12: Graph Layer & Import Subsystem

**Status:** In Progress

Neo4j-backed graph store for dependency queries and the manifest-driven import subsystem.

| Task ID | Description | Depends On | Status |
|---------|-------------|------------|--------|
| P12G.1 | Graph node/relationship types: FeatureNode, WorkPackageNode, ModuleNode, CycleNode, DeviceNode; edge types DependsOn, BelongsTo, AssignedTo, PartOf | P1.1 | Done |
| P12G.2 | GraphStore Neo4j adapter: Bolt connection, Cypher CRUD for nodes and edges | P12G.1 | Done |
| P12G.3 | Dependency query engine: topological sort, cycle detection, blocked WP path, critical path (Cypher) | P12G.2 | Done |
| P12G.4 | Graph health check: verify Neo4j connectivity, expose ServiceHealth result | P12G.2 | Done |
| P12G.5 | Graph sync hook: write graph nodes/edges on every SQLite state mutation to keep graph consistent | P12G.2, P1.4 | Partial |
| P12G.6 | ImportManifest schema: define JSON/YAML manifest format with field-mapping support | P1.1 | Done |
| P12G.7 | Importer: validate each manifest entry, partial-import semantics, upsert by slug | P12G.6, P1.4 | Done |
| P12G.8 | ImportReport: per-entry outcome tracking (imported/skipped/failed/total), JSON serialization | P12G.7 | Done |
| P12G.9 | `agileplus import` CLI command wiring; idempotency test | P12G.8, P1.10 | Partial |

**Deliverables:**
- `agileplus-graph` crate functional with Neo4j
- `agileplus-import` crate functional with manifest and report
- `agileplus import` CLI command wired end-to-end

---

## Phase 13: P2P Replication

**Status:** Planned

Multi-device state sync via vector clocks, mDNS discovery, and state export/import.

| Task ID | Description | Depends On | Status |
|---------|-------------|------------|--------|
| P13P.1 | VectorClock: assign, increment, merge (component-wise max); attach to all mutable entities | P1.1 | Partial |
| P13P.2 | Device registry: unique device ID (UUID), hostname, address, last_seen; persist to SQLite | P1.4 | Partial |
| P13P.3 | mDNS discovery: advertise `_agileplus._tcp`, discover peers, update device registry | P13P.2 | Planned |
| P13P.4 | State export: portable JSON archive of features/WPs/audit entries/vector clocks; entity filter support | P13P.1 | Partial |
| P13P.5 | State import: merge incoming archive using vector clock comparison; conflict report for concurrent edits | P13P.4 | Partial |
| P13P.6 | Git-adjacent metadata merge: reconcile branch names, commit SHAs, worktree paths via git object model | P13P.5, P1.7 | Partial |
| P13P.7 | Replication session: authenticated handshake (pre-shared key), delta exchange since last sync | P13P.3, P13P.5 | Planned |
| P13P.8 | CLI: `agileplus sync --peer <address>` for manual point-to-point sync | P13P.7, P1.10 | Planned |

**Deliverables:**
- `agileplus-p2p` crate functional with vector clock and export/import
- `agileplus sync --peer` CLI command working for point-to-point sync
- Conflict report output when concurrent edits detected

---

## Phase 14: Cross-Project Reuse & Platform Integration

**Status:** Planned

Extract shared modules, integrate with thegent and other Phenotype projects, consolidate governance docs.

| Task ID | Description | Depends On | Status |
|---------|-------------|------------|--------|
| P12.1 | Identify sharable modules: governance rules, event model, CLI patterns | — | Planned |
| P12.2 | Extract governance rules to shared repo: agileplus-governance crate | P12.1 | Planned |
| P12.3 | Extract event model to shared repo: agileplus-events crate (external) | P12.1 | Planned |
| P12.4 | thegent integration: use AgilePlus for dotfiles/system setup tracking | P12.1 | Planned |
| P12.5 | phenotype-shared integration: link to shared auth, config modules if applicable | P12.1 | Planned |
| P12.6 | Consolidate governance docs: move cross-project docs to thegent | P12.1 | Planned |
| P12.7 | Establish plugin distribution: GitHub releases for agileplus-plugin-* crates | P1.6 | Planned |

**Deliverables:**
- Identified reuse opportunities documented in cross-project section of plans
- Shared modules extracted and published as separate crates
- Cross-project dependency references updated

---

## Phase 13: Future Enhancements (Post-MVP)

**Status:** Not Started

Advanced features for post-MVP refinement and scaling.

| Task ID | Description | Depends On | Status |
|---------|-------------|------------|--------|
| P13.1 | Multi-user RBAC: role-based access control (admin, reviewer, assignee) | P2.6 | Planned |
| P13.2 | Feature templates: reusable specifications for common feature types | P5.1 | Planned |
| P13.3 | Workflow automation: trigger actions on state transitions (e.g., auto-deploy on ship) | P3.2 | Planned |
| P13.4 | Integrations with more external systems: Jira, Azure DevOps, GitLab | P7.1, P7.5 | Planned |
| P13.5 | AI-assisted triage: use LLMs to classify and route work items | P5.8 | Planned |
| P13.6 | Custom governance rules DSL: define policies in YAML or DSL | P4.5 | Planned |
| P13.7 | Retrospective analytics: trend analysis, cycle velocity forecasting | P5.7 | Planned |
| P13.8 | Web-based spec editor: replace kitty-specs file structure with web UI | P9.1 | Planned |

**Deliverables:**
- Documented design proposals for each feature
- Estimated effort and dependencies
- Clear acceptance criteria for future roadmap planning

---

## Dependency Graph (DAG)

```
Phase 1 (Foundation):
  P1.1 → P1.2 → P1.3
  P1.1 → P1.4 → P1.5
  P1.1 → P1.6
  P1.1 → P1.9
  P1.7 (parallel)
  P1.8 (parallel)
  P1.10 (parallel)

Phase 2 (API):
  P1.4 → P2.2
  P1.4 → P2.3
  P1.4 → P2.4
  P1.4 → P2.5
  P2.1 → P2.2, P2.3, P2.4, P2.5, P2.6, P2.7, P2.8, P2.9, P2.10

Phase 3 (State Machine):
  P1.1 → P3.1 → P3.2
  P3.2 → P3.3
  P3.1 → P3.5 → P3.6
  P3.2 → P3.7
  P3.1 → P3.8

Phase 4 (Governance):
  P1.1 → P4.1
  P1.1 → P4.2
  P4.2 → P4.3
  P4.3 → P4.4
  P1.1 → P4.5
  P4.5 → P4.6
  P4.1, P4.6 → P4.7, P4.8
  P4.6 → P4.9
  P4.3 → P4.10

Phase 5 (CLI):
  P1.10, P3.1 → P5.1
  P1.10 → P5.2
  P1.10, P3.5 → P5.3
  P1.10, P6.1 → P5.4
  P1.10, P4.6 → P5.5
  P1.10, P3.2 → P5.6
  P1.10 → P5.7
  P1.10 → P5.8, P5.9, P5.10, P5.11

Phase 6 (Agent):
  P1.10 → P6.1
  P6.1 → P6.2 → P6.3 → P6.4
  P1.1 → P6.5
  P6.4, P6.5 → P6.6
  P6.6 → P6.7
  P5.4, P6.6 → P6.8
  P6.8 → P6.9
  P6.4 → P6.10

Phase 7 (Integrations):
  P1.10 → P7.1 → P7.2 → P7.3
  P7.3 → P7.4
  P1.10 → P7.5 → P7.6, P7.7
  P1.2, P1.10 → P7.8 → P7.9
  P1.4, P7.1, P7.5 → P7.10

Phase 8 (MCP):
  P1.10 → P8.1, P8.2, P8.3, P8.4, P8.5
  P1.9 → P8.6
  P8.1-P8.5 → P8.7
  P8.1-P8.7 → P8.8

Phase 9 (Dashboard):
  P2.1 → P9.1 → P9.2-P9.10
  P1.8, P5.1-P5.11 → P9.11
  P2.1, P1.8 → P9.12
  P9.7, P4.3 → P9.13

Phase 10 (Testing):
  P1.4, P3.1 → P10.1
  P1.9 → P10.2
  P5.1-P5.7 → P10.3
  P1.2, P3.1 → P10.4
  P5.1-P5.7 → P10.5
  P1.9 → P10.8
  P10.1-P10.5 → P10.7
  P1.3, P1.2 → P10.9
  P1.9, P1.2 → P10.10

Phase 11 (Hardening):
  P2.6, P7.1, P7.5 → P11.1
  P2.1 → P11.3, P11.4
  P1.3 → P11.5, P11.6
  P1.4 → P11.7, P11.8
  P2.1, P1.4 → P11.9
  P5.1-P5.11, P2.1 → P11.10
  P2.1, P1.8 → P11.11
  P2.1, P1.2 → P11.12

Phase 12 (Graph + Import):
  P1.1 → P12G.1 → P12G.2 → P12G.3
  P12G.2 → P12G.4
  P12G.2, P1.4 → P12G.5
  P1.1 → P12G.6 → P12G.7 → P12G.8
  P12G.8, P1.10 → P12G.9

Phase 13 (P2P):
  P1.1 → P13P.1
  P1.4 → P13P.2
  P13P.2 → P13P.3
  P13P.1 → P13P.4 → P13P.5
  P13P.5, P1.7 → P13P.6
  P13P.3, P13P.5 → P13P.7
  P13P.7, P1.10 → P13P.8

Phase 14 (Reuse):
  P14.1 → P14.2, P14.3, P14.4, P14.5, P14.6
  P1.6 → P14.7

Phase 15 (Future):
  P2.6 → P15.1
  P5.1 → P15.2
  P3.2 → P15.3
  P7.1, P7.5 → P15.4
  P5.8 → P15.5
  P4.5 → P15.6
  P5.7 → P15.7
  P9.1 → P15.8
```

---

## Implementation Status Summary

### Completed (14 phases, 40+ WPs)
- **Phase 1:** Foundation ✓ (all 10 tasks done)
- **Phase 2:** Core API ✓ (all 10 tasks done)
- **Phase 3:** State Machine ✓ (all 8 tasks done)
- **Phase 5:** CLI (Core Workflow) — 8/11 done (specify, plan, triage, queue, cycle, module done; research, implement (partial), validate, ship, retrospective planned)
- **Phase 6:** Agent Dispatch — 4/10 done (stub, partial dispatch, partial lifecycle, partial result collection; orchestration, max-cycles, archival planned)
- **Phase 7:** Integrations — 5/10 done (partial Plane.so, GitHub, NATS; conflict detection, event mapping, full import planned)
- **Phase 9:** Dashboard — 10/13 done (UI pages, metrics, agent monitor, evidence gallery, timeline, config page all done; health checks, evidence persistence planned)
- **Phase 10:** Testing — 5/10 partial (lint/buf done, integration tests partial; BDD, coverage gates, fuzz planned)
- **Phase 11:** Production — 3/12 partial (credentials, secret detection, documentation started; rate limiting, backup/restore, deployment scripts planned)

### Planned/In-Progress
- **Phase 4:** Governance (policy rules, evidence evaluation) — 3/10 partial
- **Phase 8:** MCP Server — 1/8 partial (gRPC client)
- **Phase 12:** Graph Layer + Import — 7/9 partial (graph nodes/store/queries done; sync hook and CLI partial)
- **Phase 13:** P2P Replication — 4/8 partial (vector clock, device registry, export/import partial; mDNS, session, CLI planned)
- **Phase 14:** Cross-Project Reuse — 0/7 not started
- **Phase 15:** Future Features — 0/8 not started

---

## Critical Path to MVP

**Minimum Viable Product (MVP) Definition:** Feature lifecycle (specify → plan → validate → ship), governance validation, and agent dispatch.

**Critical Path (dependencies only):**
1. P1.1-P1.10 (Foundation) — ~20 tool calls
2. P2.1-P2.10 (API) — ~15 tool calls
3. P3.1-P3.8 (State Machine) — ~10 tool calls
4. P5.1-P5.7 (CLI Core Workflow) — ~15 tool calls
5. P4.1-P4.9 (Governance) — ~12 tool calls
6. P6.2-P6.8 (Agent Dispatch Core) — ~10 tool calls

**Total Critical Path:** ~82 tool calls, ~25-30 min wall-clock time with 3-4 parallel agents.

---

## Success Metrics

1. **All Phase 1-3 tests pass** (foundation, API, state machine)
2. **Core CLI workflow end-to-end:** specify → plan → validate → ship (with governance checks)
3. **Agent dispatch functional:** agents spawn, execute, return results, loop on review
4. **Integration tests pass** (80%+ coverage on domain and CLI)
5. **Dashboard operational:** feature status, metrics, evidence gallery, real agent activity
6. **Governance enforced:** features cannot transition without satisfying evidence requirements
7. **No breaking changes** in proto files without version bump (buf breaking enforced)
8. **Documentation complete:** API docs, CLI help, deployment guide

---

## Phase Rollout Timeline (Aggressive Estimate)

- **Weeks 1-2:** Phase 1-3 complete (foundation, API, state machine) — DONE
- **Weeks 2-3:** Phase 5 complete (CLI core workflow) — IN PROGRESS
- **Weeks 3-4:** Phase 4 complete (governance) — PLANNED
- **Weeks 4-5:** Phase 6 complete (agent dispatch) — IN PROGRESS
- **Weeks 5-6:** Phase 7 complete (integrations) — IN PROGRESS
- **Weeks 6-7:** Phase 9 complete (dashboard refinements) — DONE
- **Weeks 7-8:** Phase 10 complete (testing, coverage gates) — PLANNED
- **Weeks 8-10:** Phase 11 complete (production hardening) — PLANNED
- **Weeks 10+:** Phase 12-13 (reuse, future features) — BACKLOG

**MVP Target:** End of Week 5 (Phase 1-4, 6 core features).

---

## Notes

- All phases follow hexagonal architecture: domain model, ports/adapters, CLI/API/gRPC layers
- Storage is SQLite-first (local), with optional external sync (Plane.so, GitHub)
- All code is Rust (22 crates) except MCP server (Python)
- Tests are mandatory: integration tests, contract tests (gRPC), BDD, property-based tests
- Governance is the enforcement layer: evidence requirements block transitions
- Agent dispatch is asynchronous with lifecycle tracking and review loops
- Dashboard is read-only visualization (CLI is authoring interface)
- Cross-project reuse will be addressed post-MVP

