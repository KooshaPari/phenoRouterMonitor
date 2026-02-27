# Work Packages: AgilePlus — Spec-Driven Development Engine

**Inputs**: Design documents from `kitty-specs/001-spec-driven-development-engine/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: BDD acceptance tests and contract tests included per the test strategy (cucumber-rs, behave, Pact).

**Organization**: 127 subtasks → 21 work packages. Average ~6 subtasks per WP, ~370 lines per prompt.

---

## Work Package WP01: Rust Workspace & Build Scaffold (Priority: P0)

**Goal**: Create the Cargo workspace with all 9 crate stubs, Makefile, Docker Compose, and CI skeleton.
**Independent Test**: `cargo build --workspace` succeeds, `cargo test --workspace` runs (0 tests, 0 errors).
**Prompt**: `tasks/WP01-rust-workspace-scaffold.md`
**Estimated**: ~300 lines, 6 subtasks

### Included Subtasks
- [ ] T001 Create root `Cargo.toml` workspace manifest with all 9 crate members
- [ ] T002 [P] Scaffold `crates/agileplus-core/` with `lib.rs`, domain module stubs, port trait stubs
- [ ] T003 [P] Scaffold remaining 8 adapter crates with `lib.rs` and dependency declarations
- [ ] T004 [P] Create `Makefile` with targets: build, test, lint, format, proto-gen, all
- [ ] T005 [P] Create `docker-compose.yml` for dev environment (Rust builder, Python MCP, SQLite volume)
- [ ] T006 Create `buf.yaml` + `proto/agileplus.proto` from contracts and generate Rust/Python stubs

### Implementation Notes
- Use Rust 2024 edition in all crates
- Core crate has no external deps except serde, sha2, chrono
- Adapter crates depend on core via workspace path
- Proto generation: `tonic-build` for Rust, `grpcio-tools` for Python

### Parallel Opportunities
- T002, T003, T004, T005 are all independent after T001

### Dependencies
- None (starting package)

### Risks & Mitigations
- tonic-build requires protoc; pin version in Makefile and Docker

---

## Work Package WP02: Python MCP Service Scaffold (Priority: P0)

**Goal**: Create the Python MCP service skeleton with FastMCP 3.0, gRPC client stub, and pyproject.toml.
**Independent Test**: `uv run python -m agileplus_mcp` starts without error, MCP server responds to health check.
**Prompt**: `tasks/WP02-python-mcp-scaffold.md`
**Estimated**: ~250 lines, 5 subtasks

### Included Subtasks
- [ ] T007 Create `mcp/pyproject.toml` with FastMCP 3.0, grpcio, opentelemetry-sdk deps
- [ ] T008 Create `mcp/src/agileplus_mcp/__init__.py` and `server.py` (FastMCP entry)
- [ ] T009 [P] Create `mcp/src/agileplus_mcp/grpc_client.py` (stub gRPC connection to Rust core)
- [ ] T010 [P] Create `mcp/src/agileplus_mcp/tools/` directory with stub tool files (features.py, governance.py, status.py)
- [ ] T011 Create `mcp/tests/` directory structure (unit/, bdd/, contract/)

### Implementation Notes
- Use `uv` for Python package management
- FastMCP 3.0 server should register tools from `contracts/mcp-tools.json` schema
- gRPC client connects to `localhost:50051` (Rust core default)

### Parallel Opportunities
- T009, T010 independent after T008

### Dependencies
- Depends on WP01 (proto stubs needed for gRPC client)

### Risks & Mitigations
- Python 3.13 free-threaded + FastMCP compatibility: test early, fall back to 3.12 if needed

---

## Work Package WP03: Domain Model — Feature & State Machine (Priority: P0)

**Goal**: Implement the Feature aggregate, state machine (FSM), and core domain types in `agileplus-core`.
**Independent Test**: Unit tests pass for Feature creation, all valid state transitions, and skip-with-warning behavior.
**Prompt**: `tasks/WP03-domain-feature-state-machine.md`
**Estimated**: ~400 lines, 6 subtasks

### Included Subtasks
- [ ] T012 Implement `Feature` struct with all fields from data-model.md in `crates/agileplus-core/src/domain/feature.rs`
- [ ] T013 Implement `FeatureState` enum and `StateTransition` type with strict ordering (FR-033)
- [ ] T014 Implement state machine logic: `transition()` method enforcing valid transitions, skip-with-warning (FR-034)
- [ ] T015 Implement `WorkPackage` struct with states (planned/doing/review/done/blocked) in `work_package.rs`
- [ ] T016 Implement `WpDependency` and dependency-aware scheduling logic (FR-039)
- [ ] T017 Write unit tests for FSM: all valid transitions, invalid transitions blocked, skip transitions logged

### Implementation Notes
- State machine uses Rust enums with exhaustive match — compiler enforces all transitions handled
- Skip transitions return `Result<Warning>` not `Result<()>` — caller decides to log or abort
- WorkPackage.file_scope is `Vec<String>` for overlap detection

### Parallel Opportunities
- T015-T016 (WP model) parallel with T012-T014 (Feature model) after shared types defined

### Dependencies
- Depends on WP01 (crate structure must exist)

### Risks & Mitigations
- Complex state machine edge cases: exhaustive unit tests with property-based testing (proptest)

---

## Work Package WP04: Domain Model — Governance & Audit (Priority: P0)

**Goal**: Implement governance contracts, audit entries with hash chaining, evidence, and policy rules in `agileplus-core`.
**Independent Test**: Unit tests pass for hash chain creation, verification, evidence linking, and policy evaluation.
**Prompt**: `tasks/WP04-domain-governance-audit.md`
**Estimated**: ~450 lines, 7 subtasks

### Included Subtasks
- [ ] T018 Implement `GovernanceContract` struct with versioned rules (FR-018) in `governance.rs`
- [ ] T019 Implement `AuditEntry` struct with SHA-256 hash chain (FR-016) in `audit.rs`
- [ ] T020 Implement `hash_entry()` function: SHA-256(id ‖ timestamp ‖ actor ‖ transition ‖ evidence_refs ‖ prev_hash)
- [ ] T021 Implement `verify_chain()` function: sequential scan validating prev_hash linkage
- [ ] T022 Implement `Evidence` struct with FR-to-evidence linking (FR-021) in `governance.rs`
- [ ] T023 Implement `PolicyRule` struct with domain-based evaluation (quality/security/reliability) (FR-020)
- [ ] T024 Write unit tests: chain integrity, tamper detection, evidence completeness check, policy pass/fail

### Implementation Notes
- Use sha2 crate for SHA-256
- AuditEntry.hash computed deterministically from concatenated fields
- verify_chain() returns first invalid entry ID or Ok(count)
- GovernanceContract.rules stored as serde_json::Value for flexibility

### Parallel Opportunities
- T018, T022-T023 (governance types) parallel with T019-T021 (audit chain)

### Dependencies
- Depends on WP03 (Feature and WP types needed for evidence linking)

### Risks & Mitigations
- Hash chain correctness is critical: extensive property-based tests, comparison with thegent's implementation

---

## Work Package WP05: Port Traits (Priority: P0)

**Goal**: Define all port traits in `agileplus-core/src/ports/` that adapter crates will implement.
**Independent Test**: Core crate compiles with all port traits defined, adapter crates can reference them.
**Prompt**: `tasks/WP05-port-traits.md`
**Estimated**: ~350 lines, 6 subtasks

### Included Subtasks
- [ ] T025 Define `StoragePort` trait in `ports/storage.rs`: CRUD for features, WPs, audit, evidence, policies
- [ ] T026 [P] Define `VcsPort` trait in `ports/vcs.rs`: worktree create/cleanup, branch ops, artifact read/write
- [ ] T027 [P] Define `AgentPort` trait in `ports/agent.rs`: dispatch subagent, query status, send instruction
- [ ] T028 [P] Define `ReviewPort` trait in `ports/review.rs`: await review, read comments, check CI status
- [ ] T029 [P] Define `ObservabilityPort` trait in `ports/observability.rs`: emit trace, record metric, write log
- [ ] T030 Define `mod.rs` re-exporting all ports, application service traits using ports

### Implementation Notes
- All ports are async traits (use `async_trait` or Rust 2024 native async traits)
- Port methods return `Result<T, DomainError>` — domain error type defined in core
- StoragePort methods mirror data-model.md entities
- VcsPort abstracts git2 — tests can use in-memory mock

### Parallel Opportunities
- T026-T029 all independent

### Dependencies
- Depends on WP03, WP04 (domain types referenced in port signatures)

### Risks & Mitigations
- Port design lock-in: keep ports minimal, add methods incrementally

---

## Work Package WP06: SQLite Adapter (Priority: P1)

**Goal**: Implement SQLite storage adapter with migrations, CRUD operations, and rebuild-from-git capability.
**Independent Test**: Integration tests pass for all CRUD operations, migration up/down, and rebuild from fixtures.
**Prompt**: `tasks/WP06-sqlite-adapter.md`
**Estimated**: ~500 lines, 7 subtasks

### Included Subtasks
- [ ] T031 Create SQLite migration system in `crates/agileplus-sqlite/src/migrations/` (all tables from data-model.md)
- [ ] T032 Implement `SqliteStorageAdapter` struct implementing `StoragePort`
- [ ] T033 Implement feature CRUD: create, get_by_slug, update_state, list_by_state
- [ ] T034 Implement work package CRUD: create, get, update_state, list_by_feature, dependency queries
- [ ] T035 Implement audit CRUD: append_entry, get_trail, verify_chain (delegates to domain)
- [ ] T036 Implement evidence + policy + metric CRUD
- [ ] T037 Implement `rebuild_from_git()` (FR-017): parse git artifacts → populate SQLite

### Implementation Notes
- Use rusqlite with WAL mode for concurrent reads
- Migrations are embedded SQL files, applied on startup
- rebuild_from_git reads: meta.json, audit/chain.jsonl, evidence/** from git working tree
- All queries use parameterized statements (no SQL injection)

### Parallel Opportunities
- T033, T034, T035, T036 are independent after T031-T032

### Dependencies
- Depends on WP05 (StoragePort trait)

### Risks & Mitigations
- WAL mode + single writer: use connection pooling with write serialization

---

## Work Package WP07: Git Adapter (Priority: P1)

**Goal**: Implement git adapter for worktree management, branch ops, and artifact read/write.
**Independent Test**: Integration tests pass for worktree create/cleanup, artifact read/write, branch merge in a temp repo.
**Prompt**: `tasks/WP07-git-adapter.md`
**Estimated**: ~400 lines, 6 subtasks

### Included Subtasks
- [ ] T038 Implement `GitVcsAdapter` struct implementing `VcsPort` in `crates/agileplus-git/src/`
- [ ] T039 Implement worktree operations: create_worktree(feature, wp), list_worktrees, cleanup_worktree
- [ ] T040 Implement branch operations: create_branch, checkout, merge_to_target, detect_conflicts
- [ ] T041 Implement artifact operations: read_spec, read_plan, write_audit_chain, write_evidence
- [ ] T042 Implement git history scanning for `rebuild_from_git()` support
- [ ] T043 Write integration tests using temp git repos (git2::Repository::init)

### Implementation Notes
- Worktree paths: `.worktrees/<feature-slug>-<WP-id>/`
- Use git2 for all operations — no shelling out to git CLI
- Merge conflicts detected via git2 merge analysis, surfaced as structured error

### Parallel Opportunities
- T039, T040, T041 independent after T038

### Dependencies
- Depends on WP05 (VcsPort trait)

### Risks & Mitigations
- git2 worktree API quirks: test on macOS + Linux, handle case-insensitive filesystems

---

## Work Package WP08: Agent Dispatch Adapter (Priority: P1)

**Goal**: Implement agent dispatch for Claude Code and Codex, including PR creation and review loop.
**Independent Test**: Mock dispatch test passes: agent spawned, PR created with goal context, review loop simulated.
**Prompt**: `tasks/WP08-agent-dispatch-adapter.md`
**Estimated**: ~450 lines, 6 subtasks

### Included Subtasks
- [ ] T044 Implement `AgentDispatchAdapter` struct implementing `AgentPort` in `crates/agileplus-agents/src/`
- [ ] T045 Implement `claude_code.rs`: spawn Claude Code with `--print` mode, pass WP prompt, collect output
- [ ] T046 Implement `codex.rs`: spawn Codex in batch mode, pass WP prompt, collect output
- [ ] T047 Implement `dispatch.rs`: select agent (from config), create worktree, inject prompt, spawn 1-3 subagents
- [ ] T048 Implement `pr_loop.rs`: create PR (gh CLI), set description with WP goal/prompt (FR-011), poll for review
- [ ] T049 Implement review-fix loop: read Coderabbit comments → feed to agent → re-push → re-poll (FR-012)

### Implementation Notes
- Agent invocation via `tokio::process::Command` — capture stdout/stderr
- PR creation via `gh pr create` with structured body (FR-011)
- Review loop: poll GitHub API every 30s for review status, Coderabbit comments
- Max review cycles: configurable, default 5
- Agent receives: WP prompt file + spec.md + plan.md + data-model.md as context

### Parallel Opportunities
- T045, T046 independent (different agent harnesses)

### Dependencies
- Depends on WP05 (AgentPort trait), WP07 (worktree creation)

### Risks & Mitigations
- Agent CLI changes: abstract behind AgentPort, adapter is swappable
- Coderabbit latency: configurable poll interval with exponential backoff

---

## Work Package WP09: Code Review Adapter (Priority: P1)

**Goal**: Implement Coderabbit integration and manual review fallback.
**Independent Test**: Mock test passes: Coderabbit review fetched, comments parsed, fallback to manual works.
**Prompt**: `tasks/WP09-review-adapter.md`
**Estimated**: ~300 lines, 5 subtasks

### Included Subtasks
- [ ] T050 Implement `ReviewAdapter` struct implementing `ReviewPort` in `crates/agileplus-review/src/`
- [ ] T051 Implement `coderabbit.rs`: fetch review via GitHub API, parse comments into structured feedback
- [ ] T052 Implement `fallback.rs`: manual review approval flow (user confirms via CLI prompt)
- [ ] T053 Implement CI status checking: poll GitHub checks API for PR, return pass/fail/pending
- [ ] T054 Write unit tests with mock GitHub API responses

### Implementation Notes
- GitHub API via `octocrab` or raw `reqwest` — use integration key from credential store
- Coderabbit comments identified by bot username, parsed for actionable vs informational
- Fallback triggers when Coderabbit unavailable for >5min (configurable)

### Parallel Opportunities
- T051, T052, T053 independent after T050

### Dependencies
- Depends on WP05 (ReviewPort trait)

### Risks & Mitigations
- GitHub API rate limits: cache responses, use conditional requests (ETags)

---

## Work Package WP10: Telemetry Adapter (Priority: P1)

**Goal**: Implement OpenTelemetry traces, metrics, and structured logging.
**Independent Test**: Traces and metrics exported to OTLP collector, structured logs written to file.
**Prompt**: `tasks/WP10-telemetry-adapter.md`
**Estimated**: ~300 lines, 5 subtasks

### Included Subtasks
- [ ] T055 Implement `TelemetryAdapter` struct implementing `ObservabilityPort` in `crates/agileplus-telemetry/src/`
- [ ] T056 Implement `traces.rs`: OpenTelemetry trace spans per command execution, agent dispatch
- [ ] T057 [P] Implement `metrics.rs`: counters (agent_runs, review_cycles), histograms (command_duration_ms)
- [ ] T058 [P] Implement `logs.rs`: structured JSON logging with tracing crate, configurable output (stdout/file)
- [ ] T059 Create `~/.agileplus/otel-config.yaml` schema and loader

### Implementation Notes
- Use `opentelemetry` + `opentelemetry-otlp` crates
- Traces: one span per command, child spans for agent dispatch, review loop iterations
- Metrics stored in SQLite (via StoragePort) AND exported via OTLP
- Log format: JSON with timestamp, level, span_id, message, fields

### Parallel Opportunities
- T056, T057, T058 independent after T055

### Dependencies
- Depends on WP05 (ObservabilityPort trait)

### Risks & Mitigations
- OTLP collector not running: degrade gracefully, log warning, continue without export

---

## Work Package WP11: CLI — Specify & Research Commands (Priority: P1) 🎯 MVP

**Goal**: Implement `specify` and `research` CLI commands with discovery interview and SQLite persistence.
**Independent Test**: `agileplus specify` creates a spec interactively, stores in git+SQLite. `agileplus research` produces research artifacts.
**Prompt**: `tasks/WP11-cli-specify-research.md`
**Estimated**: ~450 lines, 6 subtasks

### Included Subtasks
- [ ] T060 Create `crates/agileplus-cli/src/main.rs` with clap App, global flags, subcommand routing
- [ ] T061 Implement `commands/specify.rs`: guided discovery interview, spec generation, SQLite+git persistence (FR-001)
- [ ] T062 Implement `commands/research.rs`: pre-specify (codebase scan) and post-specify (feasibility) modes (FR-002)
- [ ] T063 Implement implicit refinement: re-run detection, diffing, revision audit logging (FR-008)
- [ ] T064 Implement governance checks within planning commands (FR-009): constitution loading, consistency validation
- [ ] T065 Wire specify/research to StoragePort, VcsPort, ObservabilityPort via dependency injection

### Implementation Notes
- CLI uses clap derive macros for arg parsing
- Discovery interview: structured prompts to stdout, read from stdin
- Spec written to git (kitty-specs/<feature>/spec.md) AND indexed in SQLite
- Research modes determined by presence/absence of spec.md in feature dir

### Parallel Opportunities
- T061, T062 independent after T060

### Dependencies
- Depends on WP06 (SQLite), WP07 (git), WP10 (telemetry)

### Risks & Mitigations
- Interactive CLI complexity: use `dialoguer` crate for structured prompts

---

## Work Package WP12: CLI — Plan & Implement Commands (Priority: P1) 🎯 MVP

**Goal**: Implement `plan` and `implement` CLI commands. Plan generates WPs. Implement dispatches agents.
**Independent Test**: `agileplus plan` generates WPs from spec. `agileplus implement` spawns agents in worktrees.
**Prompt**: `tasks/WP12-cli-plan-implement.md`
**Estimated**: ~500 lines, 7 subtasks

### Included Subtasks
- [ ] T066 Implement `commands/plan.rs`: WP generation with dependency ordering, governance contract creation (FR-003)
- [ ] T067 Implement WP file_scope detection: parse plan for file paths, build overlap graph
- [ ] T068 Implement dependency-aware scheduler: parallel WPs for non-overlapping, serial for overlapping (FR-038, FR-039)
- [ ] T069 Implement `commands/implement.rs`: worktree creation, agent dispatch, PR creation (FR-004, FR-010-013)
- [ ] T070 Implement PR description builder: inject WP goal, FR references, acceptance criteria (FR-011)
- [ ] T071 Implement review-fix loop orchestrator: await Coderabbit, loop agent, detect green (FR-012)
- [ ] T072 Wire plan/implement to all ports (storage, VCS, agent, review, telemetry)

### Implementation Notes
- Plan command reads spec.md + research.md, generates WPs with acceptance criteria traced to FRs
- Implement command respects dependency ordering (WP.depends_on must all be `done`)
- Agent dispatch: 1-3 subagents per worktree based on WP complexity (configurable)
- Review loop: poll every 30s, max 5 cycles, fail after max with governance exception

### Parallel Opportunities
- T066-T068 (plan) parallel with T069-T071 (implement) — different command files

### Dependencies
- Depends on WP08 (agent dispatch), WP09 (review), WP11 (CLI scaffold)

### Risks & Mitigations
- Agent dispatch reliability: implement retry with checkpoint (resume from last commit)

---

## Work Package WP13: CLI — Validate, Ship, Retrospective Commands (Priority: P1)

**Goal**: Implement the final 3 commands to complete the 7-command workflow.
**Independent Test**: `agileplus validate` checks governance gates. `agileplus ship` merges + archives. `agileplus retrospective` generates learnings.
**Prompt**: `tasks/WP13-cli-validate-ship-retro.md`
**Estimated**: ~450 lines, 6 subtasks

### Included Subtasks
- [ ] T073 Implement `commands/validate.rs`: FR-to-evidence tracing, quality gate checks, validation report (FR-005)
- [ ] T074 Implement governance gate evaluator: check all contract rules, collect violations, block if failing (FR-018, FR-019)
- [ ] T075 Implement `commands/ship.rs`: merge to target branch, cleanup worktrees, archive feature, finalize audit (FR-006)
- [ ] T076 Implement `commands/retrospective.rs`: analyze feature history, generate learnings, suggest constitution amendments (FR-007)
- [ ] T077 Implement strict state machine enforcement in all commands: verify current state, transition, log (FR-033, FR-034)
- [ ] T078 Wire validate/ship/retro to all ports

### Implementation Notes
- Validate: iterate all FRs, for each find evidence records, check policy rules
- Ship: git merge to target_branch, git worktree prune, update SQLite state, append final audit
- Retrospective: query metrics table (time-per-WP, review cycles), generate markdown report
- State enforcement: each command checks feature.state before proceeding

### Parallel Opportunities
- T073-T074 (validate) parallel with T075 (ship) — different files

### Dependencies
- Depends on WP12 (implement must exist for validate to have evidence)

### Risks & Mitigations
- Merge conflicts at ship time: detect via git2, present structured diff, suggest resolution

---

## Work Package WP14: gRPC Server & MCP Integration (Priority: P2)

**Goal**: Implement the tonic gRPC server in Rust and wire the Python MCP service to call it.
**Independent Test**: gRPC server starts, Python MCP client connects, tool calls route through to Rust core and return results.
**Prompt**: `tasks/WP14-grpc-mcp-integration.md`
**Estimated**: ~400 lines, 6 subtasks

### Included Subtasks
- [ ] T079 Implement tonic gRPC server in `crates/agileplus-grpc/src/server.rs` implementing `AgilePlusCore` service
- [ ] T080 Wire gRPC handlers to domain services (feature queries, governance checks, audit trail, command dispatch)
- [ ] T081 Implement Python gRPC client in `mcp/src/agileplus_mcp/grpc_client.py`
- [ ] T082 Implement MCP tool handlers in `mcp/src/agileplus_mcp/tools/` — each tool calls gRPC client
- [ ] T083 Implement agent event streaming: bidirectional gRPC stream for real-time agent status
- [ ] T084 Write Pact contract tests for Rust↔Python gRPC boundary

### Implementation Notes
- gRPC server runs on `0.0.0.0:50051` (configurable)
- MCP tools map 1:1 to contracts/mcp-tools.json definitions
- Agent event stream: Rust sends events (agent started, PR created, review received), Python forwards to MCP clients
- Pact: Rust is provider, Python is consumer

### Parallel Opportunities
- T079-T080 (Rust server) parallel with T081-T082 (Python client)

### Dependencies
- Depends on WP13 (all CLI commands must exist for command dispatch)

### Risks & Mitigations
- gRPC streaming complexity: start with unary calls, add streaming incrementally

---

## Work Package WP15: API Layer & Credential Management (Priority: P2)

**Goal**: Implement axum HTTP API for web UI integration and credential management system.
**Independent Test**: API endpoints return feature/WP/audit data as JSON. Credentials stored/retrieved from OS keychain.
**Prompt**: `tasks/WP15-api-credentials.md`
**Estimated**: ~400 lines, 6 subtasks

### Included Subtasks
- [ ] T085 Implement axum router in `crates/agileplus-api/src/` with routes for features, WPs, governance, audit
- [ ] T086 Implement API route handlers: delegate to domain services via ports
- [ ] T087 [P] Implement integration key auth middleware: validate API keys from credential store (FR-030)
- [ ] T088 [P] Implement credential management: OS keychain storage (macOS Keychain, Linux secret-service) (FR-030, FR-031)
- [ ] T089 [P] Create `~/.agileplus/config.toml` schema and loader (core config, credential references)
- [ ] T090 Write API integration tests with mock HTTP client

### Implementation Notes
- axum runs alongside gRPC in the same tokio runtime (shared binary)
- API designed for Plane.so consumption: JSON responses match Plane.so work item format where possible
- Credentials: use `keyring` crate, fallback to encrypted file with passphrase
- Config: TOML with sections for core, credentials, telemetry, api

### Parallel Opportunities
- T087, T088, T089 independent after T085-T086

### Dependencies
- Depends on WP14 (gRPC server — shared binary architecture)

### Risks & Mitigations
- Keychain access permissions on Linux: test with both gnome-keyring and kwallet

---

## Work Package WP16: BDD Acceptance Tests & Integration Suite (Priority: P2)

**Goal**: Write BDD acceptance tests mapping to spec FRs, contract tests, and Docker-based integration tests.
**Independent Test**: `make test` runs all unit, BDD, contract, and integration tests with >80% coverage.
**Prompt**: `tasks/WP16-bdd-integration-tests.md`
**Estimated**: ~450 lines, 7 subtasks

### Included Subtasks
- [ ] T091 Create `.feature` files for core user stories: specify.feature, implement.feature, governance.feature, audit.feature
- [ ] T092 Implement cucumber-rs step definitions for Rust BDD tests in `tests/bdd/`
- [ ] T093 [P] Implement behave step definitions for Python BDD tests in `mcp/tests/bdd/`
- [ ] T094 [P] Create Pact contract test fixtures for gRPC boundary in `tests/contract/`
- [ ] T095 Create `docker-compose.test.yml` for full-stack integration tests
- [ ] T096 Implement integration test scenarios: full workflow (specify → ship) on test repo
- [ ] T097 Create test fixtures: sample specs, plans, WPs, evidence artifacts in `tests/fixtures/`

### Implementation Notes
- BDD .feature files reference FR IDs in scenario names (e.g., "Scenario: FR-001 - Specify creates spec in git+SQLite")
- cucumber-rs and behave share the same .feature files (copied or symlinked)
- Pact: Python consumer writes expected interactions, Rust provider verifies
- Integration tests use a temp git repo created in setUp, torn down in tearDown

### Parallel Opportunities
- T092, T093, T094 all independent

### Dependencies
- Depends on WP15 (all components must exist for full integration)

### Risks & Mitigations
- Docker test environment complexity: use Docker Compose profiles for partial testing

---

## Work Package WP17: Triage & Backlog Adapter (Priority: P2)

**Goal**: Implement the triage classifier, backlog management, and prompt router generation in `agileplus-triage`.
**Independent Test**: Triage classifies input correctly (bug/feature/idea), creates backlog entries, generates a valid CLAUDE.md router.
**Prompt**: `tasks/WP17-triage-backlog-adapter.md`
**Estimated**: ~400 lines, 6 subtasks

### Included Subtasks
- [ ] T098 Implement `TriageAdapter` struct with `classify()` method: bug, feature, idea, task classification
- [ ] T099 Implement `BacklogItem` CRUD: create, list_by_type, list_by_feature, promote_to_feature
- [ ] T100 Implement `classifier.rs`: rule-based + keyword intent classification (extensible for LLM-based later)
- [ ] T101 Implement `router.rs`: generate project-specific CLAUDE.md with prompt routing rules (FR-046)
- [ ] T102 Implement `router.rs`: generate project-specific AGENTS.md with sub-command vocabulary (FR-047)
- [ ] T103 Write unit tests for classification accuracy, backlog operations, router generation

### Dependencies
- Depends on WP05 (port traits), WP06 (SQLite for backlog storage)

---

## Work Package WP18: Plane.so Sync Adapter (Priority: P2)

**Goal**: Implement bidirectional-aware sync from SQLite to Plane.so for features and work packages.
**Independent Test**: Feature state change in SQLite creates/updates corresponding Plane.so work item.
**Prompt**: `tasks/WP18-plane-sync-adapter.md`
**Estimated**: ~350 lines, 5 subtasks

### Included Subtasks
- [ ] T104 Implement `PlaneSyncAdapter` struct with Plane.so REST API client (FR-043)
- [ ] T105 Implement feature sync: SQLite feature → Plane.so work item (create/update on state change)
- [ ] T106 Implement WP sync: SQLite WP → Plane.so sub-item (status, assignee, PR link)
- [ ] T107 Implement conflict detection: poll Plane.so for mirror-side edits, warn on conflicts (FR-045)
- [ ] T108 Write integration tests with mock Plane.so API

### Dependencies
- Depends on WP06 (SQLite), WP15 (credential management for Plane.so API key)

---

## Work Package WP19: GitHub Sync Adapter (Priority: P2)

**Goal**: Implement bug-to-issue sync from SQLite to GitHub Issues with structured metadata.
**Independent Test**: Bug triaged in SQLite creates a GitHub issue with labels, cross-references, and metadata.
**Prompt**: `tasks/WP19-github-sync-adapter.md`
**Estimated**: ~350 lines, 5 subtasks

### Included Subtasks
- [ ] T109 Implement `GitHubSyncAdapter` struct with octocrab GitHub API client (FR-044)
- [ ] T110 Implement bug sync: SQLite backlog bug → GitHub issue (title, body, labels, feature/WP refs)
- [ ] T111 Implement issue status sync: GitHub issue closed → SQLite backlog item resolved
- [ ] T112 Implement conflict detection: warn on GitHub-side edits that conflict with SQLite state (FR-045)
- [ ] T113 Write integration tests with mock GitHub API (wiremock)

### Dependencies
- Depends on WP06 (SQLite), WP15 (credential management for GitHub token)

---

## Work Package WP20: Hidden Sub-Commands & SlashCommand Integration (Priority: P2)

**Goal**: Implement the ~25 hidden sub-commands and wire them for invocation via Claude Code's SlashCommand tool.
**Independent Test**: Each sub-command executes correctly when invoked programmatically; audit log captures all invocations.
**Prompt**: `tasks/WP20-hidden-subcommands.md`
**Estimated**: ~500 lines, 7 subtasks

### Included Subtasks
- [ ] T114 Define sub-command registry: enum of all ~25 sub-commands with metadata (category, description, required args)
- [ ] T115 Implement triage sub-commands: `triage:classify`, `triage:file-bug`, `triage:queue-idea` (FR-040, FR-041, FR-042)
- [ ] T116 Implement governance sub-commands: `governance:check-gates`, `governance:evaluate-policy`, `governance:verify-chain`
- [ ] T117 Implement sync sub-commands: `sync:push-plane`, `sync:push-github`, `sync:pull-status` (FR-043, FR-044)
- [ ] T118 Implement git/devops sub-commands: `git:create-worktree`, `git:branch-from-wp`, `devops:lint-and-format`, `devops:conventional-commit` (FR-051)
- [ ] T119 Implement context + escape sub-commands: `context:load-spec`, `context:scan-codebase`, `escape:quick-fix`, `escape:hotfix`, `meta:generate-router`
- [ ] T120 Implement audit logging for all sub-command invocations (FR-048) and write integration tests

### Dependencies
- Depends on WP13 (CLI commands), WP17 (triage), WP18 (Plane sync), WP19 (GitHub sync)

---

## Work Package WP21: CLI Triage & Queue Commands + Agent Defaults (Priority: P2)

**Goal**: Add `triage` and `queue` as user-facing CLI commands, implement agent DevOps defaults, and auto-triage during implement.
**Independent Test**: `agileplus triage "login is broken"` classifies as bug and creates GitHub issue. Agent auto-triages during implement.
**Prompt**: `tasks/WP21-cli-triage-queue.md`
**Estimated**: ~400 lines, 6 subtasks

### Included Subtasks
- [ ] T121 Implement `commands/triage.rs`: accept input, classify, route to appropriate store (FR-040)
- [ ] T122 Implement `commands/queue.rs`: add to backlog, surface during next specify/plan cycle (FR-042)
- [ ] T123 Implement agent auto-triage hook: during implement, agents auto-file discovered bugs (FR-041)
- [ ] T124 Implement agent DevOps defaults: conventional commits, branch naming, lint-before-push (FR-051)
- [ ] T125 Implement CLAUDE.md/AGENTS.md first-action classifier integration (FR-052)
- [ ] T126 Wire triage/queue to StoragePort, sync adapters, telemetry; write CLI integration tests
- [ ] T127 Seed sub-command prompt files from hybridized reference commands (spec-kitty, bmad, gsd, openspec superset)

### Dependencies
- Depends on WP17 (triage adapter), WP20 (sub-commands)

---

## Dependency & Execution Summary

```
Phase 0 (Foundation — parallel):
  WP01 (Rust scaffold) ──┐
  WP02 (Python scaffold) ─┤── can start in parallel
                          │
Phase 1 (Domain — parallel after WP01):
  WP03 (Feature/FSM) ────┐
  WP04 (Governance/Audit) ┤── parallel, both depend on WP01
  WP05 (Port traits) ─────┘── depends on WP03, WP04

Phase 2 (Adapters — parallel after WP05):
  WP06 (SQLite) ──────────┐
  WP07 (Git) ─────────────┤
  WP08 (Agent dispatch) ──┤── all parallel, all depend on WP05
  WP09 (Review) ──────────┤
  WP10 (Telemetry) ───────┘

Phase 3 (CLI — after adapters):
  WP11 (Specify/Research) ─── depends on WP06, WP07, WP10
  WP12 (Plan/Implement) ──── depends on WP08, WP09, WP11
  WP13 (Validate/Ship/Retro) ── depends on WP12

Phase 4 (Integration — after CLI):
  WP14 (gRPC + MCP) ────── depends on WP13
  WP15 (API + Creds) ───── depends on WP14
  WP16 (BDD + Integration) ── depends on WP15

Phase 5 (Triage, Sync & Sub-Commands — after Phase 2+4):
  WP17 (Triage/Backlog) ──── depends on WP05, WP06
  WP18 (Plane.so Sync) ───── depends on WP06, WP15
  WP19 (GitHub Sync) ──────── depends on WP06, WP15
  WP20 (Hidden Sub-Cmds) ─── depends on WP13, WP17, WP18, WP19
  WP21 (CLI Triage/Queue) ── depends on WP17, WP20
```

**Parallelization**: Up to 5 WPs in Phase 2, WP17-WP19 can run in parallel in Phase 5.

**MVP Scope**: WP01 → WP03 → WP05 → WP06 → WP07 → WP11 → WP12 → WP13 = 8 WPs for core CLI workflow (specify → ship). Phase 5 adds triage/sync/sub-commands as a secondary milestone.

---

## Subtask Index (Reference)

| Subtask | Summary | WP | Priority | Parallel? |
|---------|---------|-----|----------|-----------|
| T001 | Cargo workspace manifest | WP01 | P0 | No |
| T002 | Scaffold agileplus-core | WP01 | P0 | Yes |
| T003 | Scaffold adapter crates | WP01 | P0 | Yes |
| T004 | Makefile | WP01 | P0 | Yes |
| T005 | Docker Compose | WP01 | P0 | Yes |
| T006 | Proto generation | WP01 | P0 | No |
| T007 | Python pyproject.toml | WP02 | P0 | No |
| T008 | FastMCP server entry | WP02 | P0 | No |
| T009 | gRPC client stub | WP02 | P0 | Yes |
| T010 | MCP tool stubs | WP02 | P0 | Yes |
| T011 | Python test structure | WP02 | P0 | No |
| T012 | Feature struct | WP03 | P0 | No |
| T013 | FeatureState enum | WP03 | P0 | No |
| T014 | State machine logic | WP03 | P0 | No |
| T015 | WorkPackage struct | WP03 | P0 | Yes |
| T016 | WP dependency logic | WP03 | P0 | Yes |
| T017 | FSM unit tests | WP03 | P0 | No |
| T018 | GovernanceContract | WP04 | P0 | Yes |
| T019 | AuditEntry struct | WP04 | P0 | Yes |
| T020 | hash_entry() | WP04 | P0 | No |
| T021 | verify_chain() | WP04 | P0 | No |
| T022 | Evidence struct | WP04 | P0 | Yes |
| T023 | PolicyRule struct | WP04 | P0 | Yes |
| T024 | Governance unit tests | WP04 | P0 | No |
| T025 | StoragePort trait | WP05 | P0 | No |
| T026 | VcsPort trait | WP05 | P0 | Yes |
| T027 | AgentPort trait | WP05 | P0 | Yes |
| T028 | ReviewPort trait | WP05 | P0 | Yes |
| T029 | ObservabilityPort trait | WP05 | P0 | Yes |
| T030 | Port module re-exports | WP05 | P0 | No |
| T031 | SQLite migrations | WP06 | P1 | No |
| T032 | SqliteStorageAdapter | WP06 | P1 | No |
| T033 | Feature CRUD | WP06 | P1 | Yes |
| T034 | WP CRUD | WP06 | P1 | Yes |
| T035 | Audit CRUD | WP06 | P1 | Yes |
| T036 | Evidence+policy+metric CRUD | WP06 | P1 | Yes |
| T037 | rebuild_from_git | WP06 | P1 | No |
| T038 | GitVcsAdapter | WP07 | P1 | No |
| T039 | Worktree ops | WP07 | P1 | Yes |
| T040 | Branch ops | WP07 | P1 | Yes |
| T041 | Artifact ops | WP07 | P1 | Yes |
| T042 | Git history scanning | WP07 | P1 | No |
| T043 | Git integration tests | WP07 | P1 | No |
| T044 | AgentDispatchAdapter | WP08 | P1 | No |
| T045 | Claude Code harness | WP08 | P1 | Yes |
| T046 | Codex harness | WP08 | P1 | Yes |
| T047 | Agent dispatch logic | WP08 | P1 | No |
| T048 | PR creation + description | WP08 | P1 | No |
| T049 | Review-fix loop | WP08 | P1 | No |
| T050 | ReviewAdapter | WP09 | P1 | No |
| T051 | Coderabbit integration | WP09 | P1 | Yes |
| T052 | Manual review fallback | WP09 | P1 | Yes |
| T053 | CI status checking | WP09 | P1 | Yes |
| T054 | Review unit tests | WP09 | P1 | No |
| T055 | TelemetryAdapter | WP10 | P1 | No |
| T056 | OTel traces | WP10 | P1 | Yes |
| T057 | OTel metrics | WP10 | P1 | Yes |
| T058 | Structured logging | WP10 | P1 | Yes |
| T059 | OTel config schema | WP10 | P1 | No |
| T060 | CLI main + clap | WP11 | P1 | No |
| T061 | specify command | WP11 | P1 | Yes |
| T062 | research command | WP11 | P1 | Yes |
| T063 | Refinement loop logic | WP11 | P1 | No |
| T064 | Governance checks in planning | WP11 | P1 | No |
| T065 | DI wiring for specify/research | WP11 | P1 | No |
| T066 | plan command | WP12 | P1 | Yes |
| T067 | File scope detection | WP12 | P1 | No |
| T068 | Dependency-aware scheduler | WP12 | P1 | No |
| T069 | implement command | WP12 | P1 | Yes |
| T070 | PR description builder | WP12 | P1 | No |
| T071 | Review-fix orchestrator | WP12 | P1 | No |
| T072 | DI wiring for plan/implement | WP12 | P1 | No |
| T073 | validate command | WP13 | P1 | Yes |
| T074 | Governance gate evaluator | WP13 | P1 | No |
| T075 | ship command | WP13 | P1 | Yes |
| T076 | retrospective command | WP13 | P1 | No |
| T077 | State machine enforcement | WP13 | P1 | No |
| T078 | DI wiring for validate/ship/retro | WP13 | P1 | No |
| T079 | tonic gRPC server | WP14 | P2 | Yes |
| T080 | gRPC handler wiring | WP14 | P2 | No |
| T081 | Python gRPC client | WP14 | P2 | Yes |
| T082 | MCP tool handlers | WP14 | P2 | No |
| T083 | Agent event streaming | WP14 | P2 | No |
| T084 | Pact contract tests | WP14 | P2 | No |
| T085 | axum router | WP15 | P2 | No |
| T086 | API route handlers | WP15 | P2 | No |
| T087 | Auth middleware | WP15 | P2 | Yes |
| T088 | Credential management | WP15 | P2 | Yes |
| T089 | Config schema + loader | WP15 | P2 | Yes |
| T090 | API integration tests | WP15 | P2 | No |
| T091 | BDD .feature files | WP16 | P2 | No |
| T092 | Rust BDD step defs | WP16 | P2 | Yes |
| T093 | Python BDD step defs | WP16 | P2 | Yes |
| T094 | Pact contract fixtures | WP16 | P2 | Yes |
| T095 | Docker Compose test env | WP16 | P2 | No |
| T096 | Full workflow integration test | WP16 | P2 | No |
| T097 | Test fixtures | WP16 | P2 | No |
| T098 | TriageAdapter + classify | WP17 | P2 | No |
| T099 | BacklogItem CRUD | WP17 | P2 | No |
| T100 | Intent classifier | WP17 | P2 | No |
| T101 | CLAUDE.md router gen | WP17 | P2 | No |
| T102 | AGENTS.md gen | WP17 | P2 | No |
| T103 | Triage unit tests | WP17 | P2 | No |
| T104 | PlaneSyncAdapter | WP18 | P2 | No |
| T105 | Feature → Plane.so sync | WP18 | P2 | Yes |
| T106 | WP → Plane.so sync | WP18 | P2 | Yes |
| T107 | Plane.so conflict detection | WP18 | P2 | No |
| T108 | Plane.so mock tests | WP18 | P2 | No |
| T109 | GitHubSyncAdapter | WP19 | P2 | No |
| T110 | Bug → GitHub issue sync | WP19 | P2 | Yes |
| T111 | Issue status sync | WP19 | P2 | Yes |
| T112 | GitHub conflict detection | WP19 | P2 | No |
| T113 | GitHub mock tests | WP19 | P2 | No |
| T114 | Sub-command registry | WP20 | P2 | No |
| T115 | Triage sub-commands | WP20 | P2 | Yes |
| T116 | Governance sub-commands | WP20 | P2 | Yes |
| T117 | Sync sub-commands | WP20 | P2 | Yes |
| T118 | Git/devops sub-commands | WP20 | P2 | Yes |
| T119 | Context/escape sub-commands | WP20 | P2 | Yes |
| T120 | Sub-command audit logging | WP20 | P2 | No |
| T121 | triage CLI command | WP21 | P2 | No |
| T122 | queue CLI command | WP21 | P2 | No |
| T123 | Agent auto-triage hook | WP21 | P2 | No |
| T124 | Agent DevOps defaults | WP21 | P2 | No |
| T125 | CLAUDE.md first-action classifier | WP21 | P2 | No |
| T126 | Triage/queue DI wiring | WP21 | P2 | No |
| T127 | Seed sub-command prompts | WP21 | P2 | No |
