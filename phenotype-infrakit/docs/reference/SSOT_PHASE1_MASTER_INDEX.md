# Polyrepo SSOT Phase 1 — Master Index & Navigation

**Status:** Complete (All 5 Documents Delivered)
**Created:** 2026-03-31
**Total Lines:** 4,920+ lines of documentation
**Audience:** Project sponsors, team leads, implementers, agents

---

## Quick Navigation

**For executives:** Start here → [Executive Summary](#executive-summary)
**For team leads:** Start here → [Work Packages](#work-packages)
**For engineers:** Start here → [Implementation Plan](#implementation-plan)
**For agents:** Start here → [Agent Workflow](#agent-workflow)
**For architects:** Start here → [Auto-Merge Service](#auto-merge-service)

---

## Document Overview

### 1. Executive Summary
**File:** `docs/reference/SSOT_PHASE1_EXECUTIVE_SUMMARY.md`
**Length:** 14 KB (250 lines)
**Audience:** Executives, project sponsors, team leads
**Contains:**
- What we're building (specs governance system)
- 3 core components overview
- 5 work packages summary
- Success metrics & ROI analysis
- Risk assessment
- Next steps & timeline

**Read time:** 10 minutes
**Key takeaway:** Phase 1 investment is $8,000, ROI 1,250% in first month

---

### 2. Detailed Implementation Plan
**File:** `docs/reference/SSOT_PHASE1_IMPLEMENTATION_PLAN.md`
**Length:** 31 KB (550 lines)
**Audience:** Project leads, engineers (all disciplines)
**Contains:**
- Week 1 breakdown (5 days × 8h)
- Week 2 breakdown (5 days × 8h)
- Day-by-day tasks with dependencies
- Task 1.1-2.6 detailed specifications
- Critical path analysis
- Risk mitigation strategies
- Success criteria

**Read time:** 30 minutes
**Key takeaway:** 6 major tasks across 10 days, clear execution path

**Task Breakdown:**
| Task | Effort | Owner | Days |
|------|--------|-------|------|
| 1.1: Branch Protection | 4h | DevOps | 1-2 |
| 1.2: Specs Registry | 6h | Spec Coordinator | 2-3 |
| 1.3: Pre-Push Hooks | 4h | QA | 1-2 |
| 1.4: CI Validation | 6h | CI Engineer | 1-2 |
| 1.5: Agent Onboarding | 8h | Agent Lead | 3-4 |
| 1.6: Auto-Merge Design | 8h | Architect | 3-4 |
| 2.1: Merge Orchestrator | 10h | Infra | 6-8 |
| 2.2: GHA Workflows | 6h | DevOps | 6-7 |
| 2.3: Agent Workflows | 8h | Coordinator | 7-8 |
| 2.4: Documentation | 6h | Tech Writer | 8-9 |
| 2.5: Monitoring | 8h | Platform | 9-10 |
| 2.6: Deployment & QA | 10h | Release Mgr | 9-10 |

---

### 3. Agent Workflow Guide
**File:** `docs/reference/SSOT_PHASE1_AGENT_WORKFLOW.md`
**Length:** 21 KB (400 lines)
**Audience:** All agents, team leads
**Contains:**
- Agent branching pattern (`specs/agent-<name>-<task>`)
- Commit message format with 4+ examples
- Step-by-step workflow (create, edit, validate, push)
- Conflict handling (3 scenarios with solutions)
- FR↔Test traceability (how to add traces)
- Pre-push validation checklist
- Auto-merge triggers & timeline
- PR template for spec reconciliation
- 3 complete workflows (single FR, multiple specs, conflict resolution)
- Troubleshooting guide (10+ issues)
- Role-specific quick references

**Read time:** 20 minutes (once), reference repeatedly
**Key takeaway:** Clear, step-by-step process agents follow for every spec change

**Example Workflows:**
1. **Workflow A:** Agent updates single FR (4 commits shown)
2. **Workflow B:** Agent adds multiple specs (6 commits shown)
3. **Workflow C:** Agent resolves merge conflict (step-by-step)

---

### 4. Auto-Merge Service Architecture
**File:** `docs/reference/AUTO_MERGE_SERVICE_ARCHITECTURE.md`
**Length:** 17 KB (350 lines)
**Audience:** Architects, infrastructure engineers, senior developers
**Contains:**
- Component architecture (5 pieces)
- Service specification (Rust implementation)
- Data flow sequences (3 scenarios)
- Error handling & recovery
- Monitoring & observability
- Security considerations
- Performance characteristics
- Deployment checklist

**Read time:** 25 minutes
**Key takeaway:** Detailed technical design of batch-merger service (implementation-ready)

**Components:**
1. **Batch Merger** (Rust microservice, ~400 LOC)
   - `list_agent_branches()` — Find all specs/agent-* branches
   - `validate_branch()` — Check markdown, Spec-Traces, FR↔Test
   - `attempt_merge()` — Perform 3-way merge
   - `process_batch()` — Parallel batch processing

2. **GitHub Actions Orchestrator** — Scheduled every 5 min
3. **Conflict Handler** — Creates GitHub issues for manual review
4. **Monitoring & Health** — Daily checks, alerts, metrics

---

### 5. Work Packages Breakdown
**File:** `docs/reference/SSOT_PHASE1_WORK_PACKAGES.md`
**Length:** 20 KB (380 lines)
**Audience:** Team leads, work package owners
**Contains:**
- 5 work packages (WP1.1-1.5) with full specifications
- Dependency graph (DAG)
- Team assignments & responsibilities
- Success metrics per WP
- Testing plans per WP
- Effort breakdowns per WP
- Risk mitigation
- Completion criteria

**Read time:** 20 minutes
**Key takeaway:** Clear ownership, deliverables, and acceptance criteria for each WP

**Work Packages:**

| WP | Title | Owner | Effort | Duration |
|----|-------|-------|--------|----------|
| WP1.1 | Branch Protection & Infrastructure | DevOps Lead | 4h | Days 1-2 |
| WP1.2 | Specs Registry & Metadata | Spec Coordinator | 6h | Days 2-3 |
| WP1.3 | Auto-Merge Service | Infra Engineer | 12-20h | Days 4-10 |
| WP1.4 | CI Validation & FR↔Test | QA Engineer | 12h | Days 3-9 |
| WP1.5 | Agent Onboarding & Docs | Spec + Tech Writer | 12h | Days 5-10 |

**Total:** 80 hours (parallelizable; 4-5 people × 2 weeks)

---

## Reading Paths by Role

### If You're a Project Sponsor/Executive
1. Read: **Executive Summary** (10 min)
2. Decide: Approve Phase 1 investment?
3. Action: Assign work packages to team leads

### If You're a Team Lead
1. Read: **Executive Summary** (10 min)
2. Read: **Work Packages** (20 min)
3. Read: **Implementation Plan** details for your tasks
4. Action: Assign WP to team members, track progress

### If You're an Engineer (Any Discipline)
1. Read: **Implementation Plan** (30 min) — Find your task
2. Read: **Work Packages** (20 min) — Understand full context
3. Read task-specific docs:
   - Infra team: **Auto-Merge Service Architecture**
   - QA team: Implementation Plan Task 1.4
   - DevOps team: Implementation Plan Task 1.1-1.2
4. Action: Clone starter code, begin implementation

### If You're an Agent (Using SSOT)
1. Read: **Agent Workflow Guide** (20 min)
2. Run: `scripts/agent-setup.sh <your-name>`
3. Follow: Workflow A (update single FR)
4. Reference: Troubleshooting section (as needed)

### If You're an Architect/Infrastructure Designer
1. Read: **Auto-Merge Service Architecture** (25 min)
2. Read: **Implementation Plan** Task 2.1 (merge orchestrator)
3. Review: GitHub Actions workflow design
4. Validate: Performance characteristics meet requirements

---

## Cross-References & Links

### Dependency Chains
- **Phase 1 → Phase 2:** See "Next Steps" in Work Packages
- **SSOT Phase 1 → AgilePlus:** See Integration section
- **Specs/Main → Spec Registry:** See WP1.2

### Related Documents (External)
- **Main governance:** `/repos/CLAUDE.md`
- **AgilePlus specs:** `/AgilePlus/kitty-specs/`
- **Agent instructions:** `/AGENTS.md`
- **Phase execution:** Previous phase docs in `docs/reference/`

### Implementation Resources
- **Starter code:** `libs/phenotype-batch-merger/` (stub)
- **GitHub Actions templates:** `.github/workflows/` (to be created)
- **Scripts location:** `scripts/` (validation, hooks, utilities)

---

## Key Metrics Dashboard

### Baseline (Current State)
| Metric | Value | Status |
|--------|-------|--------|
| Specs completeness | 78/100 | 🟡 Partial |
| Health score | 42/100 | 🔴 Low |
| Manual merge overhead | 2-3h/agent/week | 🔴 High |
| FR↔Test coverage | 85% | 🟡 Incomplete |
| Agent adoption (SSOT) | 0% | ⚪ Not started |

### Target (After Phase 1)
| Metric | Value | Status |
|--------|-------|--------|
| Specs completeness | 95/100 | 🟢 High |
| Health score | 65/100 | 🟢 Target |
| Manual merge overhead | <30 min/agent/week | 🟢 Minimal |
| FR↔Test coverage | 100% | 🟢 Complete |
| Agent adoption (SSOT) | 80%+ | 🟢 High |

---

## Quick Lookup Tables

### Tasks by Duration
| Duration | Tasks |
|----------|-------|
| 4h | WP1.1 (branch protection) |
| 6h | WP1.2 (registry), WP1.4 (CI) |
| 8h+ | WP1.3 (merge), WP1.5 (onboarding) |

### Tasks by Owner
| Owner | WPs |
|-------|-----|
| DevOps | WP1.1, WP2.2 (4h + 6h) |
| QA | WP1.4 (12h) |
| Spec Coordinator | WP1.2, WP2.3, WP2.4 (6h + 8h + 6h) |
| Infrastructure | WP1.3, WP2.1 (12-20h) |
| Tech Writer | WP1.5 (documentation) |

### Critical Path
1. WP1.1 (Branch setup) — 4h
2. WP1.2 (Registry) — 6h
3. WP1.5 (Onboarding) — 12h
**Total:** 22 sequential hours (+ parallel WP1.3, WP1.4)

### Parallelizable Work
- WP1.1 + WP1.2 (both start Day 1-2)
- WP1.3 + WP1.4 (both start Day 3-4)
- All finalized by Day 10

---

## Checklist for Getting Started

### Day 1 (Today)
- [ ] Review Executive Summary
- [ ] Approve Phase 1 investment
- [ ] Assign WP owners to team leads
- [ ] Schedule kickoff meeting

### Week 1 Kickoff
- [ ] All-hands overview (30 min)
- [ ] WP owners read implementation plan
- [ ] Engineers start assigned WPs
- [ ] Create #ssot-phase1 Slack channel

### Week 1 Progress
- [ ] WP1.1 complete (branch setup)
- [ ] WP1.2 foundation (registry structure)
- [ ] WP1.3 underway (merge service)
- [ ] WP1.4 underway (validation)
- [ ] Daily standup (15 min)

### Week 2 Progress
- [ ] WP1.3 complete (auto-merge working)
- [ ] WP1.4 complete (CI validation)
- [ ] WP1.5 underway (agent onboarding)
- [ ] Pilot agents testing (5 agents)
- [ ] Daily standup (15 min)

### Week 2 Final (Days 9-10)
- [ ] Final testing & QA
- [ ] Documentation finalized
- [ ] Team trained
- [ ] Readiness review
- [ ] Deployment approval

### Post-Phase 1 (Week of 2026-04-11)
- [ ] Go/no-go decision
- [ ] Retrospective meeting (1h)
- [ ] Phase 1 closure report
- [ ] Phase 2 planning begins

---

## FAQ & Common Questions

**Q: How much will Phase 1 cost?**
A: ~$8,000 (5 engineers × 40h/week × 2 weeks × $100/hr burdened). But ROI is 1,250% in month 1 (saves 100h/week × $100/h = $10k/week).

**Q: Can we do this faster than 2 weeks?**
A: Possibly in 1 week with 8-10 engineers working full-time. Current plan balances speed with quality.

**Q: What if auto-merge breaks?**
A: Rollback plan: Disable workflow → Return to manual merges → Root cause analysis → Fix & re-deploy.

**Q: Will agents need training?**
A: Yes, 1-2 hours to learn workflow. Workflow Guide + video walkthrough provided.

**Q: How does this integrate with AgilePlus?**
A: Phase 2 task: Link specs/main → work packages → execution tracking.

**Q: What if we have >50 agents?**
A: Batch processing scales to 100+ agents. Just increase batch interval (5 min → 10 min) if needed.

---

## Document Metrics

| Document | KB | Lines | Read Time | Audience |
|----------|-----|-------|-----------|----------|
| Executive Summary | 14 | 250 | 10 min | Sponsors |
| Implementation Plan | 31 | 550 | 30 min | Leads |
| Agent Workflow | 21 | 400 | 20 min | Agents |
| Auto-Merge Architecture | 17 | 350 | 25 min | Architects |
| Work Packages | 20 | 380 | 20 min | Leads |
| **TOTAL** | **103** | **1,930** | **105 min** | **All** |

**Plus existing docs:** SSOT_IMPLEMENTATION_ROADMAP.md, SSOT_ARCHITECTURE_INDEX.md, etc. (additional 2,990 lines)

**Grand total:** ~4,920 lines of Phase 1 specification

---

## Next Actions

### For Sponsors/Executives
1. Read Executive Summary (10 min)
2. Approve $8,000 investment
3. Schedule team lead assignments

### For Team Leads
1. Read Executive Summary (10 min)
2. Read Work Packages (20 min)
3. Identify which WP you'll own
4. Schedule team kickoff

### For Engineers
1. Wait for WP assignment from lead
2. Read relevant implementation plan tasks
3. Clone starter code & begin

### For Agents
1. Wait for Phase 1 completion (2 weeks)
2. Run `scripts/agent-setup.sh <your-name>`
3. Follow Agent Workflow Guide
4. Start authoring specs!

---

## Support & Communication

**For questions:**
- Technical: DM infrastructure team
- Governance: DM spec coordinator
- Scheduling: Post in #ssot-phase1 Slack

**For blockers:**
- Raise in daily standup (15 min)
- Create GitHub issue with tag `ssot-phase1-blocker`

**For feedback:**
- Post-Phase 1 retrospective (all hands)
- Anonymous feedback survey (Google Form)

---

**Master Index Version:** 1.0
**Created:** 2026-03-31
**Status:** Complete & Ready for Execution
**Next Update:** 2026-04-11 (post-Phase 1 retrospective)

---

## Footer Navigation

← [Previous Phase Docs](docs/reference/PHASE_1_EXECUTION_PLAN.md)
| [SSOT Home](docs/reference/) |
[Next: Phase 2 Planning](docs/reference/) →

**Start Phase 1 execution today. Reach 65/100 health score by 2026-04-11.**
