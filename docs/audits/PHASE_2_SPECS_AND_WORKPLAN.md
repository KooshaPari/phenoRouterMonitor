# Phase 2 Specifications & Work Plan (2026-03-30)

## Overview

Phase 2 encompasses consolidating scattered Phenotype projects and establishing unified workspace governance. This document defines three sub-phases:

- **Phase 2A: Consolidation** — Move active projects from `~/Repos/` to `/repos/`
- **Phase 2B: Foundation** — Fix workspace compilation, implement missing exports, standardize crate structure
- **Phase 2C: Monitoring Unblock** — Resolve phenotype-router-monitor compilation blockers and ready API layer

**Total Effort:** ~8-10 weeks (parallel work streams, 3-5 agents per stream)

---

## Phase 2A: Consolidate Scattered Projects

**Duration:** 1-2 weeks | **Priority:** High | **Blocking:** Phase 2B cannot start until complete

### Objective

Consolidate all active Phenotype projects currently scattered in `~/Repos/` into the canonical `/repos/` monorepo structure. This unifies governance, CI/CD, and dependency management.

### Scope

| Project | Current Location | Target Location | Status | Estimated Size |
|---------|------------------|-----------------|--------|-----------------|
| **agent-wave** | `~/Repos/agent-wave/` | `/repos/agent-wave/` | ✅ Ready | 3.3 MB, governance-phase |
| **cliproxyapi-plusplus** | `~/Repos/cliproxyapi-plusplus/` | `/repos/cliproxyapi-plusplus/` | ✅ Active | 25+ MB, Go/Rust |
| **heliosCLI** | `~/Repos/heliosCLI/` | `/repos/heliosCLI/` | ✅ Active | 100+ MB, multi-lang |
| **phenotype-design** | `~/Repos/phenotype-design/` | `/repos/phenotype-design/` | ✅ Active | 10+ MB, design system |
| **phenotype-shared** | `~/Repos/phenotype-shared/` | `/repos/phenotype-shared/` | ✅ Active | 15+ MB, shared utilities |

### Work Packages

#### WP 2A-01: Audit & Plan Repository Moves

**Title:** Audit each project for dependencies, submodules, CI/CD configs before move

**Acceptance Criteria:**
- [ ] All 5 projects assessed for circular dependencies
- [ ] All CI/CD pipelines documented (GitHub Actions, local scripts)
- [ ] Submodule/worktree dependencies identified
- [ ] Move strategy documented for each project

**Effort:** 2-3 agent days
**Owner:** Exploration agent

**Deliverables:**
- `docs/audits/2026-03-30-clipproxyapi-audit.md`
- `docs/audits/2026-03-30-heliosCLI-audit.md`
- `docs/audits/2026-03-30-phenotype-design-audit.md`
- `docs/audits/2026-03-30-phenotype-shared-audit.md`
- `docs/audits/2026-03-30-consolidation-strategy.md`

#### WP 2A-02: Execute Repository Moves

**Title:** Clone/symlink repos to `/repos/`; update git remotes; verify CI/CD

**Acceptance Criteria:**
- [ ] All 5 projects present at target locations
- [ ] Git remotes updated to canonical URLs
- [ ] Local CI/CD scripts functional
- [ ] No worktree or submodule conflicts

**Effort:** 3-4 agent days
**Owner:** Integration agent

**Sequence:**
1. Clone each repo: `git clone https://github.com/KooshaPari/<repo>.git /repos/<repo>/`
2. Verify no local changes: `git status --short`
3. Test builds locally: `cargo check`, `bun install`, `go build`, etc.

#### WP 2A-03: Archive Legacy/IDE Projects

**Title:** Identify and archive stale projects from `~/Repos/`

**Scope:** Archive outdated, abandoned, or IDE-specific projects to `.archive/legacy-repos/` with explanation document.

**Acceptance Criteria:**
- [ ] All non-active projects categorized
- [ ] Archive decision documented per project
- [ ] Legacy projects moved to `.archive/legacy-repos/`
- [ ] Root ~/Repos contains only active Phenotype projects

**Effort:** 1-2 agent days
**Owner:** Cleanup agent

**Archive Decision Matrix:**

| Project | Decision | Reason |
|---------|----------|--------|
| `Aloxaf`, `justyntemme`, etc. (user forks) | Archive | Not maintained; IDE artifacts |
| `polyhex-architecture` | Evaluate | Active research or stale? |
| `phenodocs` | Already in `/repos` | No action needed |
| Others | TBD | Review case-by-case |

#### WP 2A-04: Update Root Governance Documentation

**Title:** Update CLAUDE.md, AGENTS.md, Makefile to reference new locations

**Acceptance Criteria:**
- [ ] Root `/repos/CLAUDE.md` references all 5 new projects
- [ ] Root CI/CD pipelines include new projects
- [ ] Monorepo tooling (Turborepo/Nx) configured if applicable
- [ ] `.gitignore` updated to prevent embedded repo scanning issues

**Effort:** 1 agent day
**Owner:** Documentation agent

---

## Phase 2B: Foundation (Workspace Repair + Implementations)

**Duration:** 2-3 weeks | **Priority:** Critical | **Depends-On:** Phase 2A

### Objective

Repair workspace compilation errors and implement missing core infrastructure pieces identified in audits.

### Scope

1. **Fix Compilation Blockers** — Resolve CoreError export, phenotype-git-core duplication, missing workspace.dependencies
2. **Implement Missing Crate Exports** — Ensure all workspace members properly export public APIs
3. **Add Missing Implementations** — Implement TypeScript/Node.js tooling, missing error handlers, etc.

### Work Packages

#### WP 2B-01: Fix Root Cargo.toml Issues (CRITICAL)

**Title:** Resolve compilation errors blocking `cargo check`

**Blockers:**
- ❌ `phenotype-test-infra` → `phenotype-error-core::CoreError` unresolved
- ⚠️ `phenotype-git-core` listed in both members AND exclude
- ⚠️ `agileplus-api-types`, `agileplus-domain` declared but missing

**Acceptance Criteria:**
- [ ] `cargo check` passes with zero errors
- [ ] CoreError properly exported from phenotype-error-core
- [ ] phenotype-git-core conflict resolved (remove from exclude)
- [ ] Missing crates removed from Cargo.toml
- [ ] All 13 active members build cleanly
- [ ] Git commit: "fix(workspace): resolve Cargo.toml errors"

**Effort:** 0.5 agent day
**Owner:** Build engineer

**Steps:**
1. Fix `crates/phenotype-error-core/src/lib.rs` to export `CoreError`
2. Remove lines from `Cargo.toml`: agileplus-api-types, agileplus-domain, phenotype-git-core (from exclude)
3. Run `cargo check --all`
4. Verify all tests pass: `cargo test --workspace`

#### WP 2B-02: Clarify & Standardize Excluded Crates

**Title:** Decide fate of 14 excluded crates; archive obsolete ones

**Current Status:**
- 2 missing (agileplus-api-types, agileplus-domain) → Remove from Cargo.toml
- 12 existing (phenotype-crypto, phenotype-logging, etc.) → Include or archive?

**Acceptance Criteria:**
- [ ] Decision documented for each of 14 crates
- [ ] Obsolete crates archived to `.archive/excluded-crates/`
- [ ] Active crates either included in members or excluded with documented reason
- [ ] Cargo.toml cleaned up (only valid excludes remain)
- [ ] Meeting notes: "Phase-2B-excluded-crates-decision.md"

**Effort:** 1-2 agent days
**Owner:** Architecture lead

**Decision Template:**
```markdown
## Crate: phenotype-crypto

**Status:** [TODO — Evaluate]
**Size:** 5 KB
**Current Location:** crates/phenotype-crypto/
**Dependencies:** None (standalone)
**Usage:** Grep for imports

**Decision:** [INCLUDE/ARCHIVE/REMOVE]
**Rationale:** [Why this decision]
**Action:** [Include in members / Move to .archive/ / Delete]
```

#### WP 2B-03: Implement TypeScript Tooling for agent-wave

**Title:** Set up TypeScript, ESLint, Prettier, type-checking for agent-wave

**Acceptance Criteria:**
- [ ] TypeScript config (`tsconfig.json`) added
- [ ] ESLint config added with Phenotype rules
- [ ] Prettier config added and enforced
- [ ] Pre-commit hooks include TypeScript linting
- [ ] `bun run build` and `bun run check` work

**Effort:** 1 agent day
**Owner:** TypeScript engineer

**Files to Create:**
- `agent-wave/tsconfig.json`
- `agent-wave/.eslintrc.json`
- `agent-wave/.prettierrc.json`
- Update `agent-wave/package.json` with scripts

#### WP 2B-04: Add Missing Workspace Dependencies

**Title:** Add missing transitive dependencies to workspace.dependencies

**Scope:**
- Audit each crate for `workspace = true` references
- Identify dependencies not in root `workspace.dependencies`
- Add missing entries (e.g., `once_cell`, `figment`, etc.)

**Acceptance Criteria:**
- [ ] All `workspace = true` references have corresponding root declarations
- [ ] No crate uses inline version specs when workspace alternatives exist
- [ ] `cargo check` passes for all crates
- [ ] Dependency versions are consistent across workspace

**Effort:** 1 agent day
**Owner:** Build engineer

#### WP 2B-05: Implement Core Error Consolidation (Phase 1 Extension)

**Title:** Expand phenotype-error-core to include all Phenotype error types

**Current State:** phenotype-error-core exists but may be incomplete.

**Acceptance Criteria:**
- [ ] All crate-specific error types consolidated into phenotype-error-core
- [ ] From/Into traits implemented for conversions
- [ ] Error display/debug output is human-readable
- [ ] Tests for each error variant
- [ ] Documentation: `docs/architecture/error-handling-guide.md`

**Effort:** 2 agent days
**Owner:** Error architecture lead

---

## Phase 2C: Monitoring Unblock + API Ready

**Duration:** 1-2 weeks | **Priority:** High | **Depends-On:** Phase 2B (partially)

### Objective

Unblock phenotype-router-monitor compilation and ensure ClipProxy API layer is ready for integration testing.

### Work Packages

#### WP 2C-01: Fix phenotype-router-monitor Build

**Title:** Resolve all compilation errors in phenotype-router-monitor

**Current Status:** Unknown (not yet audited)

**Acceptance Criteria:**
- [ ] `cargo check` passes for phenotype-router-monitor
- [ ] All dependencies resolved (no missing crates)
- [ ] Tests pass: `cargo test --package phenotype-router-monitor`
- [ ] Documentation updated: `docs/audits/2026-03-30-router-monitor-fix.md`

**Effort:** 1-2 agent days
**Owner:** Router engineer

**Likely Blockers:**
- Dependency on Cargo.toml fixes from WP 2B-01
- Missing health check trait integration
- Missing telemetry/logging integration

#### WP 2C-02: Implement Router Health Check Integration

**Title:** Integrate phenotype-health trait into router monitoring

**Acceptance Criteria:**
- [ ] Router implements `HealthChecker` trait
- [ ] Health endpoint returns structured response
- [ ] Metrics collection integrated with phenotype-telemetry
- [ ] Tests cover all health states (healthy/degraded/unhealthy)

**Effort:** 1 agent day
**Owner:** Health/metrics engineer

#### WP 2C-03: Document ClipProxy API Contracts

**Title:** Publish API contract definitions for ClipProxy integration

**Acceptance Criteria:**
- [ ] OpenAPI/Swagger spec generated from code
- [ ] Task submission API contract documented
- [ ] Result callback format specified
- [ ] Error response schema defined
- [ ] Example requests/responses provided
- [ ] File: `docs/architecture/clipproxy-api-contract.md`

**Effort:** 1 agent day
**Owner:** API documentation lead

#### WP 2C-04: Set Up Integration Test Harness

**Title:** Create end-to-end test framework for router → ClipProxy → agent workflows

**Acceptance Criteria:**
- [ ] Test fixtures created (mock agents, routes, tasks)
- [ ] Integration test suite runs: `cargo test --features integration-tests`
- [ ] Tests cover happy path + error cases
- [ ] CI/CD includes integration tests

**Effort:** 2 agent days
**Owner:** QA engineer

---

## Cross-Phase Dependencies & Sequencing

### Dependency Graph

```
Phase 2A (Consolidation)
    ↓
Phase 2B-01 (Fix Cargo.toml) ← CRITICAL BLOCKER
    ↓
Phase 2B-02..05 (Parallel implementations)
    ↓
Phase 2C-01..04 (Monitoring unblock + API ready)
    ↓
Phase 3 (Implementation begins)
```

### Parallel Work Streams

**Stream 1: Workspace Repair** (2B)
- Agent: Build engineer
- Duration: 1-2 weeks
- Deliverable: Fully compiling workspace

**Stream 2: Project Consolidation** (2A)
- Agent: Integration engineer
- Duration: 1-2 weeks (parallel with Stream 1)
- Deliverable: All projects moved to `/repos/`

**Stream 3: Router Unblock** (2C)
- Agent: Router engineer
- Duration: 1-2 weeks (starts after 2B-01)
- Deliverable: Router-monitor compiles + API contracts published

---

## Success Criteria

### By End of Phase 2A
- [ ] All 5 projects live in `/repos/`
- [ ] CI/CD pipelines functional
- [ ] Legacy projects archived to `.archive/`
- [ ] Root governance docs updated

### By End of Phase 2B
- [ ] Workspace compiles with zero errors
- [ ] All crate exports documented
- [ ] TypeScript tooling configured (agent-wave)
- [ ] Dependency versions consistent

### By End of Phase 2C
- [ ] phenotype-router-monitor builds & tests pass
- [ ] API contracts published
- [ ] Integration test harness functional
- [ ] Ready for Phase 3 implementation

---

## AgilePlus Specifications (To Be Created)

Once these Phase 2 specs are finalized, create the following AgilePlus specs:

### Spec: phase2a-consolidation
```
Title: Phase 2A: Consolidate Scattered Projects
Description: Move agent-wave, clipproxyapi-plusplus, heliosCLI, phenotype-design, phenotype-shared from ~/Repos to /repos

Work Packages:
  WP01: Audit & Plan Repository Moves
  WP02: Execute Repository Moves
  WP03: Archive Legacy/IDE Projects
  WP04: Update Root Governance Documentation
```

### Spec: phase2b-foundation
```
Title: Phase 2B: Foundation (Workspace Repair + Implementations)
Description: Fix Cargo.toml, implement missing exports, standardize structure

Work Packages:
  WP01: Fix Root Cargo.toml Issues (CRITICAL)
  WP02: Clarify & Standardize Excluded Crates
  WP03: Implement TypeScript Tooling for agent-wave
  WP04: Add Missing Workspace Dependencies
  WP05: Implement Core Error Consolidation
```

### Spec: phase2c-monitoring
```
Title: Phase 2C: Monitoring Unblock + API Ready
Description: Resolve phenotype-router-monitor build, publish API contracts

Work Packages:
  WP01: Fix phenotype-router-monitor Build
  WP02: Implement Router Health Check Integration
  WP03: Document ClipProxy API Contracts
  WP04: Set Up Integration Test Harness
```

---

## File References

### Phase 2 Audit Documents (Generated 2026-03-30)
- `/repos/docs/audits/2026-03-30-agent-wave-audit.md` — Agent Wave architecture review
- `/repos/docs/audits/2026-03-30-root-workspace-audit.md` — Workspace compilation issues & cleanup

### Phase 2 Planning Documents (This File)
- `/repos/docs/audits/PHASE_2_SPECS_AND_WORKPLAN.md` — Comprehensive Phase 2 plan

### Future Audit Documents (To Be Generated)
- `docs/audits/2026-03-30-clipproxyapi-audit.md` (WP 2A-01)
- `docs/audits/2026-03-30-heliosCLI-audit.md` (WP 2A-01)
- `docs/audits/2026-03-30-phenotype-design-audit.md` (WP 2A-01)
- `docs/audits/2026-03-30-phenotype-shared-audit.md` (WP 2A-01)
- `docs/audits/CONSOLIDATION_STRATEGY.md` (WP 2A-01)

---

## Metadata

- **Plan Created:** 2026-03-30
- **Planner:** Claude Code (Haiku 4.5)
- **Total Estimated Duration:** 8-10 weeks
- **Parallel Work Streams:** 3-5 agents
- **Total Work Packages:** 13
- **Critical Blockers:** 1 (Fix Cargo.toml in WP 2B-01)

---

## Next Steps

1. **Immediate:** Create AgilePlus specs (phase2a, phase2b, phase2c) with work packages
2. **Immediate:** Assign agents to parallel work streams
3. **Week 1:** Complete Phase 2A consolidation audits
4. **Week 2:** Execute Phase 2B-01 fix (unblock compilation)
5. **Week 2-3:** Execute remaining Phase 2B and 2C work in parallel

---

## Questions for Product Lead

1. Should excluded crates (phenotype-crypto, phenotype-logging, etc.) be included in main workspace or archived?
2. Timeline pressure: Is Phase 2 completion needed by specific date?
3. Should monorepo tooling (Turborepo, Nx) be added, or stick with cargo workspace?
4. Who owns each work stream? (Build, Integration, Router, API, QA)
