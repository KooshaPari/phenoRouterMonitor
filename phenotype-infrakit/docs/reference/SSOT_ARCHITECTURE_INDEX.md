# SSOT Architecture — Complete Documentation Index

**Purpose**: Navigation hub for the polyrepo Single Source of Truth (SSOT) architecture
**Updated**: 2026-03-30
**Audience**: All agents, team leads, and system operators

---

## Quick Start (Choose Your Role)

### I'm an Agent (Feature Development)
1. Read: **SSOT_QUICK_REFERENCE.md** (5 min)
2. Bookmark: Decision tree for common scenarios
3. Start: Creating feature branch (see Workflow A)

### I'm a Team Lead (Architecture & Oversight)
1. Read: **POLYREPO_SSOT_ARCHITECTURE.md** (20 min) — Full design
2. Review: Phase 1 deliverables (**SSOT_IMPLEMENTATION_ROADMAP.md**)
3. Monitor: Metrics and success criteria
4. Reference: Risk management section

### I'm a DevOps/Infrastructure Engineer
1. Read: **SSOT_IMPLEMENTATION_ROADMAP.md** (30 min) — Detailed tasks
2. Review: Service architecture (spec/dependency reconciliation)
3. Execute: WP1-WP6 tasks in order
4. Deploy: CI workflows and branch protections

### I'm a Stakeholder (Business/Product)
1. Read: **SSOT_VISUAL_GUIDE.md** (10 min) — Diagrams and flows
2. Understand: Phase timeline and resource allocation
3. Review: Overall health scorecard (Part 10)
4. Question: Impact on delivery velocity

---

## Document Catalog

### 1. **POLYREPO_SSOT_ARCHITECTURE.md** (Main Design)
**Type**: Comprehensive architecture specification
**Length**: ~5,000 words, 10 sections
**Purpose**: Complete design reference for entire SSOT system

**Contains**:
- Part 1: Current state analysis (polyrepo structure, challenges, risks)
- Part 2: Three-phase roadmap (timelines, deliverables, success criteria)
- Part 3: Multi-agent concurrency model (conflict resolution, worktree lifecycle)
- Part 4: Branching strategy (three-branch model, naming conventions)
- Part 5: Reconciliation service requirements (architecture, API, deployment)
- Part 6: Implementation timeline (detailed phase breakdown)
- Part 7: Success criteria & metrics (phase-by-phase)
- Part 8: Risk management (identified risks + mitigation)
- Part 9: Appendices (glossary, related docs, checklists)
- Part 10: Conclusion & next steps

**When to Read**:
- First time understanding SSOT architecture
- Designing related systems or extensions
- Risk assessment or due diligence

**Key Sections**:
- Phase 1 (Specs Canonicalization): Weeks 1-2, 8-12 agents
- Phase 2 (Dependency Reconciliation): Weeks 3-6, 12-20 agents
- Phase 3 (Platform Chassis Federation): Weeks 8-12, 15-20 agents

---

### 2. **SSOT_QUICK_REFERENCE.md** (Agent Cheat Sheet)
**Type**: One-page quick reference
**Length**: ~1,500 words
**Purpose**: Rapid lookup for agents in feature development

**Contains**:
- Three-branch model at a glance
- Creating a feature branch (step-by-step)
- Spec conflicts (what happens automatically)
- Circular dependency handling
- Ready-to-merge workflow
- Decision tree for conflict resolution
- Common mistakes & fixes
- Links to full documentation

**When to Read**:
- Creating a new feature branch
- Debugging a CI failure
- Resolving a spec conflict
- First-time agent orientation

**Key Takeaway**: "Work in `.worktrees/*`, push, open PR, service handles rest"

---

### 3. **SSOT_VISUAL_GUIDE.md** (Diagrams & Flows)
**Type**: Visual reference with Mermaid diagrams
**Length**: ~2,000 words, 11 diagrams
**Purpose**: Understanding architecture via diagrams and flows

**Contains**:
1. Three-branch model (diagram)
2. Spec conflict resolution flow (ASCII)
3. Multi-agent merge orchestration (ASCII)
4. Worktree lifecycle state machine (Mermaid)
5. FR↔Test traceability diagram
6. Dependency graph validation flow
7. Concurrent spec merge sequence diagram
8. Phase timeline overview
9. System health scorecard (before/after)
10. Common agent workflows (A: features, B: bugs)
11. Glossary with visual examples

**When to Read**:
- Need to understand system visually
- Explaining architecture to stakeholders
- Decision trees for conflict scenarios
- Progress tracking (health scorecard)

**Key Diagram**: Health scorecard showing 42/100 (now) → 94/100 (Phase 3 complete)

---

### 4. **SSOT_IMPLEMENTATION_ROADMAP.md** (Work Breakdown)
**Type**: Detailed task breakdown with acceptance criteria
**Length**: ~3,000 words, 20+ tasks
**Purpose**: Execution plan for Phase 1 (Weeks 1-2)

**Contains**:
- WP1: Git Infrastructure Setup (Tasks 1.1-1.3)
  - Resolve git conflict markers
  - Create `specs/main` branch
  - Configure CI checks

- WP2: Specs Registry Creation (Tasks 2.1-2.2)
  - Index all current specs
  - Backfill missing specs

- WP3: Spec Merge Service Deployment (Tasks 3.1-3.2)
  - Build spec reconciliation service
  - Integrate with CI

- WP4: FR↔Test Traceability Gate (Tasks 4.1-4.2)
  - Build validation script
  - Enforce in CI

- WP5: Agent Training & Documentation (Tasks 5.1-5.2)
  - Update AGENTS.md
  - Create FAQ

- WP6: Soft Launch & Validation (Tasks 6.1-6.2)
  - Log-only mode (validate service)
  - Full rollout (auto-merge mode)

**When to Use**:
- Planning Phase 1 execution
- Assigning tasks to agents
- Tracking progress (WP-by-WP)
- Resource allocation

**Key Metric**: 10 agents, 80-100 tool calls, ~2 weeks wall-clock

---

### 5. **SSOT_ARCHITECTURE_INDEX.md** (This Document)
**Type**: Navigation and orientation hub
**Length**: ~1,500 words
**Purpose**: Help readers find the right document for their needs

**Contains**:
- Quick-start by role (agent, lead, DevOps, stakeholder)
- Complete document catalog with summaries
- Cross-references between documents
- Reading order recommendations
- FAQ for common questions

**When to Read**:
- First time accessing SSOT documentation
- Not sure which document to read next
- Orienting new team members

---

## Reading Order Recommendations

### For Agents (Fastest Path)
1. **SSOT_QUICK_REFERENCE.md** (5 min)
2. Bookmark decision tree
3. Start creating feature branches
4. Reference full arch if questions: **POLYREPO_SSOT_ARCHITECTURE.md** (Part 3-4)

**Estimated Time**: 20-30 min total (including skimming full arch)

### For Team Leads (Strategic Overview)
1. **SSOT_ARCHITECTURE_INDEX.md** (this doc) (10 min)
2. **POLYREPO_SSOT_ARCHITECTURE.md** (Part 1: current state, Part 2: phases) (15 min)
3. **SSOT_VISUAL_GUIDE.md** (health scorecard, timeline) (10 min)
4. **SSOT_IMPLEMENTATION_ROADMAP.md** (Phase 1 deliverables, resource allocation) (15 min)

**Estimated Time**: 50 min total

### For DevOps/Infrastructure (Implementation Focus)
1. **SSOT_IMPLEMENTATION_ROADMAP.md** (WP1-WP6, detailed tasks) (45 min)
2. **POLYREPO_SSOT_ARCHITECTURE.md** (Part 5: reconciliation service requirements) (15 min)
3. **SSOT_VISUAL_GUIDE.md** (diagrams 6-7: dependency validation, spec merge) (10 min)
4. Code template in roadmap (Python script skeleton)

**Estimated Time**: 70 min total

### For Business/Product (30,000 ft View)
1. **SSOT_VISUAL_GUIDE.md** (intro, timeline, health scorecard) (10 min)
2. **POLYREPO_SSOT_ARCHITECTURE.md** (Part 2: phases + deliverables) (15 min)
3. Key metrics (Part 7: success criteria, especially table in section 7.4)

**Estimated Time**: 25 min total

---

## Cross-References

### By Topic

**Branching Strategy**:
- Full design: `POLYREPO_SSOT_ARCHITECTURE.md` Part 4
- Quick reference: `SSOT_QUICK_REFERENCE.md` "Branches at a Glance"
- Visual: `SSOT_VISUAL_GUIDE.md` Diagram 1 & Diagram 4
- Implementation: `SSOT_IMPLEMENTATION_ROADMAP.md` Task 1.2

**Spec Management**:
- Full design: `POLYREPO_SSOT_ARCHITECTURE.md` Part 2.1 & 3.2
- Agent workflow: `SSOT_QUICK_REFERENCE.md` "Creating a Feature Branch"
- Visual flow: `SSOT_VISUAL_GUIDE.md` Diagram 2 & 7
- Implementation: `SSOT_IMPLEMENTATION_ROADMAP.md` WP2-3

**Merge Orchestration**:
- Full design: `POLYREPO_SSOT_ARCHITECTURE.md` Part 3.1-3.4
- Concurrency model: `POLYREPO_SSOT_ARCHITECTURE.md` Part 3.4
- Visual: `SSOT_VISUAL_GUIDE.md` Diagram 3 & 8
- Implementation: `SSOT_IMPLEMENTATION_ROADMAP.md` WP6

**Multi-Agent Coordination**:
- Conflict resolution: `POLYREPO_SSOT_ARCHITECTURE.md` Part 3.3
- State machine: `SSOT_VISUAL_GUIDE.md` Diagram 4
- Decision tree: `SSOT_QUICK_REFERENCE.md` "Decision Tree"
- Phase 2 details: `POLYREPO_SSOT_ARCHITECTURE.md` Part 2.2

**CI/Infrastructure**:
- Service architecture: `POLYREPO_SSOT_ARCHITECTURE.md` Part 5
- Implementation: `SSOT_IMPLEMENTATION_ROADMAP.md` Tasks 1.3, 3.2, 4.2
- Configuration examples: Code snippets in roadmap

---

## FAQ: Which Document Should I Read?

| Question | Answer | Document |
|----------|--------|----------|
| I'm starting a new feature; what do I do? | Follow the 4-step workflow | SSOT_QUICK_REFERENCE.md |
| Two agents created the same spec ID; what happens? | Service auto-reassigns; no manual work | SSOT_QUICK_REFERENCE.md + SSOT_VISUAL_GUIDE.md Diagram 2 |
| How do I understand the full architecture? | Read Part 1-4 for complete picture | POLYREPO_SSOT_ARCHITECTURE.md |
| I need to deploy Phase 1; where do I start? | WP1-WP6 with detailed tasks and acceptance criteria | SSOT_IMPLEMENTATION_ROADMAP.md |
| How does merge orchestration prevent bottlenecks? | Parallel integration/* branches + topological sort | SSOT_VISUAL_GUIDE.md Diagram 3 + POLYREPO_SSOT_ARCHITECTURE.md Part 3.1 |
| What are the success criteria for Phase 1? | List of checkboxes and metrics | POLYREPO_SSOT_ARCHITECTURE.md Part 7 + SSOT_IMPLEMENTATION_ROADMAP.md Phase 1 |
| I want to explain this to stakeholders; what visuals? | Health scorecard, timeline, phase overview | SSOT_VISUAL_GUIDE.md Part 9-10 |
| What are the risks? | Identified risks + mitigation strategies | POLYREPO_SSOT_ARCHITECTURE.md Part 8 |
| How long will Phase 1 take? | 2 weeks, 10 agents, 80-100 tool calls | SSOT_IMPLEMENTATION_ROADMAP.md |
| What's the overall timeline? | Phase 1: 2 weeks, Phase 2: 4 weeks, Phase 3: 4 weeks (10-12 weeks total) | SSOT_VISUAL_GUIDE.md Diagram 8 |

---

## Navigation Map

```
SSOT Architecture Documentation
│
├─ Quick Start (Pick Your Role)
│  ├─ Agent → SSOT_QUICK_REFERENCE.md
│  ├─ Team Lead → POLYREPO_SSOT_ARCHITECTURE.md (Part 1-2)
│  ├─ DevOps → SSOT_IMPLEMENTATION_ROADMAP.md
│  └─ Stakeholder → SSOT_VISUAL_GUIDE.md (Part 9)
│
├─ Core Documents
│  ├─ POLYREPO_SSOT_ARCHITECTURE.md (Full design, 10 sections)
│  ├─ SSOT_IMPLEMENTATION_ROADMAP.md (Phase 1 tasks, 20+ items)
│  ├─ SSOT_QUICK_REFERENCE.md (Agent cheat sheet, 1 page)
│  └─ SSOT_VISUAL_GUIDE.md (11 diagrams)
│
├─ By Topic
│  ├─ Branching Strategy → Part 4 + Diagram 1
│  ├─ Spec Management → Part 2.1 + Diagram 2
│  ├─ Merge Orchestration → Part 3 + Diagram 3
│  ├─ CI/Infrastructure → Part 5 + Roadmap Tasks 1.3, 3.2
│  └─ Metrics & Health → Part 7 + Diagram 9
│
└─ Implementation
   ├─ Phase 1 (Weeks 1-2) → Roadmap WP1-WP6
   ├─ Phase 2 (Weeks 3-6) → Part 2.2
   └─ Phase 3 (Weeks 8-12) → Part 2.3
```

---

## Key Metrics at a Glance

### Phase 1 (Specs Canonicalization)
- Duration: 2 weeks
- Agents: 8-12
- Deliverables: specs/main, spec merge service, FR↔Test gate
- Success: 100% FR↔Test coverage, 0 spec conflicts

### Phase 2 (Dependency Reconciliation)
- Duration: 4 weeks (Weeks 3-6)
- Agents: 12-20
- Deliverables: Dependency graph, circular dep detection, parallel merges
- Success: 5 simultaneous merges, <10 min total merge time

### Phase 3 (Platform Chassis Federation)
- Duration: 4-5 weeks (Weeks 8-12)
- Agents: 15-20
- Deliverables: Versioned chassis, platform contracts, health monitoring
- Success: 100% governance centralization, 95/100 health score

---

## How to Use These Documents Effectively

### 1. **Bookmark Key Sections**
- Agent: `SSOT_QUICK_REFERENCE.md` (entire doc)
- Team Lead: `POLYREPO_SSOT_ARCHITECTURE.md` Part 2 & 6
- DevOps: `SSOT_IMPLEMENTATION_ROADMAP.md` Tasks and acceptance criteria

### 2. **Use in Different Contexts**
- **Planning**: Use roadmap for resource allocation
- **Execution**: Use quick ref for daily work
- **Review**: Use visual guide for team discussions
- **Risk**: Use Part 8 for contingency planning

### 3. **Reference as You Build**
- Creating spec merge service? → See Task 3.1 Python skeleton
- Setting up CI? → See Task 1.3 GitHub Actions config
- Training agents? → Use SSOT_QUICK_REFERENCE.md + visual flows

### 4. **Track Progress**
- Use acceptance criteria checklists (roadmap)
- Use success criteria metrics (Part 7)
- Use health scorecard (Diagram 9) for overall status

---

## Related Documents

- `POLYREPO_SSOT_ARCHITECTURE.md` — Full specification (primary)
- `SSOT_IMPLEMENTATION_ROADMAP.md` — Phase 1 execution plan
- `SSOT_QUICK_REFERENCE.md` — Agent cheat sheet
- `SSOT_VISUAL_GUIDE.md` — Diagrams and flowcharts
- `AGENTS.md` — Agent coordination rules (to be updated)
- `CLAUDE.md` — Governance rules (to be consolidated)
- `FUNCTIONAL_REQUIREMENTS.md` — Project specs
- `docs/reference/SPECS_REGISTRY.md` — Canonical spec registry (Phase 1 deliverable)
- `AUDIT_LOG.md` — Decision and conflict trail (Phase 1 deliverable)

---

## Getting Help

### Common Questions
- **"What's my next step?"** → Check "Quick Start" at top of this doc
- **"I'm stuck on a conflict"** → See SSOT_QUICK_REFERENCE.md decision tree
- **"How does the service work?"** → See POLYREPO_SSOT_ARCHITECTURE.md Part 5
- **"What's our timeline?"** → See SSOT_VISUAL_GUIDE.md Diagram 8

### Reporting Issues
- Service bugs? → Comment in code, create GitHub issue
- Doc unclear? → File issue with doc name + section
- Architecture question? → Slack #engineering with document reference

### Getting Trained
- Phase 1 rollout will include 30-min mandatory training session
- Materials will reference SSOT_QUICK_REFERENCE.md + SSOT_VISUAL_GUIDE.md
- Q&A session will cover common scenarios

---

## Document Maintenance

**Last Updated**: 2026-03-30
**Maintained By**: Claude Code (AI) + team leads
**Update Frequency**: After each phase completion or major revision
**Version Control**: Git (all docs in version control)

**To Update**:
1. Edit the relevant document
2. Update this index if structure changes
3. Commit with message: "docs: update SSOT <document> <reason>"
4. All docs stay synced in git

---

## Summary

This index provides a roadmap through the complete SSOT architecture documentation. Start with your role-based quick start, then dive into the specific documents you need.

**Key Takeaway**: The Phenotype ecosystem will move from serial, manually-choreographed merges (42/100 health) to parallel, automated conflict-resolution (94/100 health) across 50+ concurrent agents.

---

**Next Step**: Choose your role above and start with the recommended first document.
