# Phase 1 Execution Plan — Spec Completeness to 95/100

**Date:** 2026-03-29
**Coordinator:** phenosdk-fixer
**Status:** Ready for Team-Lead Assignment
**Target Completion:** 2026-04-05 (1 sprint)

---

## Executive Summary

Gap-sweep audit complete. Spec completeness ecosystem score: **92/100** (upgraded from 78/100 initial estimate).

**Only 1 remaining Tier 1 blocker:** Create `platforms/thegent/USER_JOURNEYS.md` to reach **95/100 target**.

All other Phase 1 items are either complete or deferred to Phase 2 (Tier 2 medium-term work).

---

## Current Spec State (Actual, 2026-03-29)

### phenotype-infrakit — COMPLETE ✅
| Spec | Lines | Status | Score |
|------|-------|--------|-------|
| PRD.md | 184 | ✅ Substantive | 95/100 |
| ADR.md | 144 | ✅ Substantive | |
| FUNCTIONAL_REQUIREMENTS.md | 180 | ✅ Substantive | |
| PLAN.md | 25 | ⚠️ Minimal | |
| USER_JOURNEYS.md | 262 | ✅ Substantive | |

**Status:** L5 Complete — All specs present, substantive, traceable. Minimal PLAN is acceptable for mature project.

---

### heliosCLI — COMPLETE ✅
| Spec | Lines | Status | Score |
|------|-------|--------|-------|
| PRD.md | 217 | ✅ Substantive | 95/100 |
| ADR.md | 144 | ✅ Substantive | |
| FUNCTIONAL_REQUIREMENTS.md | 227 | ✅ Substantive | |
| PLAN.md | 61 | ✅ Substantive | |
| USER_JOURNEYS.md | 449 | ✅ Substantive (was stub, now complete) | |

**Status:** L5 Complete — Upgraded from L4 (80/100). All specs substantive with full FR↔UJ traceability.

**Note:** Previous audit report stated USER_JOURNEYS was "3-line stub" — this was stale. Actual file contains 10 detailed journeys (UJ-001 through UJ-010) with ASCII flows, actor definitions, and metrics.

---

### platforms/thegent — PARTIAL+ ⚠️
| Spec | Lines | Status | Score |
|------|-------|--------|-------|
| PRD.md | 65 | ⚠️ Minimal | 85/100 |
| ADR.md | 447 | ✅ Substantive (NEW - was missing) | |
| FUNCTIONAL_REQUIREMENTS.md | 704 | ✅ Substantive | |
| PLAN.md | 482 | ✅ Substantive (NEW - was missing) | |
| USER_JOURNEYS.md | 0 | ❌ **MISSING** | |

**Status:** L3→L4 upgraded — ADR and PLAN now present. Only USER_JOURNEYS missing to complete Phase 1.

**Key Insight:** platforms/thegent had more substantive specs than earlier audits suggested. ADR covers multi-agent coordination, memory systems, and architecture decisions comprehensively (447 lines). PLAN contains detailed phase-based WBS with DAG dependencies (482 lines).

---

## Phase 1 Tier 1 Blocker (Remaining)

### Task #31: Create platforms/thegent/USER_JOURNEYS.md

**Scope:**
- Document 5-7 user actor journeys through thegent multi-agent orchestration workflows
- Map to existing FUNCTIONAL_REQUIREMENTS.md (704 lines, already comprehensive)
- User actors: HAX agent operators, researchers, platform engineers, system integrators
- Include ASCII flow diagrams for each journey
- Full FR↔UJ traceability matrix (100% coverage of all FRs)

**Reference Templates:**
- `heliosCLI/USER_JOURNEYS.md` (449 lines) — CLI/interaction pattern
- `phenotype-infrakit/USER_JOURNEYS.md` (262 lines) — system-level pattern
- Both follow: UJ-NNN header, Actor/Goal/Preconditions/Flow/Involved FRs/Metrics structure

**Expected Output:**
- platforms/thegent/USER_JOURNEYS.md (~300-400 lines substantive)
- Journey Metrics table (5-7 journeys × columns: UJ ID, Actor, Frequency, Priority, Status)
- Traceability Matrix (all 20+ FRs covered by ≥1 journey, 100% coverage target)
- Sign-off section with document owner and next review date

**Effort:** 1 sprint (~2 hours direct writing)

**Result on Completion:**
- platforms/thegent moves from L4 (85/100) → L5 (95/100)
- Ecosystem score: 92/100 → 95/100 ✅
- Phase 1 COMPLETE: All 3 canonical repos at L5

**Acceptance Criteria:**
- ✅ USER_JOURNEYS.md exists at platforms/thegent/USER_JOURNEYS.md
- ✅ ≥300 lines substantive content (excluding metadata)
- ✅ 5-7 end-to-end journeys documented with ASCII flows
- ✅ 100% FR coverage (every FR-* in FUNCTIONAL_REQUIREMENTS.md traced to ≥1 UJ)
- ✅ Journey Metrics table present
- ✅ FR↔UJ Traceability Matrix shows 100% coverage
- ✅ Follows same structure/formatting as heliosCLI and phenotype-infrakit templates

---

## Phase 1 Tier 2 Items (Deferred to Phase 2)

These are non-blocking but should follow thegent USER_JOURNEYS:

| Priority | Issue | Repo | Action | Timeline | ROI |
|----------|-------|------|--------|----------|-----|
| P2 | PRD expansion | thegent | Expand from 65→150+ lines with feature epics | Sprint 2 | Medium |
| P2 | PLAN expansion | phenotype-infrakit | Expand from 25→100+ lines with timeline | Sprint 2 | Low |
| P3 | Cross-repo traceability | ALL | Create TRACEABILITY_MAP.md linking shared patterns | Sprints 3-4 | High (ecosystem coherence) |

---

## Blockers Still Awaiting Team-Lead Decision

Four decisions remain unresolved from shelf audit phase:

1. **AgilePlus repo location clarity**
   - Question: Is canonical AgilePlus at `apps/AgilePlus` or `platforms/agileplus`?
   - Impact: Affects PR targeting for shelf reorganization work
   - Decision Required: Yes/No on creating apps/AgilePlus → platforms/agileplus redirect PR

2. **agent-wave branch merge strategy**
   - Current state: tooling/agent-wave on `main`, 1 modified (package.json), 1 untracked (scripts/enforce-bun.sh)
   - Question: Merge PR or direct commit?
   - Impact: Determines rollout strategy for bun enforcement script

3. **platforms/thegent branch resolution**
   - Current state: On chore/sync-docs-security-deps branch, 27 new files
   - Question: Merge to main, or reset to main and stash?
   - Impact: Determines whether security-deps sync is incorporated into canonical

4. **Task #5 thegent path clarification**
   - Question: Which thegent is canonical for task assignments — platforms/thegent, src/thegent, or apps/thegent?
   - Impact: Determines which repo receives Phase 1 and Phase 2 work

---

## Execution Plan: How to Reach 95/100

### Week 1 (Sprint 1) — Current

**Task:** Create platforms/thegent/USER_JOURNEYS.md (Task #31)

**Steps:**
1. Analyze platforms/thegent/FUNCTIONAL_REQUIREMENTS.md (704 lines)
   - Extract major feature clusters: agent coordination, routing, memory, observability
   - Identify user actor types: operators, researchers, integrators, platform engineers

2. Design 5-7 journeys mapping to FR clusters
   - UJ-001: Operator Orchestrates Multi-Agent Task
   - UJ-002: Researcher Explores Agent Decision Trees
   - UJ-003: Platform Engineer Configures Agent Routing
   - UJ-004: System Integrator Deploys thegent Cluster
   - UJ-005: Memory System Persistence & Recovery
   - UJ-006: Observability & Agent Activity Monitoring
   - UJ-007: Security Boundary Validation

3. Write each journey section
   - Actor definition, goal, preconditions
   - ASCII flow diagram (following heliosCLI/phenotype-infrakit patterns)
   - Involved FRs (cross-reference to FUNCTIONAL_REQUIREMENTS.md)
   - Expected results

4. Create metrics table
   - Journey ID, Actor, Frequency (Daily/Weekly/Monthly), Priority (P0/P1/P2), Status

5. Build FR↔UJ traceability matrix
   - Rows: All FRs from FUNCTIONAL_REQUIREMENTS.md
   - Columns: Covered by UJ-001 through UJ-007?
   - Target: 100% coverage (no orphan FRs)

6. Add sign-off section
   - Document owner, version, next review date
   - Approval status

**Deliverable:** platforms/thegent/USER_JOURNEYS.md (300-400 lines)

**Success Metric:** All FRs traced, no gaps, follows template structure exactly

---

### Week 2+ (Phase 2) — Deferred

**Phase 2a — Expand thegent PRD (Sprint 2)**
- Expand platforms/thegent/PRD.md from 65→150+ lines
- Add feature epics E1-E5 with acceptance criteria
- Score: 85/100 → 88/100

**Phase 2b — Create TRACEABILITY_MAP.md (Sprints 3-4)**
- Cross-project traceability linking:
  - phenotype-infrakit policies → heliosCLI commands → thegent agents
  - Shared FR patterns (authentication, caching, event sourcing)
  - Code entity mapping (where each FR is implemented)
- Score: 88/100 → 92/100+ (ecosystem coherence)

---

## Updated Ecosystem Score Progression

| Milestone | Date | Score | Status |
|-----------|------|-------|--------|
| Initial gap-sweep audit | 2026-03-29 | 78/100 | ⚠️ Underestimated (stale report) |
| Actual current state | 2026-03-29 | 92/100 | ✅ Upgraded after audit reconciliation |
| After Phase 1 (thegent UJ) | 2026-04-05 | **95/100** | 🎯 Target |
| After Phase 2a (thegent PRD) | 2026-04-12 | 98/100 | |
| After Phase 2b (Traceability) | 2026-04-26 | **100/100** | 🏁 Full Completion |

---

## Recommendation for Team-Lead

**Immediate Next Step:**
1. Resolve 4 blocker decisions (listed above)
2. Assign Task #31 (thegent USER_JOURNEYS) to Phase 1 executor
3. Set target completion: 2026-04-05 (1 sprint)

**Team Assignment:**
- Task #31 executor (1 role) — Writer/analyst
  - Skills: Technical writing, FR analysis, traceability mapping
  - Effort: ~2 hours focused work
  - Leverage: heliosCLI and phenotype-infrakit templates as reference

**Post-Sprint-1:**
- Review Phase 1 completion (Tier 1 blocker done)
- Decide on Phase 2 scope and timeline (Tier 2 items)
- 4 blocker decisions can be resolved in parallel while Phase 1 executes

---

## Appendix: Audit Report Reconciliation

The original spec completeness audit (prepared by agileplus-fixer, 2026-03-29) contained outdated findings that were superseded by repository updates between audit time and execution time:

| Repo | Audit Finding | Actual State | Reason |
|------|---------------|--------------|--------|
| heliosCLI | USER_JOURNEYS is 3-line stub (L4 80/100) | USER_JOURNEYS is 449 lines complete (L5 95/100) | Repository scaffold baseline committed after audit ran |
| platforms/thegent | ADR missing (L3 60/100) | ADR.md 447 lines present | Post-audit commit added comprehensive ADR.md |
| platforms/thegent | PLAN missing | PLAN.md 482 lines present | Post-audit commit added detailed PLAN.md with DAG |

**Resolution:** Revalidated all repos as of 2026-03-29 14:00 UTC. Current state reflects actual files on disk. Phase 1 plan updated to reflect real blockers only.

---

**Document Owner:** phenosdk-fixer
**Prepared:** 2026-03-29
**Status:** Ready for Team-Lead Assignment
**Next Review:** 2026-04-05 (post-Phase-1)
