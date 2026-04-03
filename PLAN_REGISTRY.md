# PLAN Registry — Implementation Plans Index

**Version:** 1.0
**Status:** Active
**Updated:** 2026-04-01
**Branch:** `specs/main`

---

## Overview

This registry tracks all multi-phase implementation plans across the Phenotype polyrepo, with phase structure, work packages, dependencies, and progress tracking.

---

## Master Plan Index

### phenotype-infrakit PLAN.md (v2.0)

**Total Duration:** 8 weeks (Phases 1-4)
**Status:** ✅ Phase 1-2 Complete, Phase 3 In Progress

| Phase | Name | Duration | WPs | Status | Completion |
|-------|------|----------|-----|--------|-----------|
| **Phase 1** | Foundation Crates | Weeks 1-2 | 7 WPs | ✅ Complete | 2026-03-30 |
| **Phase 2** | Advanced Patterns | Weeks 3-4 | 5 WPs | ✅ Complete | 2026-03-31 |
| **Phase 3** | Performance Optimization | Weeks 5-6 | 4 WPs | ⏳ In Progress | Est. 2026-04-15 |
| **Phase 4** | Enterprise & Ecosystem | Weeks 7-8 | 6 WPs | 🔧 Planned | Est. 2026-05-01 |

**Key Milestones:**
- 2026-03-25: Phase 1 kicked off
- 2026-03-30: Phase 1 complete (7 crates deployed)
- 2026-03-31: Phase 2 complete (patterns + contracts)
- 2026-04-15: Phase 3 target (build optimization)
- 2026-05-01: Phase 4 target (all infrastructure stable)

**Dependencies:**
- Phase 1 → Phase 2 (sequential)
- Phase 2 → Phase 3 (sequential)
- Phase 3 → Phase 4 (sequential)

---

### AgilePlus PLAN.md (v1.5)

**Total Duration:** 8 weeks (Phases 1-3)
**Status:** ✅ Phase 1-2 Complete, Phase 3 In Progress

| Phase | Name | Duration | WPs | Status |
|-------|------|----------|-----|--------|
| **Phase 1** | Core Engine (24 FRs) | Weeks 1-3 | 12 WPs | ✅ Complete |
| **Phase 2** | API + CLI Layers | Weeks 4-5 | 8 WPs | ✅ Complete |
| **Phase 3** | Governance & Specs | Weeks 6-8 | 10 WPs | ⏳ In Progress |

**Key Deliverables:**
- Phase 1: Domain model, state machine, spec registry
- Phase 2: REST API, CLI commands, dashboards
- Phase 3: Spec validation, FR↔Test traceability, auto-merge

**Dependencies:**
- phenotype-infrakit Phase 1-2 → AgilePlus Phase 1
- AgilePlus Phase 1 → AgilePlus Phase 2
- AgilePlus Phase 2 → AgilePlus Phase 3

---

### platforms/thegent PLAN.md (v2.1)

**Total Duration:** 10 weeks (Phases 1-4)
**Status:** ✅ Phase 1-2 Complete, Phase 3 In Progress

| Phase | Name | Duration | WPs | Status |
|-------|------|----------|-----|--------|
| **Phase 1** | Foundation (MCP SDKs) | Weeks 1-2 | 6 WPs | ✅ Complete |
| **Phase 2** | Resilience (Circuit Breakers) | Weeks 3-4 | 7 WPs | ✅ Complete |
| **Phase 3** | Memory (Agent Session State) | Weeks 5-7 | 8 WPs | ⏳ In Progress |
| **Phase 4** | Integration (Cross-Platform) | Weeks 8-10 | 5 WPs | 🔧 Planned |

**Key Milestones:**
- Phase 1 (2026-03-25): 3 MCP SDKs (Go, Python, TS) deployed
- Phase 2 (2026-03-31): Circuit breaker + distributed tracing
- Phase 3 (2026-04-15): Agent memory + session persistence
- Phase 4 (2026-05-01): Cross-platform integration (AgilePlus ↔ heliosCLI)

---

### heliosCLI PLAN.md (v2.3)

**Total Duration:** 9 weeks (Phases 1-3+)
**Status:** ✅ Phase 1 Complete, Phase 2 In Progress

| Phase | Name | Duration | WPs | Status |
|-------|------|----------|-----|--------|
| **Phase 1** | Core CLI + Agent Harness | Weeks 1-2 | 5 WPs | ✅ Complete |
| **Phase 2** | Sandboxing & Isolation | Weeks 3-4 | 7 WPs | ⏳ In Progress |
| **Phase 3** | Plugin System | Weeks 5-6 | 6 WPs | 🔧 Planned |
| **Phase 4** | Multi-Backend Optimization | Weeks 7-9 | 4 WPs | 🔧 Planned |

**Key Deliverables:**
- Phase 1: TUI, batch mode, agent dispatch
- Phase 2: Container sandboxing, resource limits
- Phase 3: Dynamic plugin loading, hotload agents
- Phase 4: Backend abstraction (local, remote, cloud)

---

## Cross-Repository Dependency Graph

### Sequential Dependencies

```
┌─────────────────────────────────────────────────────────────┐
│ phenotype-infrakit Phase 1 (Foundation Crates)             │
│ Weeks 1-2 | 7 WPs | Status: ✅ COMPLETE                    │
└──────────────────────┬──────────────────────────────────────┘
                       │ (MUST COMPLETE FIRST)
                       ▼
┌─────────────────────────────────────────────────────────────┐
│ phenotype-infrakit Phase 2 (Patterns)                       │
│ Weeks 3-4 | 5 WPs | Status: ✅ COMPLETE                    │
└──────────────────────┬──────────────────────────────────────┘
                       │ (BLOCKING FOR ALL DOWNSTREAM)
        ┌──────────────┼──────────────┬──────────────────┐
        ▼              ▼              ▼                  ▼
    AgilePlus    platforms/     heliosCLI         agent-wave
    Phase 1      thegent        Phase 1           Phase 1
    (Start 3/25) Phase 1        (Start 3/25)     (Planned)
```

### Phase Overlap Strategy

**Can Parallelize After Foundation:**
- AgilePlus Phase 1 ↔ platforms/thegent Phase 1 (both depend on phenotype-infrakit Phase 2)
- heliosCLI Phase 1 ↔ AgilePlus Phase 1 (independent domain concerns)
- platforms/thegent Phase 2 ↔ AgilePlus Phase 2 (no blocking deps)

**Must Serialize:**
- phenotype-infrakit Phase 1 → Phase 2 (foundation before extensions)
- AgilePlus Phase 1 → Phase 2 → Phase 3 (domain → API → governance)
- heliosCLI Phase 1 → Phase 2 (core → sandboxing)

---

## Critical Path Analysis

### Longest Dependency Chain

```
phenotype-infrakit Phase 1-2 (4 weeks)
        ↓
AgilePlus Phase 1-2 (5 weeks)
        ↓
AgilePlus Phase 3 (2 weeks)
        ↓
platforms/thegent Phase 3-4 (5 weeks)

Total: 16 weeks from start to full integration
```

### Parallelizable Paths

**Path A (Parallel with critical path):**
```
phenotype-infrakit Phase 1-2 (4 weeks)
        ↓
platforms/thegent Phase 1-2 (4 weeks) [parallel to AgilePlus 1-2]
        ↓
platforms/thegent Phase 3-4 (5 weeks)
```

**Path B (Parallel with critical path):**
```
phenotype-infrakit Phase 1-2 (4 weeks)
        ↓
heliosCLI Phase 1-2 (4 weeks) [parallel to AgilePlus 1-2]
```

### Optimized Timeline

**With Parallelization:**
- Start date: 2026-03-25
- Finish date: 2026-05-20 (8 weeks, not 16)
- Critical path: phenotype-infrakit → AgilePlus Phase 1-2 → platforms/thegent Phase 3-4

---

## Phase Details

### phenotype-infrakit Phase 3: Performance Optimization

**Goal:** Reduce build times, improve runtime performance, add caching layer

| WP | Name | Effort | Status | Owner |
|----|------|--------|--------|-------|
| WP-3.1 | Tokio feature reduction | 4h | ✅ Complete | Infrastructure |
| WP-3.2 | panic = "abort" release profile | 2h | ✅ Complete | Infrastructure |
| WP-3.3 | sccache + incremental build | 6h | ✅ Complete | Infrastructure |
| WP-3.4 | Performance audit + benchmarks | 8h | ⏳ In Progress | Performance |

**Metrics:**
- Cold build: 81.2s → 45s (45% reduction)
- Incremental: 0.9s (unchanged)
- Grade: A (well-optimized crate structure)

---

## SLA & Milestones

### Week-by-Week Timeline

| Week | Date | Milestone | Status |
|------|------|-----------|--------|
| Week 1 | 2026-03-25 | phenotype-infrakit Phase 1 kick-off | ✅ Complete |
| Week 2 | 2026-03-31 | phenotype-infrakit Phase 2 complete | ✅ Complete |
| Week 3 | 2026-04-07 | AgilePlus Phase 1 complete | ⏳ On track |
| Week 4 | 2026-04-14 | AgilePlus Phase 2 complete | ⏳ On track |
| Week 5 | 2026-04-21 | platforms/thegent Phase 1-2 complete | ⏳ Planned |
| Week 6 | 2026-04-28 | heliosCLI Phase 1-2 complete | ⏳ Planned |
| Week 7 | 2026-05-05 | AgilePlus Phase 3 complete | ⏳ Planned |
| Week 8 | 2026-05-12 | platforms/thegent Phase 3 complete | ⏳ Planned |
| Week 9 | 2026-05-19 | All phases complete | ⏳ Planned |

---

## Work Package Structure

### Example: phenotype-infrakit Phase 1

```
Phase 1: Foundation Crates (Weeks 1-2)

├─ WP-1.1: Setup & Cargo Workspace (1h)
│  └─ Create workspace.toml, Cargo.lock
│
├─ WP-1.2: phenotype-event-sourcing crate (8h)
│  ├─ Append-only event store
│  ├─ SHA-256 hash chains
│  └─ Tests (100% coverage)
│
├─ WP-1.3: phenotype-cache-adapter crate (4h)
│  ├─ LRU + DashMap cache
│  ├─ TTL support
│  └─ Tests
│
├─ WP-1.4: phenotype-policy-engine crate (6h)
│  ├─ Rule-based evaluation
│  ├─ TOML configuration
│  └─ Tests
│
├─ WP-1.5: phenotype-state-machine crate (4h)
│  ├─ Generic FSM
│  ├─ Transition guards
│  └─ Tests
│
├─ WP-1.6: phenotype-contracts crate (3h)
│  ├─ Shared trait definitions
│  └─ Type definitions
│
└─ WP-1.7: Documentation + Release (2h)
   ├─ README for each crate
   ├─ CHANGELOG
   └─ v0.1.0 release tag
```

---

## Tracking & Status

### Current Status (as of 2026-04-01)

| Repo | Phase | Progress | ETA | Blockers |
|------|-------|----------|-----|----------|
| phenotype-infrakit | 1-2: Complete, 3: 50% | 87% | 2026-04-15 | None |
| AgilePlus | 1-2: Complete, 3: 25% | 75% | 2026-04-28 | None |
| platforms/thegent | 1-2: Complete, 3: 10% | 65% | 2026-05-12 | None |
| heliosCLI | 1: Complete, 2: 50% | 60% | 2026-05-05 | ADR finalization |

**Overall Completion:** 72% (Target: 100% by 2026-05-20)

---

## Related Documents

- `PLAN.md` (per-repo) — Detailed phase breakdowns
- `FUNCTIONAL_REQUIREMENTS.md` — FRs per phase
- `ADR.md` — Architecture decisions
- `USER_JOURNEYS.md` — User workflows
- `docs/reference/SSOT_PHASE1_IMPLEMENTATION_PLAN.md` — SSOT governance

---

**Registry Owner:** Project Manager
**Last Updated:** 2026-04-01
**Next Review:** 2026-04-08
