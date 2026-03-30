# AgilePlus -- Product Requirements Document

**Version:** 2.1 | **Status:** Active | **Date:** 2026-03-27

---

## Product Vision

AgilePlus is a spec-driven development engine that treats specifications as executable contracts. It manages the full lifecycle of software features -- from initial triage through specification, research, planning, implementation by AI agents, validation against governance rules, shipping, and retrospective -- all tracked in an immutable, hash-chained audit log. The system enforces that no code ships without satisfying evidence requirements (tests, CI output, security scans, reviews) bound to governance contracts.

AgilePlus is built as a Rust workspace monorepo (22 crates) following hexagonal architecture, with a CLI as the primary interface, an Axum REST API, a gRPC service layer, a Python MCP server for AI agent integration, SQLite for local-first storage, Neo4j for graph-based dependency queries, NATS JetStream for event messaging, MinIO for artifact storage, and Plane.so for external project management sync. It is designed to be operated entirely by AI agents with human oversight limited to prompt-level direction.

---

## Target Users

- **AI coding agents** (Claude Code, Codex) that receive work packages, implement them in isolated worktrees, and submit PRs through automated review loops
- **Solo developers and small teams** who want spec-driven project management without heavyweight tooling
- **Agent orchestrators** who dispatch and monitor fleets of AI agents across features and work packages
- **Platform engineers** who need auditable, governance-enforced delivery pipelines with evidence collection

---

## Epics

### E1: Feature Lifecycle Management
**Priority**: P0
**Description**: The core domain model. A Feature progresses through an ordered state machine: Created -> Specified -> Researched -> Planned -> Implementing -> Validated -> Shipped -> Retrospected. Each transition is governed by rules, logged in an immutable audit chain, and synced to external systems. Features have slugs, spec hashes, target branches, labels, module ownership, and project association.

#### Stories
- E1.1: Feature CRUD -- Create features with slug, friendly name, spec hash, and target branch; retrieve by ID or slug; list by state or list all
- E1.2: State machine transitions -- Enforce valid transitions (e.g., Planned->Implementing is allowed, Created->Shipped is not); record skipped intermediate states; reject invalid transitions with clear errors
- E1.3: Work package decomposition -- Each feature decomposes into sequenced work packages (WP) with their own state machine (Planned->Doing->Review->Done, with Blocked), file scope, acceptance criteria, agent assignment, PR tracking, worktree path, and base/head commit SHAs
- E1.4: Work package dependency graph -- DAG-based dependency tracking between WPs with cycle detection; dependency types for ordering execution
- E1.5: Module organization -- Hierarchical modules (parent/child) that group features into logical product areas; features assigned to modules via strict ownership; module-scoped filtering
- E1.6: Cycle management -- Time-boxed delivery cycles (Draft->Active->Completed->Archived) that group features; optional module-scoped cycles; cycle progress tracking with feature/WP completion percentages

---

### E2: Governance and Evidence
**Priority**: P0
**Description**: Governance contracts bind to features and define rules that must be satisfied before state transitions are allowed. Rules require specific types of evidence (test results, CI output, review approvals, security scans, lint results, manual attestations) with optional thresholds. Evidence is collected during work package execution and linked to functional requirement IDs. Policy rules define domain-scoped enforcement (security, quality, compliance) with severity levels and auto-enforcement flags.

#### Stories
- E2.1: Governance contract binding -- Create versioned governance contracts for features; each contract contains rules mapping transitions to required evidence types and policy references
- E2.2: Evidence collection -- Record evidence artifacts (test results, CI output, security scans) linked to work packages and FR IDs; store artifact paths and metadata
- E2.3: Policy rules -- Define policy rules with domain, severity (info/warning/error/critical), descriptions, and auto-enforcement flags; evaluate policies on state transitions
- E2.4: Validation command -- CLI `validate` command that checks governance contract satisfaction, collects evidence, and produces pass/fail reports with gap analysis

---

### E3: Immutable Audit Trail and Event Sourcing
**Priority**: P0
**Description**: Every state mutation produces both a domain event and an audit entry, both forming hash chains (SHA-256) for tamper detection. Events are append-only, partitioned by entity type and ID, with sequence numbers. Snapshots materialize current state periodically for fast reads without full event replay. Audit entries link to evidence references and can be archived to object storage (MinIO).

<<<<<<< HEAD
=======
<<<<<<< HEAD
>>>>>>> origin/main
#### Stories
- E3.1: Hash-chained audit log -- Every state change produces an AuditEntry with actor, timestamp, transition description, evidence references, and SHA-256 chain linking to the previous entry
- E3.2: Event sourcing -- Append-only Event stream per entity with hash chain, sequence numbers, typed payloads, and actor attribution
- E3.3: Snapshot materialization -- Periodic snapshots of entity state at known event sequences for fast reconstruction
- E3.4: Audit chain verification -- Verify integrity of audit chains by recomputing and comparing hashes; detect tampering or corruption
- E3.5: Audit archival -- Archive old audit entries to MinIO object storage while maintaining chain integrity

---
<<<<<<< HEAD

### E4: CLI Interface
**Priority**: P0
**Description**: The `agileplus` CLI is the primary user interface, built with clap. It exposes the full feature lifecycle as subcommands that map to the domain model. Commands operate against local SQLite storage and Git worktrees.

#### Stories
- E4.1: `specify` -- Create or update a feature specification; generates kitty-specs directory structure with spec.md and plan.md; computes spec hash
- E4.2: `research` -- Run research phase for a feature (agent-assisted codebase analysis)
- E4.3: `plan` -- Generate phased WBS with DAG dependencies from a specification; parse plan artifacts into work packages
- E4.4: `implement` -- Dispatch AI agents to execute work packages in isolated worktrees; manage agent lifecycle (spawn, monitor, collect results)
- E4.5: `review-loop` -- Orchestrate iterative code review cycles between agents and reviewers until approval or max cycles reached
- E4.6: `validate` -- Run governance checks against collected evidence; produce validation reports
- E4.7: `ship` -- Merge validated work into target branch; update feature state to Shipped
- E4.8: `retrospective` -- Generate retrospective reports with metrics (duration, agent runs, review cycles); compute per-feature and per-WP statistics
- E4.9: `triage` -- Classify and route incoming work items by intent (bug, feature, idea, task) with priority assignment
- E4.10: `scope` -- Define and manage feature scope boundaries
- E4.11: `cycle` -- Create, list, show, transition, add/remove features from cycles
- E4.12: `module` -- Create, list, show, delete, assign features to, tag/untag modules
- E4.13: `queue` -- Manage backlog queue; import items; parse and output queue state
- E4.14: `branch` / `worktree` -- Git branch and worktree management for isolated WP execution
- E4.15: `import` -- Import features and work packages from external sources via manifest
- E4.16: `pr-builder` -- Construct PR descriptions from work package metadata, evidence, and audit trails
- E4.17: `scheduler` -- Schedule and prioritize work package execution order
- E4.18: `dashboard` -- Launch an htmx-driven terminal dashboard showing feature/WP state, agent activity, and audit events

---

### E5: REST API Server
**Priority**: P1
**Description**: An Axum-based HTTP API server exposing the domain model as RESTful endpoints. Includes API key authentication middleware, OpenTelemetry instrumentation middleware, and SSE event streaming. Routes cover features, work packages, cycles, modules, audit, governance, events, backlog, branches, worktrees, and import.

#### Stories
- E5.1: Feature and work package CRUD routes -- Full REST endpoints for features and work packages with JSON request/response
- E5.2: Cycle and module routes -- CRUD and relationship management for cycles and modules
- E5.3: Audit and governance routes -- Query audit trails, governance contracts, and evidence
- E5.4: Event stream (SSE) -- Server-sent events endpoint for real-time domain event streaming
- E5.5: API key authentication middleware -- Authenticate requests via API key with configurable key storage
- E5.6: OpenTelemetry middleware -- Automatic trace/span propagation and metric collection on all requests
- E5.7: Import and branch routes -- Endpoints for importing external data and managing branches

---

### E6: AI Agent Dispatch and Review
**Priority**: P1
**Description**: Agent orchestration for dispatching AI coding agents (Claude Code, Codex) to work packages. Each agent receives a task with prompt, worktree path, and context files. The system manages agent lifecycle (running, completed, failed, timed out), collects results (PR URLs, commits, stdout/stderr, exit codes), and orchestrates review loops with severity-classified comments.

#### Stories
- E6.1: Agent dispatch -- Spawn agents with configurable backend (ClaudeCode/Codex), max review cycles, timeout, and extra arguments; assign agents to work packages in isolated worktrees
- E6.2: Agent lifecycle monitoring -- Track agent status (running, completed, failed, timed out); collect results including PR URLs, commit SHAs, and output
- E6.3: Review loop orchestration -- Iterative review cycles: agent submits code, reviewer provides severity-classified comments (critical/major/minor/informational), agent addresses actionable feedback; loop until approved, rejected, or max cycles reached
- E6.4: Agent stub for testing -- Local agent stub that simulates agent behavior for development and testing

---

### E7: External Integrations
**Priority**: P1
**Description**: Bidirectional sync with external systems. Plane.so integration maps features to issues and work packages to sub-issues with content-hash-based change detection, conflict counting, and configurable sync direction (push/pull/bidirectional). GitHub integration links features to PRs and issues. NATS messaging provides inter-service event transport.

#### Stories
- E7.1: Plane.so sync -- Bidirectional sync of features and work packages with Plane.so issues; track sync mappings with content hashes, last-synced timestamps, sync direction, and conflict counts
- E7.2: GitHub integration -- Link features and work packages to GitHub PRs and issues; sync PR state (open, review, changes requested, approved, merged)
- E7.3: NATS event bus -- Publish domain events to NATS subjects for inter-service communication; subscribe to external events for inbound sync
- E7.4: Import from external sources -- Import features and work packages from Plane.so, GitHub, or manifest files into AgilePlus

---

### E8: MCP Server (Model Context Protocol)
**Priority**: P1
**Description**: A Python MCP server (`agileplus-mcp`) that exposes AgilePlus capabilities as tools, resources, and prompts consumable by AI agents via the Model Context Protocol. Communicates with the Rust backend via gRPC. Provides feature management tools, governance tools, and status reporting tools.

#### Stories
- E8.1: Feature tools -- MCP tools for creating, listing, updating, and querying features
- E8.2: Governance tools -- MCP tools for checking governance status, evidence requirements, and policy compliance
- E8.3: Status tools -- MCP tools for querying project status, cycle progress, and agent activity
- E8.4: Prompt templates -- Pre-built prompts for common agent workflows (specify, implement, review)
- E8.5: Resource exposure -- MCP resources for feature specs, plans, and audit trails
- E8.6: Sampling support -- MCP sampling integration for agent-assisted decision making

---

### E9: Storage and Persistence
**Priority**: P0
**Description**: SQLite-based local-first storage implementing the StoragePort trait. Stores all domain entities (features, work packages, cycles, modules, audit entries, events, snapshots, governance contracts, evidence, policy rules, sync mappings, metrics, backlog items). Includes caching layer for frequently accessed data. Plugin-based storage architecture allows alternative backends.

#### Stories
- E9.1: SQLite adapter -- Full StoragePort implementation with SQLite; schema migrations; all entity CRUD operations
- E9.2: Cache layer -- In-process LRU cache for hot-path reads (feature lookups, WP state queries)
- E9.3: Plugin registry -- Runtime discovery and registration of storage and VCS plugins via trait objects; domain-level plugin registry wrapping core plugin crate
- E9.4: Content storage -- Store and retrieve spec content, plan artifacts, and prompt templates

---

### E10: Graph Storage and Dependency Queries
**Priority**: P1
**Description**: Neo4j-backed graph layer (`agileplus-graph`) that represents features, work packages, modules, cycles, and their relationships as graph nodes and edges. Enables complex dependency queries (e.g., all WPs blocking a feature, all features in a module's subtree) that are impractical in SQLite. Health checks verify Neo4j connectivity.

#### Stories
- E10.1: Graph node types -- Node definitions for Feature, WorkPackage, Module, Cycle, and DeviceNode; each node maps from its corresponding domain entity
- E10.2: Graph relationship types -- Edge definitions for DEPENDS_ON, BELONGS_TO, ASSIGNED_TO, PART_OF, and SYNCS_WITH; bidirectional traversal
- E10.3: Graph store implementation -- Neo4j-backed GraphStore implementing the graph port; Cypher queries for node CRUD and relationship management
- E10.4: Dependency query engine -- Cypher-based queries for topological ordering, cycle detection, blocked WP paths, and critical path analysis
- E10.5: Graph health check -- Verify Neo4j connectivity and reachability; report graph store health via the service health system

---

### E11: P2P Replication
**Priority**: P2
**Description**: Peer-to-peer state replication (`agileplus-p2p`) for multi-device sync without a central server. Uses vector clocks for concurrent edit detection and mDNS for device discovery. Enables eventually-consistent replication of features, work packages, and audit trails across devices.

#### Stories
- E11.1: Device discovery -- mDNS-based or static-config device discovery; device registry with unique device IDs and network addresses
- E11.2: Vector clock conflict detection -- Assign vector clock entries to all mutable state; detect concurrent edits; merge or flag conflicts per entity type
- E11.3: State export and import -- Portable serialization format for exporting/importing full or partial domain state; support partial sync by entity filter
- E11.4: Git-adjacent metadata merge -- Merge git-related metadata (branch names, commit SHAs) across devices using git object model semantics
- E11.5: Replication session -- Authenticated replication handshake between devices; exchange state deltas since last sync timestamp

---

### E12: Observability and Telemetry
**Priority**: P2
**Description**: OpenTelemetry-based observability with traces, metrics, and structured logging. Metrics capture command execution telemetry (duration, agent runs, review cycles per feature). OTLP export to external collectors. Service health monitoring for all adapters.

#### Stories
- E12.1: Command metrics -- Record per-command execution metrics (duration_ms, agent_runs, review_cycles) with optional feature association
- E12.2: OpenTelemetry integration -- Trace propagation, span creation, and metric export via OTLP
- E12.3: Service health checks -- Health status reporting for storage, VCS, graph, and external service adapters
- E12.4: Structured logging -- Tracing-subscriber-based structured logging with configurable verbosity
- E12.5: Dashboard -- Web-based dashboard (Axum + HTMX) for visualizing feature status, cycle progress, module organization, and metrics; seed data for development
=======

### E4: CLI Interface
**Priority**: P0
**Description**: The `agileplus` CLI is the primary user interface, built with clap. It exposes the full feature lifecycle as subcommands that map to the domain model. Commands operate against local SQLite storage and Git worktrees.

#### Stories
- E4.1: `specify` -- Create or update a feature specification; generates kitty-specs directory structure with spec.md and plan.md; computes spec hash
- E4.2: `research` -- Run research phase for a feature (agent-assisted codebase analysis)
- E4.3: `plan` -- Generate phased WBS with DAG dependencies from a specification; parse plan artifacts into work packages
- E4.4: `implement` -- Dispatch AI agents to execute work packages in isolated worktrees; manage agent lifecycle (spawn, monitor, collect results)
- E4.5: `review-loop` -- Orchestrate iterative code review cycles between agents and reviewers until approval or max cycles reached
- E4.6: `validate` -- Run governance checks against collected evidence; produce validation reports
- E4.7: `ship` -- Merge validated work into target branch; update feature state to Shipped
- E4.8: `retrospective` -- Generate retrospective reports with metrics (duration, agent runs, review cycles); compute per-feature and per-WP statistics
- E4.9: `triage` -- Classify and route incoming work items by intent (bug, feature, idea, task) with priority assignment
- E4.10: `scope` -- Define and manage feature scope boundaries
- E4.11: `cycle` -- Create, list, show, transition, add/remove features from cycles
- E4.12: `module` -- Create, list, show, delete, assign features to, tag/untag modules
- E4.13: `queue` -- Manage backlog queue; import items; parse and output queue state
- E4.14: `branch` / `worktree` -- Git branch and worktree management for isolated WP execution
- E4.15: `import` -- Import features and work packages from external sources via manifest
- E4.16: `pr-builder` -- Construct PR descriptions from work package metadata, evidence, and audit trails
- E4.17: `scheduler` -- Schedule and prioritize work package execution order
- E4.18: `dashboard` -- Launch an htmx-driven terminal dashboard showing feature/WP state, agent activity, and audit events

---

### E5: REST API Server
**Priority**: P1
**Description**: An Axum-based HTTP API server exposing the domain model as RESTful endpoints. Includes API key authentication middleware, OpenTelemetry instrumentation middleware, and SSE event streaming. Routes cover features, work packages, cycles, modules, audit, governance, events, backlog, branches, worktrees, and import.

#### Stories
- E5.1: Feature and work package CRUD routes -- Full REST endpoints for features and work packages with JSON request/response
- E5.2: Cycle and module routes -- CRUD and relationship management for cycles and modules
- E5.3: Audit and governance routes -- Query audit trails, governance contracts, and evidence
- E5.4: Event stream (SSE) -- Server-sent events endpoint for real-time domain event streaming
- E5.5: API key authentication middleware -- Authenticate requests via API key with configurable key storage
- E5.6: OpenTelemetry middleware -- Automatic trace/span propagation and metric collection on all requests
- E5.7: Import and branch routes -- Endpoints for importing external data and managing branches

---

### E6: AI Agent Dispatch and Review
**Priority**: P1
**Description**: Agent orchestration for dispatching AI coding agents (Claude Code, Codex) to work packages. Each agent receives a task with prompt, worktree path, and context files. The system manages agent lifecycle (running, completed, failed, timed out), collects results (PR URLs, commits, stdout/stderr, exit codes), and orchestrates review loops with severity-classified comments.

#### Stories
- E6.1: Agent dispatch -- Spawn agents with configurable backend (ClaudeCode/Codex), max review cycles, timeout, and extra arguments; assign agents to work packages in isolated worktrees
- E6.2: Agent lifecycle monitoring -- Track agent status (running, completed, failed, timed out); collect results including PR URLs, commit SHAs, and output
- E6.3: Review loop orchestration -- Iterative review cycles: agent submits code, reviewer provides severity-classified comments (critical/major/minor/informational), agent addresses actionable feedback; loop until approved, rejected, or max cycles reached
- E6.4: Agent stub for testing -- Local agent stub that simulates agent behavior for development and testing

---

### E7: External Integrations
**Priority**: P1
**Description**: Bidirectional sync with external systems. Plane.so integration maps features to issues and work packages to sub-issues with content-hash-based change detection, conflict counting, and configurable sync direction (push/pull/bidirectional). GitHub integration links features to PRs and issues. NATS messaging provides inter-service event transport.

#### Stories
- E7.1: Plane.so sync -- Bidirectional sync of features and work packages with Plane.so issues; track sync mappings with content hashes, last-synced timestamps, sync direction, and conflict counts
- E7.2: GitHub integration -- Link features and work packages to GitHub PRs and issues; sync PR state (open, review, changes requested, approved, merged)
- E7.3: NATS event bus -- Publish domain events to NATS subjects for inter-service communication; subscribe to external events for inbound sync
- E7.4: Import from external sources -- Import features and work packages from Plane.so, GitHub, or manifest files into AgilePlus

---

### E8: MCP Server (Model Context Protocol)
**Priority**: P1
**Description**: A Python MCP server (`agileplus-mcp`) that exposes AgilePlus capabilities as tools, resources, and prompts consumable by AI agents via the Model Context Protocol. Communicates with the Rust backend via gRPC. Provides feature management tools, governance tools, and status reporting tools.

#### Stories
- E8.1: Feature tools -- MCP tools for creating, listing, updating, and querying features
- E8.2: Governance tools -- MCP tools for checking governance status, evidence requirements, and policy compliance
- E8.3: Status tools -- MCP tools for querying project status, cycle progress, and agent activity
- E8.4: Prompt templates -- Pre-built prompts for common agent workflows (specify, implement, review)
- E8.5: Resource exposure -- MCP resources for feature specs, plans, and audit trails
- E8.6: Sampling support -- MCP sampling integration for agent-assisted decision making

---

### E9: Storage and Persistence
**Priority**: P0
**Description**: SQLite-based local-first storage implementing the StoragePort trait. Stores all domain entities (features, work packages, cycles, modules, audit entries, events, snapshots, governance contracts, evidence, policy rules, sync mappings, metrics, backlog items). Includes caching layer for frequently accessed data. Plugin-based storage architecture allows alternative backends.

#### Stories
- E9.1: SQLite adapter -- Full StoragePort implementation with SQLite; schema migrations; all entity CRUD operations
- E9.2: Cache layer -- In-process LRU cache for hot-path reads (feature lookups, WP state queries)
- E9.3: Plugin registry -- Runtime discovery and registration of storage and VCS plugins via trait objects; domain-level plugin registry wrapping core plugin crate
- E9.4: Content storage -- Store and retrieve spec content, plan artifacts, and prompt templates

---

### E10: Graph Storage and Dependency Queries
**Priority**: P1
**Description**: Neo4j-backed graph layer (`agileplus-graph`) that represents features, work packages, modules, cycles, and their relationships as graph nodes and edges. Enables complex dependency queries (e.g., all WPs blocking a feature, all features in a module's subtree) that are impractical in SQLite. Health checks verify Neo4j connectivity.

#### Stories
- E10.1: Graph node types -- Node definitions for Feature, WorkPackage, Module, Cycle, and DeviceNode; each node maps from its corresponding domain entity
- E10.2: Graph relationship types -- Edge definitions for DEPENDS_ON, BELONGS_TO, ASSIGNED_TO, PART_OF, and SYNCS_WITH; bidirectional traversal
- E10.3: Graph store implementation -- Neo4j-backed GraphStore implementing the graph port; Cypher queries for node CRUD and relationship management
- E10.4: Dependency query engine -- Cypher-based queries for topological ordering, cycle detection, blocked WP paths, and critical path analysis
- E10.5: Graph health check -- Verify Neo4j connectivity and reachability; report graph store health via the service health system

---

### E11: P2P Replication
**Priority**: P2
**Description**: Peer-to-peer state replication (`agileplus-p2p`) for multi-device sync without a central server. Uses vector clocks for concurrent edit detection and mDNS for device discovery. Enables eventually-consistent replication of features, work packages, and audit trails across devices.

#### Stories
- E11.1: Device discovery -- mDNS-based or static-config device discovery; device registry with unique device IDs and network addresses
- E11.2: Vector clock conflict detection -- Assign vector clock entries to all mutable state; detect concurrent edits; merge or flag conflicts per entity type
- E11.3: State export and import -- Portable serialization format for exporting/importing full or partial domain state; support partial sync by entity filter
- E11.4: Git-adjacent metadata merge -- Merge git-related metadata (branch names, commit SHAs) across devices using git object model semantics
- E11.5: Replication session -- Authenticated replication handshake between devices; exchange state deltas since last sync timestamp

---

### E12: Observability and Telemetry
**Priority**: P2
**Description**: OpenTelemetry-based observability with traces, metrics, and structured logging. Metrics capture command execution telemetry (duration, agent runs, review cycles per feature). OTLP export to external collectors. Service health monitoring for all adapters.

#### Stories
- E12.1: Command metrics -- Record per-command execution metrics (duration_ms, agent_runs, review_cycles) with optional feature association
- E12.2: OpenTelemetry integration -- Trace propagation, span creation, and metric export via OTLP
- E12.3: Service health checks -- Health status reporting for storage, VCS, graph, and external service adapters
- E12.4: Structured logging -- Tracing-subscriber-based structured logging with configurable verbosity
- E12.5: Dashboard -- Web-based dashboard (Axum + HTMX) for visualizing feature status, cycle progress, module organization, and metrics; seed data for development
=======
### E5.1: Typed Forward-Only FSM

As a service developer, I want a `StateMachine<S, C>` where `S` is the state enum and `C` is the context type so workflow state is enforced with forward-only transitions and domain-specific guard callbacks operating over typed context.

**Acceptance criteria**:
- `StateMachine::new(initial_state: S, initial_context: C)` constructs a machine in the given state with an owned context.
- `Transition::new(from, to)` creates a transition registration between two states.
- `add_transition(transition)` registers a `Transition<S, C>` with the machine.
- `transition_to(target_state)` returns `Ok(())` or `Err(StateMachineError::InvalidTransition)`.
- Transitions are matched by `(from == current, to == target)` equality; no matching transition returns `InvalidTransition { from, to }`.
- All public types (`S`, `C`) are bounded by `Clone + PartialEq + Debug + Serialize + DeserializeOwned`.
- All internal state is behind `Arc<RwLock<_>>` for `Send + Sync` compatibility.

### E5.2: Guard Callbacks and Action Hooks

As a service developer, I want guard conditions that gate transitions and action hooks that run on successful transitions so domain logic is decoupled from state-machine plumbing.

**Acceptance criteria**:
- `Transition::with_guard(Fn(&C) -> bool + Send + Sync + 'static)` attaches a guard closure; evaluated before the transition is applied.
- `Transition::with_action(Fn(&mut C) + Send + Sync + 'static)` attaches an action closure; executed after guard passes but before the state is updated.
- A failing guard returns `StateMachineError::GuardConditionFailed { reason }` and the machine state is unchanged.
- A transition with no guard always succeeds (permissive by default).
- `can_transition_to(&S)` returns `Ok(true)` if a matching transition exists and its guard (if any) returns `true`; `Ok(false)` otherwise.

### E5.3: Transition History

As an auditor, I want an immutable record of every state the machine has visited so transitions can be replayed and inspected post-hoc.

**Acceptance criteria**:
- `history()` returns `Result<Vec<S>>` containing every state in visitation order, starting with the initial state.
- History is append-only; each successful `transition_to` appends the new state.
- History is persisted behind `Arc<RwLock<Vec<S>>>` for concurrent read access.
- A machine with N successful transitions has `history().len() == N + 1`.

### E5.4: Skip-State Configuration

As a platform operator, I want to declare specific non-sequential state advances so emergency or out-of-band transitions bypass the normal forward path.

**Acceptance criteria**:
- `StateMachineConfig` (or equivalent) holds a `skip_states: Vec<(S, S)>` list of allowed non-sequential transitions.
- A transition that jumps forward (target ordinal > current + 1) is rejected unless explicitly listed in `skip_states`.
- A skip-state entry `(from, to)` is validated at registration: `to` ordinal must be greater than `from` ordinal.
- Skip-state transitions still require guard evaluation and trigger action hooks identically to sequential transitions.

---

## E6: Metrics & Observability

### E6.1: Metrics Collection Interface

As a platform operator, I want a standardized metrics interface so all phenotype-infrakit crates can report operational telemetry without coupling to a specific monitoring backend.

**Acceptance criteria**:
- `MetricsCollector` trait with methods: `counter(name, value)`, `gauge(name, value)`, `histogram(name, value, duration)`
- All public types implement `Send + Sync` for concurrent access
- Metrics are tagged with crate name and version for easy filtering
- Default no-op implementation available for crates that don't need metrics

### E6.2: Health Check Abstraction

As a service operator, I want a unified health check interface so I can verify the operational status of all phenotype-infrakit components through a single endpoint.

**Acceptance criteria**:
- `HealthCheck` trait with method: `check() -> Result<HealthStatus, HealthError>`
- `HealthStatus` includes `healthy: bool`, `message: String`, `details: HashMap<String, String>`
- Each crate implements health checks for its core functionality (e.g., event store connectivity, cache connectivity)
- Health check results are aggregated and exposed via a unified endpoint
>>>>>>> origin/main
>>>>>>> origin/main

---

## Non-Functional Requirements

- **Performance**: CLI commands complete in <500ms for local operations; SQLite queries optimized with indexes; snapshot materialization prevents full event replay; LRU cache layer for hot-path reads
- **Security**: API key authentication on all API endpoints; credential management with keychain integration (macOS Keychain, system keyring) and file-based fallback; secret detection in CI; no plaintext credential storage
- **Integrity**: SHA-256 hash chains on both audit entries and domain events ensure tamper detection; chain verification is available as a CLI and API operation
- **Extensibility**: Hexagonal architecture with ports (StoragePort, VcsPort, AgentPort, ReviewPort, ObservabilityPort, ContentStoragePort) and adapters; plugin registry for runtime backend swapping
- **Local-First**: SQLite storage means no external database dependency; all state is local; sync with external systems (Plane.so, GitHub) is optional and bidirectional
- **Testability**: Workspace includes integration tests, contract tests (gRPC), BDD tests, property-based tests (proptest), and benchmarks (criterion); in-memory storage and VCS stubs for testing
- **Proto Compatibility**: buf lint and buf breaking enforce proto quality and backward compatibility; version bumps required for breaking changes

---

## Out of Scope

- Multi-user authentication and authorization (current model is single-user or API-key-based)
- Cloud-hosted SaaS deployment (AgilePlus is designed as a local-first tool)
- Web-based feature editing UI (the dashboard is read-only visualization; the CLI is the authoring interface)
- Support for non-Git version control systems
- Real-time collaborative editing of specifications
- Billing, subscription, or payment management
- Mobile clients
