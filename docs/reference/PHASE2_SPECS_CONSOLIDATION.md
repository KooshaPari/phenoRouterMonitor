# Phase 2 Specifications — Consolidation & Foundation Work

**Status:** READY FOR IMPLEMENTATION
**Created:** 2026-03-30
**Target Timeline:** 2026-04-01 through 2026-04-28 (4 weeks)
**Phase:** Phase 2A (Scatter → Unified Structure)

---

## Overview

Phase 2 consolidates the fragmented Phenotype ecosystem into a unified, buildable workspace structure. This document contains three interconnected specs:

1. **phase2a-consolidation** — Project movement & workspace restructuring
2. **phase2b-foundation** — Workspace repair & core implementations
3. **phase2c-monitoring** — Router monitor unblocking & WebSocket support

---

# SPEC 1: phase2a-consolidation

**Title:** Consolidate Scattered Projects into Unified Structure

**Description:**
Phenotype ecosystem projects are scattered across `/Users/kooshapari/Repos/` (agent-wave, civ, heliosCLI, phenodocs, phenotype-design, phenotype-go-kit, phenotype-shared) and need consolidation into a unified workspace structure. This spec covers project discovery, classification, and strategic moves to achieve organizational coherence.

**Owner:** Phase 2 Coordinator
**Priority:** CRITICAL (blocks phase2b and phase2c)
**Effort Estimate:** 3-5 parallel subagents, ~8-10 hours wall-clock time

---

## phase2a Requirements (FR-PHASE2A-001 through FR-PHASE2A-008)

### FR-PHASE2A-001: Discovery & Classification
**Description:** Audit all projects in `/Users/kooshapari/Repos/` and classify by type (infrastructure, documentation, tools, libraries, applications).

**Acceptance Criteria:**
- [ ] All 25+ projects classified (infrastructure, documentation, tooling, libraries, applications)
- [ ] Dependency map created (what depends on what)
- [ ] Archival candidates identified (stale, redundant projects)
- [ ] Report: `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/audits/2026-03-30-ecosystem-discovery.md`

**Traces to:** UJ-PHASE2A-001 (Coordinator discovers ecosystem topology)

---

### FR-PHASE2A-002: Archival of Dead/Stale Projects
**Description:** Move stale, duplicate, or obsolete projects to `.archive/` for preservation without cluttering active workspace.

**Acceptance Criteria:**
- [ ] Stale projects identified (last commit >30 days ago, no recent changes)
- [ ] All stale projects moved to `.archive/scattered-projects/` (non-destructive)
- [ ] Archival log created: `.archive/scattered-projects/ARCHIVAL_LOG.md`
- [ ] Git: `git add .archive/scattered-projects/ARCHIVAL_LOG.md && git commit`

**Projects Candidates for Archival:**
- `znap/` — Zsh plugin (maintenance: unknown)
- `seebeen/`, `justyntemme/`, etc. — User forks (not active development)
- Any with `last commit > 2026-02-01`

**Traces to:** UJ-PHASE2A-002 (Clean up orphaned projects)

---

### FR-PHASE2A-003: Create Fork Strategy for External Contributors
**Description:** Move community fork directories to `.archive/contributor-forks/` and document integration pathway.

**Acceptance Criteria:**
- [ ] 6-8 contributor fork directories identified (Aloxaf/, justyntemme/, etc.)
- [ ] Moved to `.archive/contributor-forks/` with symlinks (optional, for continuity)
- [ ] Document: How to re-integrate future contributions (git submodule, npm package, etc.)
- [ ] File: `docs/reference/CONTRIBUTOR_FORK_STRATEGY.md`

**Example Forks:**
- Aloxaf/ — Community fork of thegent
- justyntemme/ — User contribution
- kylesnowschwartz/ — User fork
- matheusml/, mpartipilo/ — Community repos

**Traces to:** UJ-PHASE2A-003 (Establish community contribution workflow)

---

### FR-PHASE2A-004: Establish Canonical Project Locations
**Description:** Define and enforce canonical locations for all active Phenotype projects.

**Acceptance Criteria:**
- [ ] Canonical locations defined in `WORKSPACE_STRUCTURE.md`:
  - `/repos/phenotype-infrakit/` — Rust workspace (28 crates)
  - `/repos/agent-wave/` — Orchestration docs & spec infrastructure
  - `/repos/heliosCLI/` — CLI runtime & agent harness
  - `/repos/phenodocs/` — @phenotype/docs theme package
  - `/repos/phenotype-design/` — Design system & components
  - `/repos/civ/` — CI/validation infrastructure
  - `/repos/phenotype-go-kit/` — Go shared library
  - `/repos/phenotype-shared/` — TypeScript/JS shared utilities
  - `/repos/polyhex-architecture/` — Architecture exploration & patterns
  - `/repos/phench/` — Benchmarking harness
  - `/repos/template-commons/` — Project templates
  - `/repos/phenotypeActions/` — GitHub Actions suite
- [ ] Document: `/Users/kooshapari/CodeProjects/Phenotype/repos/WORKSPACE_STRUCTURE.md`
- [ ] Enforce: Add to .gitignore any scattered projects not in canonical location

**Traces to:** UJ-PHASE2A-004 (Establish organizational structure)

---

### FR-PHASE2A-005: Dependency Graph Clarification
**Description:** Map all cross-project dependencies to clarify what depends on what.

**Acceptance Criteria:**
- [ ] Dependency graph created (Mermaid diagram): Which projects consume which?
- [ ] Circular dependency detection (expected: none or documented)
- [ ] Registry of npm packages, git submodules, and Cargo workspace members
- [ ] Document: `docs/reference/ECOSYSTEM_DEPENDENCY_GRAPH.md`
- [ ] Diagram embedded in WORKSPACE_STRUCTURE.md

**Key Mappings:**
- phenotype-infrakit (Rust) → depends on: none (canonical)
- agent-wave → depends on: @phenotype/docs (npm)
- phenodocs → consumed by: agent-wave, heliosCLI, phenotype-design, bifrost-extensions
- heliosCLI → depends on: agent-wave, phenodocs
- phenotype-design → depends on: phenodocs

**Traces to:** UJ-PHASE2A-005 (Understand ecosystem dependencies)

---

### FR-PHASE2A-006: Workspace.exclude Rebalancing
**Description:** Update root Cargo.toml workspace.members and workspace.exclude to reflect actual project structure and phase 2 intentions.

**Acceptance Criteria:**
- [ ] All 35 physical crates classified (members vs excluded)
- [ ] Duplicates removed (phenotype-config-core in crates/ and libs/)
- [ ] AgilePlus crates decision made (separate workspace vs exclude)
- [ ] Updated Cargo.toml: `/Users/kooshapari/CodeProjects/Phenotype/repos/Cargo.toml`
- [ ] Verification: `cargo check --workspace` passes
- [ ] Report: `docs/audits/2026-03-30-root-workspace-audit.md` (already created, needs action items tracked)

**Action Items:**
- [ ] Remove `libs/phenotype-config-core` (duplicate)
- [ ] Resolve agileplus-* crate membership conflict
- [ ] Add 18 unaccounted crates (classify each)
- [ ] Verify zero circular dependencies

**Traces to:** FR-PHASE2B-001 (Workspace Foundation)

---

### FR-PHASE2A-007: Documentation of Scattered Projects
**Description:** Document each scattered project's purpose, current state, and integration plan.

**Acceptance Criteria:**
- [ ] Each project has an audit report in `docs/audits/2026-03-30-*.md`:
  - ✓ agent-wave (complete)
  - [ ] civ (CI/validation infrastructure)
  - [ ] heliosCLI (CLI runtime)
  - [ ] phench (benchmarking harness)
  - [ ] phenodocs (@phenotype/docs npm package)
  - [ ] phenotype-design (design system)
  - [ ] phenotype-go-kit (Go shared library)
  - [ ] phenotype-shared (JS/TS shared utilities)
  - [ ] polyhex-architecture (exploration)
  - [ ] template-commons (templates)
  - [ ] phenotypeActions (GitHub Actions)

**Traces to:** UJ-PHASE2A-007 (Document all projects)

---

### FR-PHASE2A-008: Integration Pathway Planning
**Description:** Define how each scattered project integrates into unified workspace.

**Acceptance Criteria:**
- [ ] Integration strategy document: `docs/reference/INTEGRATION_PATHWAY.md`
- [ ] For each project:
  - Current state (standalone repo, monorepo member, NPM package, etc.)
  - Target state (Phase 2B, 2C, Phase 3+)
  - Migration steps (submodule, move, fork, etc.)
  - Blocked by: list of dependencies
- [ ] Phased rollout plan (Week 1, 2, 3, 4)

**Example Pathway:**
```
agent-wave (ACTIVE)
  ├─ Current: standalone repo + submodule (phenodocs)
  ├─ Target: documentation infrastructure provider
  ├─ Blocked by: phenodocs PR #91 (publishing workflow)
  └─ Action: Finalize submodule integration, unblock phenodocs

heliosCLI (ACTIVE)
  ├─ Current: standalone repo
  ├─ Target: Core CLI runtime (Phase 2B)
  ├─ Dependencies: agent-wave, phenodocs, phenotype-infrakit
  └─ Action: Begin Phase 2B workspace integration

civ (ACTIVE)
  ├─ Current: standalone repo
  ├─ Target: CI/validation provider
  ├─ Dependencies: phenotype-infrakit (error-core, health, contracts)
  └─ Action: Phase 2C (monitoring & router unblocking)
```

**Traces to:** UJ-PHASE2A-008 (Plan integration strategy)

---

## phase2a Work Packages

### WP-2a-001: Full Ecosystem Audit & Classification

**Deliverables:**
1. Audit report: `/docs/audits/2026-03-30-ecosystem-discovery.md` (35+ projects, classification, dependencies)
2. Dependency graph: `docs/reference/ECOSYSTEM_DEPENDENCY_GRAPH.md` (Mermaid diagram, registry)
3. Archival log: `.archive/scattered-projects/ARCHIVAL_LOG.md`
4. Candidate list: Stale projects for archival (with last-commit dates)

**Acceptance Criteria:**
- All 25+ projects in `/Repos/` audited
- Classification complete (infrastructure, docs, tools, libraries, apps)
- Dependency map created (text + Mermaid diagram)
- Circular dependency check passed

**Owner:** ecosystem-auditor (haiku subagent)
**Effort:** 2-3 parallel explores, ~2 hours wall-clock

**Dependencies:** None (parallel start)

---

### WP-2a-002: Archival & Cleanup

**Deliverables:**
1. Stale projects moved to `.archive/scattered-projects/`
2. Contributor forks moved to `.archive/contributor-forks/`
3. Archival log updated with timestamps
4. Git commit: "chore: archive stale scattered projects"

**Acceptance Criteria:**
- All identified stale projects moved
- All contributor forks organized
- Git history preserved (git mv, not rm)
- `.archive/scattered-projects/ARCHIVAL_LOG.md` documents each move

**Owner:** archival-agent (haiku)
**Effort:** 1 agent, ~1 hour

**Blocked By:** WP-2a-001 (identification)

---

### WP-2a-003: Canonical Structure Definition

**Deliverables:**
1. `WORKSPACE_STRUCTURE.md` — Canonical locations for all active projects
2. `docs/reference/CONTRIBUTOR_FORK_STRATEGY.md` — Community contribution workflow
3. Updated `.gitignore` (optional: exclude scattered projects not in canonical)

**Acceptance Criteria:**
- 12+ canonical locations defined
- All active Phenotype projects mapped
- Fork strategy documented (submodule, npm package, or other)
- Clear integration pathway for community contributions

**Owner:** architecture-planner (haiku)
**Effort:** 1 agent, ~1.5 hours

**Blocked By:** WP-2a-001 (classification)

---

### WP-2a-004: Individual Project Audits

**Deliverables:**
1. 10-11 individual audit reports in `docs/audits/`:
   - civ-audit.md
   - heliosCLI-audit.md
   - phench-audit.md
   - phenodocs-audit.md
   - phenotype-design-audit.md
   - phenotype-go-kit-audit.md
   - phenotype-shared-audit.md
   - polyhex-architecture-audit.md
   - template-commons-audit.md
   - phenotypeActions-audit.md

**Acceptance Criteria:**
- Each audit covers: purpose, language, structure, tests, phenotype integration, blockers
- All match agent-wave audit format (for consistency)
- Dependency list complete for each project

**Owner:** 5-6 parallel haiku auditors (one per 2-3 projects)
**Effort:** 5-6 agents, ~2 hours wall-clock

**Blocked By:** WP-2a-001 (classification)

---

### WP-2a-005: Integration Pathway & Phased Rollout

**Deliverables:**
1. `docs/reference/INTEGRATION_PATHWAY.md` — Current → Target state for each project
2. Phased rollout plan (Week 1, 2, 3, 4)
3. Blocker identification for each migration
4. Dependency list for phase2b and phase2c

**Acceptance Criteria:**
- All 12+ projects have defined pathway
- Blockers clearly identified
- Rollout plan is executable in 4 weeks
- Dependencies documented for phase2b/2c spec teams

**Owner:** integration-planner (haiku)
**Effort:** 1 agent, ~2 hours

**Blocked By:** WP-2a-001, WP-2a-003, WP-2a-004 (all audits complete)

---

### WP-2a-006: Cargo.toml Workspace Rebalancing

**Deliverables:**
1. Updated `Cargo.toml`: corrected members/exclude lists
2. Removed duplicate phenotype-config-core (libs/ version archived)
3. Resolved agileplus-* crate decision
4. Added all 18 unaccounted crates to members or excluded
5. Verification: `cargo check --workspace` passes
6. Commit: "fix: rebalance Cargo.toml workspace structure"

**Acceptance Criteria:**
- Zero duplicate crate definitions
- All 35 crates declared (either members or excluded)
- Cargo build passes with zero warnings
- No circular dependencies
- Test: `cargo test --workspace` passes

**Owner:** workspace-repair-agent (haiku)
**Effort:** 1-2 agents, ~1.5-2 hours

**Blocked By:** WP-2a-001, root-workspace-audit (Task 4, already complete)

---

## phase2a Success Metrics

- [ ] All 25+ scattered projects audited and classified
- [ ] Archival complete: stale projects moved to `.archive/`
- [ ] Canonical structure defined and documented
- [ ] All individual project audits written
- [ ] Integration pathway clear (each project has current → target state)
- [ ] Cargo.toml corrected: `cargo check --workspace` passes
- [ ] Zero unaccounted crates remaining
- [ ] 4-week rollout plan validated by all downstream teams

---

## phase2a Timeline

| Week | Deliverables |
|------|--------------|
| Week 1 (Apr 1-4) | WP-2a-001, WP-2a-002, WP-2a-003 complete (audits, archival, structure) |
| Week 1 (Apr 1-5) | WP-2a-004 complete (10+ individual audits) |
| Week 2 (Apr 7-11) | WP-2a-005, WP-2a-006 complete (pathways, Cargo.toml fix) |
| Validation | `cargo check --workspace` passes, all audits reviewed |

---

---

# SPEC 2: phase2b-foundation

**Title:** Workspace Foundation Repair & Core Implementations

**Description:**
Phase 2B follows phase2a consolidation and focuses on repairing the workspace foundation, ensuring all 28 crates in phenotype-infrakit are properly integrated, tested, and production-ready. Includes core implementations (forgecode, bifrost contracts), test infrastructure, and workspace stability.

**Owner:** Phase 2 Coordinator
**Priority:** HIGH (blocks phase2c)
**Effort Estimate:** 4-6 parallel subagents, ~12-15 hours wall-clock time

---

## phase2b Requirements (FR-PHASE2B-001 through FR-PHASE2B-004)

### FR-PHASE2B-001: Workspace Structure Verification
**Description:** Verify all 28 member crates build cleanly, tests pass, and no warnings exist.

**Acceptance Criteria:**
- [ ] All 28 member crates in Cargo.toml
- [ ] `cargo check --workspace` passes (zero errors, zero warnings)
- [ ] `cargo test --workspace` passes (all test suites)
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo fmt --check` passes
- [ ] CI: All GitHub Actions workflows pass

**Blocked By:** phase2a (WP-2a-006: Cargo.toml rebalancing)

**Traces to:** UJ-PHASE2B-001 (Ensure workspace foundation)

---

### FR-PHASE2B-002: Core Implementation — Forgecode
**Description:** Implement forgecode system for generating deterministic, universally-comparable codes for any Rust struct.

**Acceptance Criteria:**
- [ ] `crates/phenotype-forgecode/` created
- [ ] ForgecodeGenerator trait defined (sha256-based)
- [ ] Implementations for 5+ core types (String, Vec, HashMap, custom structs)
- [ ] Tests: 100% coverage of ForgecodeGenerator implementations
- [ ] Traces: All tests reference FR-PHASE2B-002-*
- [ ] Doc: forgecode semantics & determinism guarantees documented

**Acceptance Criteria (Detailed):**
- [ ] trait ForgecodeGenerator { fn forgecode(&self) -> String; }
- [ ] impl for String: sha256(utf8_bytes).to_hex()
- [ ] impl for Vec<T>: concatenate all elements' forgecodes, then sha256
- [ ] impl for HashMap<K,V>: sort by key, concatenate all kv forgecodes, then sha256
- [ ] impl for Option<T>: "None" or T.forgecode()
- [ ] impl for custom structs: derive macro or manual impl
- [ ] Test: determinism (same input → same code)
- [ ] Test: collision resistance (different inputs → different codes with high probability)
- [ ] Test: composability (nested structures maintain forgecode property)

**Language:** Rust
**Traces to:** UJ-PHASE2B-002 (Generate deterministic codes)

---

### FR-PHASE2B-003: Core Implementation — Bifrost Contracts
**Description:** Implement Bifrost contract system for multi-stage deployments.

**Acceptance Criteria:**
- [ ] `crates/phenotype-bifrost/` created
- [ ] Contract trait defined: stage lifecycle, execution guards
- [ ] 3 stage implementations: PreFlight, Deploy, PostValidation
- [ ] Tests: stage transitions, guard execution, failure handling
- [ ] Traces: All tests reference FR-PHASE2B-003-*
- [ ] Integration: Contracts can be composed (Contract<A> + Contract<B>)

**Acceptance Criteria (Detailed):**
- [ ] trait BifrostContract { fn validate(&self) -> Result<()>; fn execute(&mut self) -> Result<()>; }
- [ ] struct PreFlightStage; impl stage validation (preconditions)
- [ ] struct DeployStage; impl actual deployment logic
- [ ] struct PostValidationStage; impl post-deploy checks
- [ ] Tests: Successful contract execution (all stages pass)
- [ ] Tests: Failed precondition (PreFlight fails, Deploy not executed)
- [ ] Tests: Rollback logic (Deploy succeeds, PostValidation fails → rollback)
- [ ] Tests: Composition (multiple contracts in sequence)

**Language:** Rust
**Traces to:** UJ-PHASE2B-003 (Multi-stage deployment contracts)

---

### FR-PHASE2B-004: Test Infrastructure & Mocking
**Description:** Implement phenotype-test-infra crate with comprehensive mocking, property-based testing, and spec traceability.

**Acceptance Criteria:**
- [ ] `crates/phenotype-test-infra/` enhanced with:
  - [ ] MockBuilder<T> for quick test doubles
  - [ ] PropertyTest trait for property-based testing
  - [ ] SpecTrace macro for linking tests to FR specs
  - [ ] Suite of example implementations (MockDB, MockCache, etc.)
- [ ] Tests: 100% coverage of test infrastructure itself
- [ ] Documentation: Test infrastructure patterns & examples
- [ ] All new tests use SpecTrace linking

**Acceptance Criteria (Detailed):**
- [ ] macro test_spec(FR_ID) { /* test body */ } expands to #[test] with spec link
- [ ] MockBuilder::new().with_field(value).build() creates test double
- [ ] PropertyTest::generate() creates random input for property-based tests
- [ ] Example mocks: MockHealthChecker, MockCacheAdapter, MockRepository
- [ ] Tests: MockBuilder creates valid test doubles
- [ ] Tests: PropertyTest generates diverse inputs (doesn't repeat)
- [ ] Docs: "How to write a test linked to FR"

**Language:** Rust
**Traces to:** UJ-PHASE2B-004 (Test infrastructure)

---

## phase2b Work Packages

### WP-2b-001: Workspace Verification & Lint
**Deliverables:**
- [ ] All 28 crates build cleanly
- [ ] All tests pass
- [ ] Zero warnings (`clippy`, `fmt`)
- [ ] GitHub Actions all green
- [ ] Report: `docs/reports/2026-03-30-workspace-verification.md`

**Owner:** workspace-verifier (haiku)
**Effort:** 1-2 agents, ~2 hours

**Blocked By:** phase2a (WP-2a-006)

---

### WP-2b-002: Forgecode Implementation
**Deliverables:**
- [ ] `crates/phenotype-forgecode/` with full impl
- [ ] 8+ test cases (determinism, collision, composition)
- [ ] Documentation: forgecode semantics & guarantees
- [ ] Traces: All tests → FR-PHASE2B-002

**Owner:** forgecode-implementer (haiku)
**Effort:** 1-2 agents, ~3-4 hours

**Blocked By:** WP-2b-001 (workspace ready)

---

### WP-2b-003: Bifrost Contracts Implementation
**Deliverables:**
- [ ] `crates/phenotype-bifrost/` with stage contracts
- [ ] 3 stage types + composition logic
- [ ] 10+ test cases (success, failure, rollback)
- [ ] Traces: All tests → FR-PHASE2B-003

**Owner:** bifrost-implementer (haiku)
**Effort:** 1-2 agents, ~3-4 hours

**Blocked By:** WP-2b-001 (workspace ready)

---

### WP-2b-004: Test Infrastructure Enhancement
**Deliverables:**
- [ ] `crates/phenotype-test-infra/` enhanced
- [ ] MockBuilder, PropertyTest, SpecTrace
- [ ] 5+ example mocks
- [ ] 15+ tests of test infrastructure itself
- [ ] Documentation: test patterns & examples

**Owner:** test-infrastructure-agent (haiku)
**Effort:** 1-2 agents, ~2-3 hours

**Blocked By:** WP-2b-001 (workspace ready)

---

### WP-2b-005: Integration & Verification
**Deliverables:**
- [ ] All implementations integrated into workspace
- [ ] Tests passing (100% suite)
- [ ] Clippy zero warnings
- [ ] Final report: `docs/reports/2026-04-XX-phase2b-completion.md`

**Owner:** integration-verifier (haiku)
**Effort:** 1 agent, ~1-2 hours

**Blocked By:** WP-2b-002, WP-2b-003, WP-2b-004

---

## phase2b Timeline

| Period | Deliverables |
|--------|--------------|
| Week 2-3 (Apr 7-18) | WP-2b-001, WP-2b-002, WP-2b-003 in parallel |
| Week 3 (Apr 18-22) | WP-2b-004, WP-2b-005 |

---

---

# SPEC 3: phase2c-monitoring

**Title:** Router Monitor Unblocking & WebSocket Support

**Description:**
Phase 2C unblocks the router monitoring system (civ) by implementing missing WebSocket handler and completing observer pattern support. Enables real-time CI/validation monitoring across Phenotype ecosystem.

**Owner:** Phase 2 Coordinator
**Priority:** HIGH (unblocks real-time monitoring)
**Effort Estimate:** 2-3 parallel subagents, ~6-8 hours wall-clock time

---

## phase2c Requirements (FR-PHASE2C-001 through FR-PHASE2C-003)

### FR-PHASE2C-001: Router Monitor Implementation
**Description:** Implement missing router monitor handler for civ (CI/validation infrastructure).

**Acceptance Criteria:**
- [ ] `crates/phenotype-monitor/` created (or enhanced)
- [ ] RouterMonitor trait: `fn observe(&self, route: Route) -> Result<()>`
- [ ] WebSocket handler implemented: `fn handle_websocket(&mut self, msg: Message)`
- [ ] Tests: 100% coverage (observe, websocket handling, error cases)
- [ ] Traces: All tests → FR-PHASE2C-001

**Acceptance Criteria (Detailed):**
- [ ] trait RouterMonitor { fn observe(&self, route: Route) -> Result<()>; }
- [ ] impl for HTTP routes: track request/response pairs
- [ ] impl for WebSocket: bidirectional message handling
- [ ] WebSocket message types: Subscribe, Unsubscribe, Update, Error
- [ ] Observer pattern: Notified on route changes
- [ ] Tests: Successful observe (200 OK response)
- [ ] Tests: WebSocket subscription (receives updates)
- [ ] Tests: Observer notification (all registered observers notified)
- [ ] Tests: Error handling (invalid route, connection loss)

**Blocked By:** phase2b (workspace foundation)

**Traces to:** UJ-PHASE2C-001 (Monitor router health)

---

### FR-PHASE2C-002: Observer Pattern Completion
**Description:** Implement full observer pattern support for route monitoring.

**Acceptance Criteria:**
- [ ] Observer trait: `fn on_route_change(&mut self, route: &Route) -> Result<()>`
- [ ] Observable trait: `fn subscribe(&mut self, observer: Box<dyn Observer>)`
- [ ] Notifications: All observers notified on route state changes
- [ ] Tests: 100% coverage (subscribe, notify, unsubscribe)
- [ ] Integration: Works with RouterMonitor

**Acceptance Criteria (Detailed):**
- [ ] trait Observer { fn on_route_change(&mut self, route: &Route) -> Result<()>; }
- [ ] trait Observable { fn subscribe(&mut self, obs: Box<dyn Observer>); fn notify_all(&mut self, route: &Route); }
- [ ] Tests: Subscribe observer successfully
- [ ] Tests: Observer notified on route change
- [ ] Tests: Multiple observers (all notified)
- [ ] Tests: Unsubscribe stops notifications

**Blocked By:** phase2b (workspace foundation)

**Traces to:** UJ-PHASE2C-002 (Observer pattern)

---

### FR-PHASE2C-003: WebSocket Gateway Integration
**Description:** Integrate WebSocket support into phenotype gateway for real-time updates.

**Acceptance Criteria:**
- [ ] WebSocket endpoint: `/ws/routes` (subscribe to route updates)
- [ ] WebSocket endpoint: `/ws/health` (subscribe to health checks)
- [ ] Message format: JSON with `type`, `data`, `timestamp` fields
- [ ] Tests: 100% coverage (subscribe, receive updates, error handling)
- [ ] Integration: Works with civ's validation pipeline

**Acceptance Criteria (Detailed):**
- [ ] Route change message: `{ "type": "route_update", "data": { "route": "...", "status": "..." }, "timestamp": "..." }`
- [ ] Health check message: `{ "type": "health_check", "data": { "service": "...", "status": "..." }, "timestamp": "..." }`
- [ ] Tests: Subscribe to `/ws/routes`
- [ ] Tests: Receive route update message
- [ ] Tests: Unsubscribe stops delivery
- [ ] Tests: Connection loss handling (retry)

**Blocked By:** phase2c-001, phase2c-002

**Traces to:** UJ-PHASE2C-003 (Real-time updates via WebSocket)

---

## phase2c Work Packages

### WP-2c-001: Router Monitor Implementation
**Deliverables:**
- [ ] `crates/phenotype-monitor/` with RouterMonitor trait
- [ ] WebSocket handler implementation
- [ ] 10+ tests (observe, websocket, error cases)
- [ ] Integration with Observer pattern

**Owner:** router-monitor-implementer (haiku)
**Effort:** 1-2 agents, ~2-3 hours

**Blocked By:** phase2b (WP-2b-001)

---

### WP-2c-002: Observer Pattern Completion
**Deliverables:**
- [ ] Observer + Observable traits
- [ ] Notification logic
- [ ] 8+ tests (subscribe, notify, unsubscribe)
- [ ] Documentation: Observer pattern in Phenotype ecosystem

**Owner:** observer-implementer (haiku)
**Effort:** 1 agent, ~1.5-2 hours

**Blocked By:** phase2b (WP-2b-001)

---

### WP-2c-003: WebSocket Gateway Integration
**Deliverables:**
- [ ] `/ws/routes` endpoint
- [ ] `/ws/health` endpoint
- [ ] JSON message format
- [ ] 12+ tests (subscribe, updates, errors, reconnect)
- [ ] Integration with civ

**Owner:** websocket-integrator (haiku)
**Effort:** 1-2 agents, ~2-3 hours

**Blocked By:** WP-2c-001, WP-2c-002

---

## phase2c Timeline

| Period | Deliverables |
|--------|--------------|
| Week 3-4 (Apr 18-25) | WP-2c-001, WP-2c-002 in parallel |
| Week 4 (Apr 25-28) | WP-2c-003, final verification |

---

---

# Cross-Spec Dependencies & Rollout Schedule

```
phase2a (Apr 1-11)
├─ WP-2a-001: Ecosystem audit & classification
├─ WP-2a-002: Archival & cleanup
├─ WP-2a-003: Canonical structure definition
├─ WP-2a-004: Individual project audits (parallel)
├─ WP-2a-005: Integration pathway planning
└─ WP-2a-006: Cargo.toml workspace rebalancing
    └─ BLOCKS phase2b

phase2b (Apr 7-22) [starts mid-phase2a]
├─ WP-2b-001: Workspace verification & lint
├─ WP-2b-002: Forgecode implementation
├─ WP-2b-003: Bifrost contracts
├─ WP-2b-004: Test infrastructure
└─ WP-2b-005: Integration & verification
    └─ BLOCKS phase2c

phase2c (Apr 18-28) [starts mid-phase2b]
├─ WP-2c-001: Router monitor implementation
├─ WP-2c-002: Observer pattern completion
└─ WP-2c-003: WebSocket gateway integration
```

---

# Integration with AgilePlus Workflow

These three specs (phase2a, phase2b, phase2c) are designed to integrate with AgilePlus task management:

**AgilePlus Commands (when available):**
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus

# Create phase2a spec
agileplus specify --title "Phase 2A: Consolidate Scattered Projects" \
  --description "Audit, classify, and organize 25+ scattered Phenotype projects"

# Add work packages
agileplus task add phase2a-consolidation --id "WP01" \
  --title "Full Ecosystem Audit & Classification" \
  --description "Audit 25+ projects, classify by type, create dependency graph"

agileplus task add phase2a-consolidation --id "WP02" \
  --title "Archival & Cleanup" \
  --description "Move stale projects to .archive/, organize contributor forks"

# And so on for WP03-06, then phase2b WP01-05, then phase2c WP01-03
```

**For now (manual tracking):**
- Use `docs/reference/PHASE2_SPECS_CONSOLIDATION.md` (this document) as source of truth
- Track progress in `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/worklogs/PHASE2_PROGRESS.md`
- Update spec status after each WP completion

---

# Acceptance Criteria Summary

Phase 2 is complete when:

✅ **phase2a (Consolidation):**
- All 25+ scattered projects audited & classified
- Stale projects archived
- Canonical structure defined & documented
- Cargo.toml workspace corrected (all 35 crates accounted for)
- Integration pathways clear for all projects

✅ **phase2b (Foundation):**
- All 28 member crates build cleanly (zero warnings)
- forgecode system implemented & tested
- bifrost contracts implemented & tested
- test infrastructure enhanced with MockBuilder, PropertyTest, SpecTrace
- All tests passing, all traces linked to FR

✅ **phase2c (Monitoring):**
- RouterMonitor implemented with WebSocket support
- Observer pattern fully functional
- WebSocket gateway endpoints live (/ws/routes, /ws/health)
- Real-time monitoring working in civ

---

**Report Generated:** 2026-03-30
**Phase Timeline:** 2026-04-01 through 2026-04-28 (4 weeks)
**Status:** READY FOR IMPLEMENTATION
**Next Steps:** Execute phase2a per spec, trigger phase2b when phase2a-WP-06 complete, trigger phase2c when phase2b-WP-05 complete
