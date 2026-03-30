# Phase 2 Specifications & Work Plan — 2026-03-30

## Overview

Phase 2 encompasses consolidating scattered Phenotype projects and establishing unified workspace governance.

**Sub-Phases:**
- **2A: Consolidation** — Move active projects from `~/Repos/` to `/repos/`
- **2B: Foundation** — Fix workspace compilation, missing exports, crate structure
- **2C: Monitoring Unblock** — Resolve router compilation, publish API contracts

**Total Effort:** 8-10 weeks | **Parallel Streams:** 3-5 agents

---

## Phase 2A: Consolidate Scattered Projects

**Duration:** 1-2 weeks | **Priority:** High | **Blocking:** Phase 2B

### Objective
Move all active Phenotype projects from `~/Repos/` to canonical `/repos/` monorepo.

### Projects to Consolidate

| Project | Current | Target | Status | Size |
|---------|---------|--------|--------|------|
| agent-wave | ~/Repos/ | /repos/ | ✅ Ready | 3.3 MB |
| clipproxyapi-plusplus | ~/Repos/ | /repos/ | ✅ Active | 25+ MB |
| heliosCLI | ~/Repos/ | /repos/ | ✅ Active | 100+ MB |
| phenotype-design | ~/Repos/ | /repos/ | ✅ Active | 10+ MB |
| phenotype-shared | ~/Repos/ | /repos/ | ✅ Active | 15+ MB |

### Work Packages

**WP 2A-01:** Audit & Plan Repository Moves
- Assess dependencies, submodules, CI/CD configs
- Output: Consolidation strategy document

**WP 2A-02:** Execute Repository Moves
- Clone repos to `/repos/`
- Update git remotes
- Verify CI/CD functional

**WP 2A-03:** Archive Legacy/IDE Projects
- Categorize stale projects
- Move to `.archive/legacy-repos/`

**WP 2A-04:** Update Root Governance
- Update CLAUDE.md, AGENTS.md
- Configure monorepo tooling if needed

---

## Phase 2B: Foundation (Workspace Repair)

**Duration:** 2-3 weeks | **Priority:** Critical | **Depends-On:** Phase 2A

### Objective
Repair workspace compilation and implement missing infrastructure.

### Work Packages

**WP 2B-01:** Fix Root Cargo.toml Issues (CRITICAL)
- Add CoreError export to phenotype-error-core
- Remove phenotype-git-core from exclude list
- Delete missing crates (agileplus-api-types, agileplus-domain)
- Verify: `cargo check --all` passes
- Effort: <1 day

**WP 2B-02:** Clarify Excluded Crates
- Decide: Include, Archive, or Delete for each of 14 crates
- Document decisions in `Phase-2B-excluded-crates-decision.md`
- Effort: 1-2 days

**WP 2B-03:** TypeScript Tooling for agent-wave
- Add tsconfig.json, eslint, prettier configs
- Setup pre-commit hooks
- Effort: 1 day

**WP 2B-04:** Add Missing Workspace Dependencies
- Audit for `workspace = true` references without root declarations
- Add missing entries (once_cell, figment, etc.)
- Effort: 1 day

**WP 2B-05:** Implement Error Consolidation (Phase 1 Extension)
- Expand phenotype-error-core with all Phenotype error types
- Implement From/Into traits
- Add tests and docs
- Effort: 2 days

---

## Phase 2C: Monitoring Unblock + API Ready

**Duration:** 1-2 weeks | **Priority:** High | **Depends-On:** Phase 2B (partial)

### Objective
Unblock phenotype-router-monitor compilation and publish API contracts.

### Work Packages

**WP 2C-01:** Fix phenotype-router-monitor Build
- Resolve compilation errors
- Verify tests pass
- Effort: 1-2 days

**WP 2C-02:** Router Health Check Integration
- Implement HealthChecker trait in router
- Setup metrics collection
- Effort: 1 day

**WP 2C-03:** Document ClipProxy API Contracts
- Generate OpenAPI/Swagger spec
- Define task submission/result callback formats
- Effort: 1 day

**WP 2C-04:** Integration Test Harness
- Create test fixtures (mock agents, routes)
- Setup end-to-end test suite
- Effort: 2 days

---

## Dependency Graph

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

---

## Success Criteria

### By End of Phase 2A
- ✅ All 5 projects in `/repos/`
- ✅ CI/CD pipelines functional
- ✅ Legacy projects archived
- ✅ Root docs updated

### By End of Phase 2B
- ✅ Workspace compiles (zero errors)
- ✅ All crate exports documented
- ✅ TypeScript tooling configured
- ✅ Dependencies consistent

### By End of Phase 2C
- ✅ Router compiles + tests pass
- ✅ API contracts published
- ✅ Integration tests functional
- ✅ Ready for Phase 3

---

## AgilePlus Specs (To Create)

### Spec: phase2a-consolidation
Title: Phase 2A: Consolidate Scattered Projects
WPs: 2A-01, 2A-02, 2A-03, 2A-04

### Spec: phase2b-foundation
Title: Phase 2B: Foundation (Workspace Repair)
WPs: 2B-01, 2B-02, 2B-03, 2B-04, 2B-05

### Spec: phase2c-monitoring
Title: Phase 2C: Monitoring Unblock + API Ready
WPs: 2C-01, 2C-02, 2C-03, 2C-04

---

## Metadata

- **Created:** 2026-03-30
- **Estimated Duration:** 8-10 weeks
- **Parallel Streams:** 3-5 agents
- **Total Work Packages:** 13
- **Critical Blockers:** 1 (WP 2B-01)
