# Phenotype Polyrepo: Master Functional Requirements

**Version:** 3.0 | **Status:** Active | **Updated:** 2026-03-30
**Branch:** `specs/main` (canonical single source of truth)
**Format:** Master consolidated registry of all FR files across Phenotype organization repositories
**Last Consolidated:** 2026-03-30 03:15 UTC

---

## Overview

This master FUNCTIONAL_REQUIREMENTS.md consolidates all functional requirements from the Phenotype polyrepo ecosystem. Each FR includes:
- **Unique ID**: `FR-<REPO>-<NNN>` (e.g., `FR-INFRA-001`, `FR-AGILE-001`, `FR-HELIOS-001`)
- **Requirement**: Clear SHALL/MUST statement
- **Epic Trace**: Links to PRD epic (e.g., E1.1, E2.3)
- **Code Location**: Path to implementation
- **Repository Origin**: Source repo (phenotype-infrakit, AgilePlus, heliosCLI, platforms/thegent, etc.)
- **Status**: Active, Planned, Deprecated, or Superseded

---

## Repository Index

| Repository | Prefix | Primary Domain | Owner | Status |
|-----------|--------|---------------|-------|--------|
| phenotype-infrakit | `FR-INFRA` | Generic infrastructure crates | Core | Active |
| AgilePlus | `FR-AGILE` | Specification & workflow management | Core | Active |
| heliosCLI | `FR-HELIOS` | CLI agent harness & sandboxing | Tools | Active |
| platforms/thegent | `FR-THEGENT` | Agentic execution platform | Platform | Active |
| agent-wave | `FR-WAVE` | Agent orchestration & federation | Platform | Active |
| agentapi-plusplus | `FR-API` | API gateway & backend services | Services | Active |
| cliproxyapi-plusplus | `FR-CLIPROXY` | Proxy services for CLI agents | Services | Active |
| portage | `FR-PORTAGE` | Package management & versioning | Tools | Planned |
| clikit | `FR-CLIKIT` | CLI toolkit abstractions | Libraries | Planned |
| forgecode | `FR-FORGE` | Code generation & templates | Tools | Planned |
| kits | `FR-KITS` | Project templates & scaffolding | Libraries | Planned |

---

## Master FR Registry

### FR-INFRA: Infrastructure Crates (phenotype-infrakit)

#### FR-INFRA-001: Event Sourcing Store
**Requirement:** System SHALL implement an append-only event store with SHA-256 hash chains for immutable event persistence.
**Traces To:** E3.1
**Code Location:** `crates/phenotype-event-sourcing/src/async_store.rs`
**Repository:** phenotype-infrakit
**Status:** Active
**Test Traces:** Inline tests in async_store.rs

#### FR-INFRA-002: Cache Abstraction with TTL
**Requirement:** System SHALL implement a two-tier LRU + DashMap cache with configurable TTL and eviction policies.
**Traces To:** E9.2
**Code Location:** `crates/phenotype-cache-adapter/src/lib.rs`
**Repository:** phenotype-infrakit
**Status:** Active
**Test Traces:** phenotype-cache-adapter tests

#### FR-INFRA-003: Policy Engine
**Requirement:** System SHALL implement rule-based policy evaluation engine with TOML configuration support.
**Traces To:** E4.6
**Code Location:** `crates/phenotype-policy-engine/src/lib.rs`
**Repository:** phenotype-infrakit
**Status:** Active
**Test Traces:** phenotype-policy-engine tests

#### FR-INFRA-004: Generic State Machine
**Requirement:** System SHALL implement generic FSM with transition guards and state validation.
**Traces To:** E1.2
**Code Location:** `crates/phenotype-state-machine/src/lib.rs`
**Repository:** phenotype-infrakit
**Status:** Active
**Test Traces:** phenotype-state-machine tests

#### FR-INFRA-005: Canonical Error Types
**Requirement:** System SHALL define 5 canonical error types (Validation, Conflict, NotFound, IO, Integration) consolidating 85+ project error enums.
**Traces To:** E5.1
**Code Location:** `crates/phenotype-error-core/src/lib.rs`
**Repository:** phenotype-infrakit
**Status:** Active
**Test Traces:** phenotype-error-core tests

#### FR-INFRA-006: Health Check Abstraction
**Requirement:** System SHALL define HealthChecker trait with 4 standard implementations (HTTP, Database, Service, Aggregate).
**Traces To:** E12.3
**Code Location:** `crates/phenotype-health/src/lib.rs`
**Repository:** phenotype-infrakit
**Status:** Active
**Test Traces:** phenotype-health tests

#### FR-INFRA-007: Configuration Management
**Requirement:** System SHALL provide unified config loader supporting multiple sources (environment, files, TOML) with validation.
**Traces To:** E5.3
**Code Location:** `crates/phenotype-config-core/src/lib.rs`
**Repository:** phenotype-infrakit
**Status:** Active
**Test Traces:** phenotype-config-core tests

---

### FR-AGILE: AgilePlus (AgilePlus)

#### FR-AGILE-001: Feature Entity
**Requirement:** System SHALL define a `Feature` entity with fields: `id`, `slug`, `friendly_name`, `state` (FeatureState), `spec_hash`, `target_branch`, `plane_issue_id`, `plane_state_id`, `labels`, `module_id`, `project_id`, `created_at_commit`, `last_modified_commit`, `created_at`, `updated_at`.
**Traces To:** E1.1
**Code Location:** `crates/agileplus-domain/src/domain/feature.rs`
**Repository:** AgilePlus
**Status:** Active
**Test Traces:** Feature entity tests

#### FR-AGILE-002: Feature State Machine
**Requirement:** System SHALL define `FeatureState` as ordered enum: `Created`, `Specified`, `Researched`, `Planned`, `Implementing`, `Validated`, `Shipped`, `Retrospected` with monotonically increasing ordinals.
**Traces To:** E1.2
**Code Location:** `crates/agileplus-domain/src/domain/state_machine.rs`
**Repository:** AgilePlus
**Status:** Active
**Test Traces:** State machine validation tests

#### FR-AGILE-003: Forward-Only State Transitions
**Requirement:** System SHALL enforce forward-only state transitions; any attempt to transition to lower-ordinal state SHALL return `DomainError::InvalidTransition`.
**Traces To:** E1.2
**Code Location:** `crates/agileplus-domain/src/domain/state_machine.rs`
**Repository:** AgilePlus
**Status:** Active
**Test Traces:** Transition validation tests

#### FR-AGILE-004: Audit Trail with Hash Chain
**Requirement:** System SHALL define `AuditEntry` with hash chain validation; each entry's hash computed as SHA-256 over `feature_id`, `wp_id`, `timestamp`, `actor`, `transition`, and `prev_hash`.
**Traces To:** E3.1
**Code Location:** `crates/agileplus-domain/src/domain/audit.rs`
**Repository:** AgilePlus
**Status:** Active
**Test Traces:** Audit chain verification tests

#### FR-AGILE-005: Work Package Entity
**Requirement:** System SHALL define `WorkPackage` entity with fields: `id`, `feature_id`, `ordinal`, `title`, `description`, `state`, `file_scope`, `acceptance_criteria`, `assigned_agent`, `pr_url`, `worktree_path`, `base_commit`, `head_commit`, `created_at`, `updated_at`.
**Traces To:** E1.3
**Code Location:** `crates/agileplus-domain/src/domain/work_package/`
**Repository:** AgilePlus
**Status:** Active
**Test Traces:** Work package entity tests

#### FR-AGILE-006: CLI Feature Commands
**Requirement:** System SHALL implement `feature create`, `feature list`, `feature transition` CLI commands with proper state validation and audit logging.
**Traces To:** E1.1
**Code Location:** `crates/agileplus-cli/`
**Repository:** AgilePlus
**Status:** Active
**Test Traces:** CLI integration tests

#### FR-AGILE-007: REST API Endpoints
**Requirement:** System SHALL expose REST API endpoints for feature CRUD (Create, Read, Update, Delete), state transitions, and audit trail queries.
**Traces To:** E1.1
**Code Location:** `crates/agileplus-api/`
**Repository:** AgilePlus
**Status:** Active
**Test Traces:** API endpoint tests

#### FR-AGILE-008: Event Bus Integration
**Requirement:** System SHALL publish domain events to NATS JetStream when features or work packages change state with subject pattern `agileplus.features.{feature_id}.{event_type}`.
**Traces To:** E7.3
**Code Location:** `crates/agileplus-nats/`, `crates/agileplus-events/`
**Repository:** AgilePlus
**Status:** Active
**Test Traces:** Event publishing tests

#### FR-AGILE-009: SQLite Persistence
**Requirement:** System SHALL persist domain entities to SQLite via repository port pattern; all schema migrations SHALL be embedded and applied automatically on startup.
**Traces To:** E9.1
**Code Location:** `crates/agileplus-sqlite/`
**Repository:** AgilePlus
**Status:** Active
**Test Traces:** Persistence layer tests

#### FR-AGILE-010: Import & Export
**Requirement:** System SHALL implement idempotent import of features and work packages from manifest files with comprehensive error reporting.
**Traces To:** E7.4
**Code Location:** `crates/agileplus-import/`
**Repository:** AgilePlus
**Status:** Active
**Test Traces:** Import validation tests

---

### FR-HELIOS: heliosCLI

#### FR-HELIOS-001: Multi-Backend Agent Dispatch
**Requirement:** The `helios` binary SHALL dispatch to any registered AI backend via `--model` and `--provider` flags, with alias expansion for `oss-provider` and `local-provider`.
**Traces To:** E1.1
**Code Location:** `src/commands/`
**Repository:** heliosCLI
**Status:** Active
**Test Traces:** Backend dispatch tests

#### FR-HELIOS-002: Interactive TUI Launch
**Requirement:** When stdin is a TTY and no subcommand is given, the binary SHALL launch the `helios-tui` interactive session.
**Traces To:** E1.2
**Code Location:** `src/main.rs`
**Repository:** heliosCLI
**Status:** Active
**Test Traces:** TUI integration tests

#### FR-HELIOS-003: Batch Non-Interactive Mode
**Requirement:** When stdin is not a TTY, the binary SHALL process piped input without interactive prompts and exit with code 0 on success, non-zero on error.
**Traces To:** E1.3
**Code Location:** `src/commands/`
**Repository:** heliosCLI
**Status:** Active
**Test Traces:** Batch mode tests

#### FR-HELIOS-004: Session Resume
**Requirement:** `helios resume <session-id>` SHALL restore prior session state from `$HELIOS_HOME/sessions/`.
**Traces To:** E1.2
**Code Location:** `src/commands/session.rs`
**Repository:** heliosCLI
**Status:** Active
**Test Traces:** Session persistence tests

#### FR-HELIOS-005: Session Fork
**Requirement:** `helios fork` SHALL create new session branched from existing session, inheriting conversation history up to fork point.
**Traces To:** E1.2
**Code Location:** `src/commands/session.rs`
**Repository:** heliosCLI
**Status:** Active
**Test Traces:** Session branching tests

#### FR-HELIOS-006: Patch Application
**Requirement:** `helios apply <patch-file>` SHALL apply unified diff to working tree via the `ApplyCommand`.
**Traces To:** E1.4
**Code Location:** `src/commands/apply.rs`
**Repository:** heliosCLI
**Status:** Active
**Test Traces:** Patch application tests

---

### FR-WAVE: Agent Wave (agent-wave)

#### FR-WAVE-001: Agent Orchestration
**Requirement:** System SHALL provide agent orchestration infrastructure for launching, monitoring, and coordinating parallel agents.
**Traces To:** E6.1
**Code Location:** `agent-wave/`
**Repository:** agent-wave
**Status:** Active
**Test Traces:** Agent coordination tests

#### FR-WAVE-002: Federation Protocol
**Requirement:** System SHALL implement federation protocol for cross-repo agent communication and task delegation.
**Traces To:** E11.2
**Code Location:** `agent-wave/protocols/`
**Repository:** agent-wave
**Status:** Planned
**Test Traces:** TBD

---

### FR-API: AgentAPI++ (agentapi-plusplus)

#### FR-API-001: API Gateway
**Requirement:** System SHALL provide centralized API gateway for routing requests to backend services with authentication and rate limiting.
**Traces To:** E5.5
**Code Location:** `api/gateway/`
**Repository:** agentapi-plusplus
**Status:** Active
**Test Traces:** Gateway routing tests

#### FR-API-002: Service Discovery
**Requirement:** System SHALL implement service discovery for dynamic service registration and load balancing.
**Traces To:** E5.5
**Code Location:** `api/discovery/`
**Repository:** agentapi-plusplus
**Status:** Planned
**Test Traces:** TBD

---

### FR-THEGENT: Platforms/Thegent

#### FR-THEGENT-001: Agentic Execution Runtime
**Requirement:** System SHALL provide execution runtime for agentic workflows with task scheduling and monitoring.
**Traces To:** E1.1
**Code Location:** `platforms/thegent/`
**Repository:** platforms/thegent
**Status:** Active
**Test Traces:** Runtime execution tests

#### FR-THEGENT-002: Workflow Orchestration
**Requirement:** System SHALL orchestrate multi-step workflows with parallel execution, conditional branching, and error recovery.
**Traces To:** E6.1
**Code Location:** `platforms/thegent/workflow/`
**Repository:** platforms/thegent
**Status:** Active
**Test Traces:** Workflow execution tests

---

## Cross-Repo Dependency Matrix

| From | To | Dependency Type | Status |
|------|----|-----------------|----|
| FR-AGILE-* | FR-INFRA-* | Shared crates (error-core, config-core, health) | Active |
| FR-HELIOS-* | FR-INFRA-* | Shared crates | Active |
| FR-WAVE-* | FR-AGILE-*, FR-HELIOS-* | Agent coordination | Active |
| FR-THEGENT-* | FR-WAVE-* | Agent federation | Planned |
| FR-API-* | FR-AGILE-* | Domain models | Active |

---

## Validation Rules

### Uniqueness
All FR IDs MUST be globally unique across all repositories. Format: `FR-<REPO>-<NNN>` where NNN is zero-padded to 3 digits.

### Traceability
Every FR MUST:
1. Have a unique ID in the format `FR-<REPO>-<NNN>`
2. Contain a SHALL or MUST statement (clear, testable requirement)
3. Reference an epic (`Traces To` field)
4. Have a code location or be marked as Planned
5. List test traces or acceptance criteria

### Versioning
This document uses semantic versioning. Changes:
- **MAJOR (3.x)**: New repos added or repos removed from registry
- **MINOR (x.1)**: New FRs added, status changes
- **PATCH (x.x.1)**: Clarifications, link updates, formatting

---

## Agent Workflow for Spec Branch

### For Implementing Agents

1. **Pull latest specs/main:**
   ```bash
   git checkout specs/main
   git pull origin specs/main
   ```

2. **Create agent spec branch:**
   ```bash
   git checkout -b specs/agent-<your-id>-<feature>
   ```

3. **Edit FUNCTIONAL_REQUIREMENTS.md:**
   - Add new FRs with unique IDs
   - Update existing FRs status if implementing
   - Ensure all changes include repo origin field

4. **Validate and commit:**
   ```bash
   cargo test --workspace            # Run all tests
   git add FUNCTIONAL_REQUIREMENTS.md
   git commit -m "specs: add FR-<REPO>-NNN for <feature>"
   ```

5. **Push and create spec PR:**
   ```bash
   git push origin specs/agent-<your-id>-<feature>
   gh pr create --title "specs: <feature>" --body "<description>"
   ```

6. **Merge to specs/main (async):**
   - Spec PRs merge independently of code PRs on `main`
   - No blocking relationship between specs/main and main
   - Spec validation CI runs on specs/main merges

---

## CI/CD Validation

### On specs/main commits (automatic)

```bash
# Validate FR uniqueness across all repos
spec-verifier validate --master-file FUNCTIONAL_REQUIREMENTS.md

# Check for broken traceability
spec-verifier trace --check-coverage

# Verify test mappings
spec-verifier tests --verify-all
```

### On specs/agent-* PRs (automatic)

1. Check no duplicate FR IDs introduced
2. Validate new FRs have code locations or are marked Planned
3. Verify Epic traces reference valid PRD epics
4. Run full test suite to ensure FR tests exist

---

## Related Documents

| Document | Location | Purpose |
|----------|----------|---------|
| SPEC_BRANCH_WORKFLOW.md | `/docs/reference/` | Agent-facing spec branch workflow guide |
| PRD.md | Root of each repo | High-level product requirements |
| ADR.md | `/docs/adr/` | Architecture decision records |
| PLAN.md | Root of each repo | Implementation plans |
| USER_JOURNEYS.md | Root of each repo | End-user interaction flows |

---

**Last Updated:** 2026-03-30
**Maintained By:** Architecture team (phenotype-infrakit, AgilePlus, heliosCLI owners)
**Next Review:** 2026-04-30


#### FR-PHENOSDK-001: LLM Contract Extraction

**Requirement:** System SHALL extract LLM integration contracts from phenotype-SDK codebase into standalone phenosdk-llm crate with canonical trait interfaces.
**Traces To:** E8.1
**Code Location:** `crates/phenosdk-llm/src/lib.rs`
**Repository:** phenotype-infrakit
**Status:** Planned
**Test Traces:** `crates/phenosdk-llm/tests/llm_contract_tests.rs`

