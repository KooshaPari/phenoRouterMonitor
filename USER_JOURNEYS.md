# AgilePlus User Journeys

**Version:** 1.0  
**Status:** Approved  
**Date:** 2026-03-26

---

## Overview

This document describes the key user journeys for AgilePlus, covering the perspectives of:
1. **AI Coding Agents** — autonomous execution of work packages
2. **Solo Developers** — manual specification and oversight
3. **Agent Orchestrators** — fleet management and dispatch
4. **Platform Engineers** — governance and compliance operations

Each journey maps to one or more epics and demonstrates how the system supports the target user's goals.

---

## Journey 1: AI Agent Completes a Feature (E1, E2, E6, E7)

**Persona**: AI Coding Agent (Claude Code, Codex)  
**Goal**: Execute a work package to completion with minimal human intervention  
**Time**: 30–120 minutes per work package

### Preconditions
- Feature is in **Researched** or **Planned** state
- Work package is assigned to agent with clear acceptance criteria
- Worktree is prepared with target branch and base commit
- Governance contract defines evidence requirements (tests, CI output, review)

### Steps

1. **Agent receives task dispatch** (E6.1)
   - MCP server provides work package context (spec, acceptance criteria, file scope)
   - Agent receives prompt with task instructions, feature context, and module ownership

2. **Agent checks governance requirements** (E2)
   - MCP tool queries governance contract
   - Agent understands what evidence types are required (test coverage, CI build status, code review)
   - Agent identifies dependency graph for upstream/downstream work packages (E1.4)

3. **Agent implements work package** (E4.4, E6.1)
   - Agent creates feature branch in isolated worktree
   - Agent writes code, commits with descriptive messages
   - Agent runs local tests and linting

4. **Agent collects evidence** (E2.2)
   - Agent runs unit tests and captures coverage reports
   - Agent runs CI pipeline (GitHub Actions / GH CLI) and saves output
   - Agent runs security scanner (semgrep, trivy, or similar) and collects results
   - MCP tool records all evidence artifacts linked to work package

5. **Agent submits PR** (E4.16)
   - Agent uses MCP tool to generate PR description from work package metadata, evidence, and audit trail
   - Agent opens PR on target branch with generated description
   - Agent updates work package state to **Implementing** → **Validated**

6. **Agent enters review loop** (E6.3)
   - Human or AI reviewer provides severity-classified comments (critical/major/minor)
   - Agent receives feedback via MCP tool
   - Agent addresses critical/major issues, commits fixes
   - Loop repeats until:
     - ✅ **Approved** → move to step 7
     - ❌ **Rejected** → WP marked **Blocked**, review with human
     - ⏱️ **Max cycles reached** → escalate to human review

7. **Agent ships work** (E4.7)
   - Work package state transitions to **Done**
   - Feature state advances (Implementing → Validated → Shipped)
   - MCP tool records state transition with actor attribution in immutable audit log (E3.1)
   - Event is published to NATS for external sync (E7.3)
   - If Plane.so sync is configured, feature is updated to reflect completed work (E7.1)

### Post-Conditions
- Work package state: **Done**
- Feature may advance to **Shipped** if all WPs are done
- Audit log contains hash-chained entries for all transitions
- External systems (Plane.so, GitHub) are synced with new feature state

---

## Journey 2: Solo Developer Specifies and Tracks a Feature (E1, E4, E5)

**Persona**: Solo Developer  
**Goal**: Create a feature specification, assign it to a cycle, dispatch agents, and monitor progress  
**Time**: 1–2 days for specification; 5–10 days for full implementation cycle

### Preconditions
- User has AgilePlus CLI installed and initialized
- Local Git repository with target project
- Optional: Plane.so account linked for external sync

### Steps

1. **Developer creates a feature** (E1.1, E4.1)
   - CLI: `agileplus specify --name "Dark mode toggle" --desc "..."`
   - System creates feature with slug `dark-mode-toggle`, assigns unique ID
   - Kitty-specs directory structure created: `kitty-specs/dark-mode-toggle/{spec.md, plan.md, acceptance_criteria.md}`

2. **Developer assigns to cycle and module** (E1.5, E1.6, E4.11, E4.12)
   - CLI: `agileplus module create --name "UI Improvements"`
   - CLI: `agileplus cycle create --name "Sprint 3"`
   - Feature tracking enables sprint-based planning

3. **Developer reviews feature status** (E5.4, E5.2)
   - CLI: `agileplus show dark-mode-toggle`
   - Web dashboard displays feature timeline, module affiliation, and associated work packages

4. **Developer researches feature** (E4.2)
   - CLI: `agileplus research dark-mode-toggle --agent claude-code`
   - Agent analyzes codebase, identifies relevant files
   - Feature state transitions to **Researched**

5. **Developer creates plan** (E1.3, E4.3)
   - CLI: `agileplus plan dark-mode-toggle --decompose`
   - System creates work packages with dependencies

6. **Developer defines governance** (E2.1)
   - Creates governance.yaml with evidence requirements
   - Unit tests: >80% coverage
   - Code review: ≥1 approval

7. **Developer dispatches agents** (E6.1, E6.2)
   - CLI: `agileplus implement dark-mode-toggle --agents 4`
   - Agents begin execution in isolated worktrees

8. **Developer monitors progress** (E6.3)
   - CLI: `agileplus queue --watch` displays real-time agent progress
   - Dashboard shows review cycle status

9. **Developer validates and ships** (E2.4, E4.7)
   - CLI: `agileplus validate dark-mode-toggle`
   - CLI: `agileplus ship dark-mode-toggle`

10. **Developer generates retrospective** (E4.8)
    - CLI: `agileplus retrospective dark-mode-toggle`
    - System generates metrics and learning report

---

## Journey 3: Agent Orchestrator Manages a Fleet (E6, E7)

**Persona**: Agent Orchestrator  
**Goal**: Dispatch and monitor multiple AI agents across features  
**Time**: 10–15 minutes per day for monitoring

### Preconditions
- Multiple features queued with work packages
- Agent pool configured (4–20 agents)
- Governance contracts defined

### Steps

1. **Orchestrator views backlog** (E4.13)
   - CLI: `agileplus queue --status`
   - Dashboard shows sprint burndown and agent utilization

2. **Orchestrator configures dispatch** (E6.1)
   - CLI: `agileplus implement batch --features dark-mode,notifications,api-docs --max-agents 12`

3. **Orchestrator monitors fleet** (E6.2)
   - Dashboard displays real-time agent status
   - REST API provides detailed metrics

4. **Orchestrator handles failures** (E6.3)
   - Reviews failed work packages and reassigns as needed

5. **Orchestrator validates and ships** (E2.4, E4.7)
   - CLI: `agileplus validate batch --features dark-mode,notifications,api-docs`
   - CLI: `agileplus ship batch --target main`

---

## Journey 4: Platform Engineer Enforces Governance (E2, E3)

**Persona**: Platform Engineer  
**Goal**: Define and enforce governance rules  
**Time**: 1–2 hours setup; ongoing monitoring

### Preconditions
- CI/CD pipeline configured
- Policy rules defined

### Steps

1. **Engineer defines governance contracts** (E2.1)
   - Creates policies/security.yaml, policies/quality.yaml

2. **Engineer binds contracts** (E2.1)
   - CLI: `agileplus governance bind dark-mode policies/security.yaml`

3. **Engineer validates compliance** (E2.4)
   - CLI: `agileplus validate --feature dark-mode`

4. **Engineer queries audit trail** (E3.1)
   - REST API: `GET /audit/features/dark-mode`

5. **Engineer monitors policy compliance** (E10.3)
   - Dashboard shows compliance metrics across all features

---

## Journey 5: External Team Syncs with Plane.so (E7.1, E7.3)

**Persona**: Project Manager  
**Goal**: Keep Plane.so issues in sync with AgilePlus features  
**Time**: Setup 15 min; automatic thereafter

### Preconditions
- AgilePlus feature exists
- Plane.so issue exists
- Sync mapping configured

### Steps

1. **Engineer configures sync** (E7.1)
   - CLI: `agileplus sync configure --type plane.so --api-key $KEY`

2. **Engineer initiates sync** (E7.1)
   - CLI: `agileplus sync plane.so --direction bidirectional`

3. **Bidirectional updates** (E7.1, E7.3)
   - AgilePlus updates Plane.so on state changes
   - Plane.so updates AgilePlus on external changes

4. **Conflict resolution** (E7.1)
   - Concurrent edits detected
   - User resolves: `agileplus sync resolve --feature dark-mode --action prefer-local`

---

## Key Touchpoints and State Transitions

| Journey | Epic | Key Commands | States |
|---------|------|-------------|--------|
| AI Agent | E1, E2, E6, E7 | `dispatch`, `review-loop`, `ship` | Implementing → Shipped |
| Solo Dev | E1, E4, E5 | `specify`, `plan`, `implement`, `ship` | Created → Shipped |
| Orchestrator | E6, E7 | `implement batch`, `ship batch` | Multi-feature parallel |
| Platform Eng | E2, E3 | `governance bind`, `validate` | All features enforced |
| Sync | E7 | `sync configure`, `sync resolve` | Bidirectional sync |

---

## Success Metrics

- **AI Agent**: Completes WP without human intervention; governance passes
- **Solo Dev**: Specification-to-shipped in <5 working days
- **Orchestrator**: >80% agent utilization; >95% success rate
- **Platform Eng**: 100% governance compliance; zero audit tampering
- **Sync**: <1 min sync latency; <5% conflict rate

---

## Design Principles

1. **Immutable Audit Trail** — Every state change recorded and verifiable
2. **Evidence-Driven** — Features ship only when governance satisfied
3. **Autonomous Agents** — Minimal human intervention required
4. **Local-First** — Core functionality offline; external sync optional
5. **Extensible** — Pluggable adapters via ports
