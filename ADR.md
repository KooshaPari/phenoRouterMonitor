# AgilePlus -- Architecture Decision Records

**Version:** 1.2 | **Status:** Active | **Updated:** 2026-03-27

---

## ADR-001: Rust Workspace Monorepo with 22 Crates
**Date**: 2026-03-25
**Status**: Accepted
**Context**: AgilePlus requires a spec-driven development engine with distinct subsystems (domain logic, CLI, API, gRPC, storage, git, telemetry, events, caching, graph, p2p, sync, dashboard, and more). Each subsystem has different dependency requirements, compilation profiles, and test isolation needs. A single-crate approach would create circular dependencies and prevent independent crate versioning. A polyrepo approach would fragment the tightly coupled domain model.
**Decision**: Use a Rust Cargo workspace with `resolver = "3"` and 22 member crates, each scoped to a single architectural concern (e.g., `agileplus-domain`, `agileplus-cli`, `agileplus-api`, `agileplus-grpc`, `agileplus-sqlite`, `agileplus-git`, `agileplus-p2p`). Workspace-level `[workspace.dependencies]` pins all shared dependency versions.
**Consequences**: Independent crate compilation and caching; enforced module boundaries by Rust's visibility rules; workspace-wide dependency deduplication via shared lockfile; crate-level feature flags (e.g., `keychain`, `plugins` on `agileplus-domain`); longer cold build times on first compile; contributors must understand the crate graph before adding cross-crate dependencies.
**Alternatives Considered**: Single-crate monolith (rejected: circular deps, no isolation); polyrepo per subsystem (rejected: excessive coordination overhead for a tightly-coupled domain model); feature-flag-gated modules inside one crate (rejected: no compilation isolation, no independent versioning).

---

## ADR-002: Hexagonal Architecture with Port/Adapter Pattern
**Date**: 2026-03-25
**Status**: Accepted
**Context**: AgilePlus must support swappable backends for storage (SQLite today, potentially Postgres tomorrow), VCS (git), agent dispatch (Claude Code, Codex), review, observability, and content storage. Business rules in the domain must remain independent of any specific I/O technology.
**Decision**: Apply hexagonal (ports and adapters) architecture. The `agileplus-domain` crate defines port traits (`StoragePort`, `VcsPort`, `AgentPort`, `ReviewPort`, `ObservabilityPort`, `ContentStoragePort`). Each external subsystem crate (`agileplus-sqlite`, `agileplus-git`, `agileplus-github`, `agileplus-telemetry`) provides one or more adapter implementations of those traits. The CLI and API layers depend only on domain types and port traits, never on concrete adapters.
**Consequences**: Domain logic is testable with in-memory stubs; adapters can be swapped at runtime via the plugin registry without recompiling the domain; new integrations (e.g., a PostgreSQL adapter) require only a new crate implementing `StoragePort`; adds indirection that increases code volume and requires discipline to prevent adapter logic leaking into the domain.
**Alternatives Considered**: Service-layer pattern with direct database calls (rejected: tight coupling, prevents backend swapping); trait objects via `dyn StoragePort` throughout (accepted as the mechanism); dynamic dispatch at compile time via generics (considered for hot-path performance but increases monomorphization code size).

---

## ADR-003: SQLite as Local-First Storage with Optional External Sync
**Date**: 2026-03-25
**Status**: Accepted
**Context**: AgilePlus targets solo developers and AI agents running in isolated worktrees. Requiring a running Postgres or MySQL instance would create friction for local setups, CI environments, and offline operation. At the same time, teams need optional bidirectional sync with external project management systems (Plane.so, GitHub).
**Decision**: Use SQLite (via `rusqlite` with the `bundled` feature for zero-dependency deployment) as the sole persistence layer. The `agileplus-sqlite` crate implements `StoragePort` against a local file at `.agileplus/`. Sync with external systems (Plane.so, GitHub) is an optional, explicit operation tracked via a sync-mapping table with content-hash-based change detection.
**Consequences**: Zero-dependency installation; full offline operation; no connection management overhead; SQLite's serialized write model limits concurrent write throughput (acceptable for the target workload of a single developer/agent fleet); sync conflicts between local and remote must be explicitly resolved; `rusqlite` with `bundled` increases binary size by ~3MB.
**Alternatives Considered**: PostgreSQL (rejected: requires a running server, not local-first); SurrealDB (rejected: immature embedding story at decision time); pure in-memory storage (rejected: no persistence across restarts); file-per-entity JSON (rejected: no query capability, difficult migrations).

---

## ADR-004: SHA-256 Hash-Chained Immutable Audit Log and Event Store
**Date**: 2026-03-25
**Status**: Accepted
**Context**: AgilePlus enforces governance contracts that require verifiable proof that evidence was collected, transitions were approved, and no records were tampered with after the fact. Without cryptographic integrity, audit trails are unreliable as governance artifacts.
**Decision**: Every state mutation produces two records: a domain `Event` and an `AuditEntry`. Both are append-only and form independent SHA-256 hash chains -- each entry stores the hash of the previous entry. Chain integrity can be verified by the `validate` command and the audit API route. Audit entries include actor, timestamp, transition description, evidence references, and the chain link hash. Events include typed payloads, sequence numbers, and actor attribution.
**Consequences**: Tamper detection for any audit entry or event; full event-sourcing capability (current state can be reconstructed from event stream); append-only writes are fast and never block on updates; snapshots (`agileplus-events` snapshot materialization) prevent full-replay cost; storage grows monotonically and requires periodic archival to MinIO; hash verification is a sequential O(n) scan of the chain.
**Alternatives Considered**: Mutable audit log with soft-delete (rejected: no tamper detection); external blockchain/ledger (rejected: operational complexity, latency, no offline support); append-only log without hashing (rejected: no integrity guarantee); Merkle tree per entity (considered for parallel verification, deferred to a future ADR).

---

## ADR-005: gRPC Service Layer with Tonic + Protobuf for MCP and Inter-Service Communication
**Date**: 2026-03-25
**Status**: Accepted
**Context**: The Python MCP server (`agileplus-mcp`) needs a stable, typed contract to call into the Rust backend. HTTP REST is an option but requires manual serialization and lacks bidirectional streaming. The system also needs inter-service communication between the API server and background workers.
**Decision**: Define all service contracts in `.proto` files under `proto/agileplus/v1/` using Buf for linting and breaking-change detection. Generate Rust server/client code via `tonic-build` + `prost`. The `agileplus-grpc` crate exposes gRPC service implementations. The Python MCP server uses the generated Python stubs (via `buf generate`) to call the Rust backend.
**Consequences**: Strongly-typed, versioned contracts between Python and Rust; Buf linting (`STANDARD` ruleset) catches API regressions before they reach production; bidirectional streaming available for SSE-like use cases; Proto schemas serve as the canonical API documentation; adds a code-generation step to the build pipeline; binary wire format is less human-readable than JSON for debugging.
**Alternatives Considered**: JSON REST API from Python to Rust (rejected: no schema enforcement, manual serialization, no streaming); GraphQL (rejected: overfitting for internal service communication); NATS request/reply (considered for internal comms, used alongside gRPC for event publishing but not for typed RPC calls); Cap'n Proto (rejected: smaller ecosystem, less tooling).

---

## ADR-006: NATS JetStream as the Event Bus
**Date**: 2026-03-25
**Status**: Accepted
**Context**: Domain events need to be delivered to multiple consumers (API SSE streams, sync workers, telemetry collectors) without tight coupling between producers and consumers. The event bus must support persistent delivery and replay for consumers that fall behind.
**Decision**: Run a native `nats-server` with JetStream enabled (store dir `.agileplus/nats-data`, port 4222, HTTP monitoring port 8222). The `agileplus-nats` crate publishes domain events to NATS subjects for inter-service communication and subscribes to inbound events from external systems. `process-compose` manages the NATS process lifecycle with a readiness probe against the `/healthz` endpoint.
**Consequences**: Decoupled event producers and consumers; JetStream persistence survives process restarts; NATS has low resource overhead suitable for local dev; native NATS server runs without Docker; adds an operational process that must be running for event delivery; NATS subjects must be documented and treated as a public API contract.
**Alternatives Considered**: Redis Streams (rejected: requires separate Redis process, more overhead); Kafka (rejected: heavy operational footprint, inappropriate for local-first workloads); in-process event bus only (rejected: no cross-process delivery, no persistence); PostgreSQL LISTEN/NOTIFY (rejected: couples event bus to database).

---

## ADR-007: Plugin Architecture via External Git-Sourced Crates
**Date**: 2026-03-25
**Status**: Accepted
**Context**: Storage backends and VCS adapters need to be extensible without modifying the core workspace. Third parties or future platform engineers should be able to ship a plugin crate that satisfies a port trait and be registered at runtime without a full recompile of the monorepo.
**Decision**: Define plugin trait contracts in three separate Git repositories (`agileplus-plugin-core`, `agileplus-plugin-git`, `agileplus-plugin-sqlite`) referenced in `[workspace.dependencies]` as Git sources. The `agileplus-domain` crate's `plugins` feature gate enables the `agileplus-plugin-core` dependency. A `PluginRegistry` in `agileplus-domain/src/plugins/` discovers and registers plugins at startup via trait objects.
**Consequences**: Plugin trait contracts are versioned independently from the main workspace; breaking changes to plugin interfaces can be detected via semver without affecting core; plugins compile as regular Rust crates (no WASM required today); Git-sourced dependencies add a network fetch step to first-time builds and depend on remote availability; no ABI stability (plugins must be compiled with the same Rust toolchain).
**Alternatives Considered**: Extism/WASM plugin system (considered for ABI stability; deferred -- WASM plugins are a post-MVP enhancement); dlopen dynamic loading (rejected: unsound in Rust without significant unsafe code); feature-flagged modules in the workspace (rejected: couples plugin code to core workspace, prevents independent versioning).

---

## ADR-008: Python MCP Server for AI Agent Integration
**Date**: 2026-03-25
**Status**: Accepted
**Context**: AI coding agents (Claude Code, Codex) consume capabilities via the Model Context Protocol (MCP). The Rust backend does not expose an MCP-native interface. A Python MCP server bridges the protocol boundary, exposing AgilePlus tools, resources, and prompts as first-class MCP constructs.
**Decision**: Implement `agileplus-mcp` as a Python package that acts as an MCP server. It communicates with the Rust backend exclusively over gRPC (using generated stubs from the Buf-managed proto files). It exposes MCP tools for feature management, governance checks, and status reporting; MCP resources for spec and audit data; and MCP prompt templates for common agent workflows. Containerized via `Dockerfile.python` for reproducible deployment.
**Consequences**: AI agents get a first-class, protocol-native interface without any Rust knowledge; Python's MCP SDK ecosystem is richer than Rust's at decision time; gRPC ensures typed, versioned contracts between Python and Rust; two-language boundary adds operational complexity (two runtimes, two deployment units); the Python service must be running for agent-facing tools to work.
**Alternatives Considered**: Rust MCP server (rejected: Rust MCP SDK was immature at decision time); exposing REST endpoints and wrapping with a thin Python MCP proxy (rejected: two serialization layers, loses type safety); embedding Python in Rust via PyO3 (rejected: complex build, harder to iterate on Python code independently).

---

## ADR-009: OpenTelemetry for Observability with OTLP Export
**Date**: 2026-03-25
**Status**: Accepted
**Context**: AgilePlus dispatches AI agents across multiple worktrees and processes. Debugging failures, measuring command latency, and understanding agent lifecycle events requires distributed tracing and metrics. Vendor-specific SDKs would lock the platform to a single observability backend.
**Decision**: Use the OpenTelemetry Rust SDK (`opentelemetry`, `opentelemetry-otlp`, `opentelemetry_sdk`) with OTLP export. The `agileplus-telemetry` crate implements the `ObservabilityPort` and provides trace propagation, span creation, and metric collection for all commands. Metrics record per-command execution data (duration_ms, agent_runs, review_cycles). `tracing-opentelemetry` bridges the `tracing` subscriber ecosystem to OTel spans.
**Consequences**: Backend-agnostic observability -- any OTLP-compatible collector (Jaeger, Grafana Tempo, Honeycomb) works; `tracing` macros throughout the codebase automatically produce structured logs and spans; OTLP export is optional (no-op if no collector is configured); adds ~5 transitive dependencies; the telemetry crate is a required workspace member even when observability is not actively used.
**Alternatives Considered**: Prometheus-only metrics (rejected: no distributed tracing, no log correlation); Datadog agent SDK (rejected: vendor lock-in, not OSS); `log` crate without OTel (rejected: no trace propagation across async boundaries); custom structured logging only (rejected: no metrics, no trace correlation).

---

## ADR-010: process-compose for Local Dev Orchestration (OrbStack + Native Services)
**Date**: 2026-03-25
**Status**: Accepted
**Context**: AgilePlus requires multiple services to run concurrently for local development: NATS, Neo4j, MinIO, PostgreSQL (via OrbStack container), DragonflyDB (Redis-compatible, via OrbStack container), and the Rust API/gRPC servers. Docker Compose adds container overhead for services that can run natively. A TUI-friendly process supervisor improves developer experience.
**Decision**: Use `process-compose` as the local dev orchestrator. OrbStack containers manage Dragonfly and PostgreSQL (started via `scripts/orb-up.sh`). NATS, Neo4j, and MinIO run as native processes. Each service has a readiness probe and logs to `.agileplus/logs/`. Services declare `depends_on` conditions to enforce startup ordering.
**Consequences**: Developers see all service logs in a unified TUI without Docker overhead for native services; readiness probes prevent false-start ordering bugs; OrbStack provides near-native container performance on macOS for the few services that need containerization; `process-compose` is a project-specific tool (requires installation); the mixed native/container approach is more complex than pure Docker Compose.
**Alternatives Considered**: Pure Docker Compose (rejected: container overhead for services like NATS and Neo4j that run well natively; slower cold start); `overmind`/`foreman` Procfile (rejected: no readiness probes, no dependency ordering); manual `tmux` sessions (rejected: no programmatic orchestration, hard to script); Tilt (rejected: Kubernetes-focused, overkill for local dev).

---

## ADR-011: Credentials Management via OS Keychain with File-Based Fallback
**Date**: 2026-03-25
**Status**: Accepted
**Context**: AgilePlus stores credentials for external services (Plane.so API keys, GitHub tokens, MinIO secrets). These must not be stored in plaintext on disk or committed to git. Different deployment environments (macOS developer machine, Linux CI container) have different secure storage capabilities.
**Decision**: The `agileplus-domain` crate exposes a `credentials` module with a `keychain` feature gate. When compiled with `--features keychain`, credentials are stored and retrieved from the OS keychain via the `keyring` crate (macOS Keychain, Linux Secret Service, Windows Credential Manager). When the feature is disabled or keychain access fails, credentials fall back to a file-based store at `.agileplus/credentials` with `zeroize` applied to in-memory credential values on drop.
**Consequences**: Developer machines get native OS keychain integration; CI containers get file-based credentials without keychain daemons; `zeroize` prevents credential values from lingering in memory after use; the optional feature adds the `keyring` crate dependency only when needed; file-based fallback is less secure than OS keychain and requires filesystem permission controls.
**Alternatives Considered**: Environment variables only (rejected: no persistence, leaks in process listings); Vault (rejected: requires running a Vault server, heavy for local-first workloads); plaintext `.env` file (rejected: too easy to commit accidentally); SOPS-encrypted secrets file (considered for CI; deferred as the file-based fallback can be wrapped with SOPS externally).

---

## ADR-012: P2P Replication with Vector Clocks for Multi-Device Sync
**Date**: 2026-03-25
**Status**: Accepted
**Context**: Solo developers who work across multiple machines (laptop, desktop, CI runner) need their AgilePlus state (features, work packages, audit trails) synchronized without a central server. A client-server sync architecture would require always-on infrastructure and break the local-first principle.
**Decision**: Implement peer-to-peer replication in the `agileplus-p2p` crate. Device discovery uses mDNS or static configuration. State replication uses vector clocks (`vector_clock.rs`) to detect and merge concurrent edits. The crate includes `import.rs` and `export.rs` for portable state serialization and `git_merge.rs` for merging git-adjacent metadata. Replication is eventually consistent.
**Consequences**: No central server required; multiple devices can work offline and sync when reconnected; vector clocks detect concurrent edits without requiring synchronized clocks; conflict resolution must be explicitly defined per entity type (last-write-wins, merge, or manual); P2P discovery via mDNS may not work across network segments; the crate is in an early state and not yet enabled in the default feature set.
**Alternatives Considered**: Central sync server (rejected: breaks local-first principle, adds infrastructure dependency); Git-based state sync (considered as a simpler alternative; partially implemented in `git_merge.rs` for git-adjacent data, but insufficient for the full domain model); CRDTs (considered for convergent data structures; deferred -- vector clocks are simpler and sufficient for the current conflict model); rsync-style file transfer (rejected: no semantic merge capability).

---

## ADR-013: Neo4j for Graph-Based Dependency and Relationship Queries
**Date**: 2026-03-27
**Status**: Accepted
**Context**: Work package dependency relationships form a DAG that must be queried for topological ordering, cycle detection, blocked WP identification, and critical path analysis. SQLite can store edge records, but expressing recursive graph queries (e.g., all transitive dependencies of a WP) requires CTEs that are complex, slow, and hard to maintain. The system also needs to represent richer relationships (Module ownership trees, Cycle membership, feature-to-device replication topology).
**Decision**: Add `agileplus-graph` as a Neo4j-backed graph adapter. The graph crate defines typed node and relationship structs and exposes Cypher-based query functions for the dependency and relationship use cases. Neo4j runs natively via `process-compose` in local dev. The domain model does not depend on the graph crate directly; the CLI and API layers inject the graph store alongside the SQLite store.
**Consequences**: Rich graph queries are expressed naturally in Cypher; the graph layer is a read-optimized secondary store (SQLite is the system of record); Neo4j adds an operational process (like NATS); the graph must be kept in sync with SQLite state -- a sync event or hook writes to both on state mutations; graph crate is an optional feature for deployments that do not need complex dependency queries.
**Alternatives Considered**: SQLite recursive CTEs only (rejected: complex, hard to maintain, limited expressiveness for multi-hop traversals); DGraph (rejected: smaller ecosystem, less mature Rust client); SurrealDB graph queries (considered: multi-model but Rust embedding story is immature); in-process petgraph (considered: suitable for small graphs, but no persistence and no cross-process query capability).

---

## ADR-014: Import Subsystem with Manifest-Driven, Idempotent Ingestion
**Date**: 2026-03-27
**Status**: Accepted
**Context**: Users and agents need to bring existing work items (from Plane.so exports, GitHub issue lists, or JSON files) into AgilePlus without writing code. The import operation must be safe to re-run, must not silently drop errors, and must produce a machine-readable report for downstream automation.
**Decision**: Implement a dedicated `agileplus-import` crate with three components: `ImportManifest` (the input schema), `Importer` (the stateful ingestion engine that validates and persists), and `ImportReport` (the structured outcome). The importer validates each entry against the domain model before persistence. Validation errors are collected per-entry; remaining valid entries are still imported (partial import semantics). Re-importing an existing entity by slug updates non-key fields and emits an audit entry.
**Consequences**: Operators can safely run `agileplus import` repeatedly; a machine-readable JSON report enables automation pipelines to detect failures without screen-scraping; partial import means a malformed entry does not block other entries; the manifest schema is a stable, versioned contract that external tools can target; idempotency requires a slug-based upsert, which means slugs are a stable external identity.
**Alternatives Considered**: Single API endpoint for import (rejected: no manifest schema, hard to batch, requires HTTP client for CLI use); raw SQL import scripts (rejected: bypasses domain model, no audit trail, no validation); full-replace semantics (rejected: destroys existing data for re-imports, not safe for partial updates).
