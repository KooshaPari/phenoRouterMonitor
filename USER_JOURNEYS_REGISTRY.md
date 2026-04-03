# User Journeys Registry

**Version:** 1.0
**Status:** Active
**Updated:** 2026-04-01
**Branch:** `specs/main`

---

## Overview

This registry consolidates all User Journeys (UJs) across the Phenotype polyrepo, tracking user workflows, actors, goals, and success metrics.

---

## Master Journey Index

### phenotype-infrakit Journeys (10 total)

**Health:** ✅ 100% Deployed

| ID | Actor | Goal | Journey | Status | Coverage |
|----|-------|------|---------|--------|----------|
| UJ-001 | Developer | Set up workspace | Clone repo → Install deps → Build → Run tests | ✅ | 100% |
| UJ-002 | Developer | Add new crate | Create crate dir → Write Cargo.toml → Implement → Test | ✅ | 100% |
| UJ-003 | Agent | Run tests | `cargo test --workspace` → Parse results → Report → Store logs | ✅ | 100% |
| UJ-004 | DevOps | Deploy to production | Build release → Tag → Push → GitHub Actions CI/CD | ✅ | 100% |
| UJ-005 | Agent | Debug failures | Check logs → Identify error → Run isolated test → Fix → Re-run | ✅ | 100% |
| UJ-006 | Operator | Monitor health | Query health endpoint → Parse metrics → Alert if threshold exceeded | ✅ | 100% |
| UJ-007 | Manager | Track progress | View dashboard → Query phase status → Generate reports | ✅ | 100% |
| UJ-008 | Agent | Rollback safely | Check version history → Revert to stable → Re-test → Deploy | ✅ | 100% |
| UJ-009 | Security | Audit logs | Query audit trail → Verify hash chain → Check authorization | ✅ | 100% |
| UJ-010 | Team | Incident response | Detect anomaly → Create issue → Triage → Execute mitigation plan | ✅ | 100% |

### AgilePlus Journeys (6 total)

**Health:** ⚠️ 50% (3 deployed, 3 draft)

| ID | Actor | Goal | Journey | Status |
|----|-------|------|---------|--------|
| AJ-001 | PM | Create spec | Write FR/ADR/PLAN → Review → Trace to tests → Merge → Deploy | ✅ |
| AJ-002 | Engineer | Implement FR | Read spec → Write code → Add tests → Link tests to FR → Commit | ✅ |
| AJ-003 | QA | Trace FR↔Test | Run coverage validation → Generate matrix → Report gaps → Create issues | ✅ |
| AJ-004 | Agent | Auto-merge specs | Push to specs/agent-* → CI validates → Auto-merge to specs/main within 5min | 🔧 Draft |
| AJ-005 | Manager | View dashboard | Check SSOT health → See FR coverage % → Monitor merge health → Alert on drop | 🔧 Draft |
| AJ-006 | Stakeholder | Review progress | View milestone tracker → Check phase completion → Generate burndown → Present | 🔧 Draft |

**Blockers:** Journeys AJ-004 through AJ-006 need implementation

### platforms/thegent Journeys (12 total)

**Health:** ✅ 100% Deployed

| ID | Actor | Goal | Journey | Status |
|----|-------|------|---------|--------|
| TJ-001 | Agent | Execute task | Load task → Acquire resources → Execute → Report status → Release resources | ✅ |
| TJ-002 | Agent | Load plugin | Fetch MCP tool → Validate signature → Register capability → Test → Ready | ✅ |
| TJ-003 | Agent | Recover from failure | Detect error → Check circuit breaker → Decide retry/abort → Log outcome | ✅ |
| TJ-004 | Operator | Monitor agents | Query metrics → Check resource usage → Detect anomalies → Alert | ✅ |
| TJ-005 | Engineer | Add new MCP tool | Write spec → Implement handler → Register in registry → Test → Deploy | ✅ |
| TJ-006 | Manager | Track execution | Query run history → Check success rate → Generate SLA report → Present | ✅ |
| TJ-007 | Agent | Hotload capability | Request new tool → Load WASM module → Test → Integrate → Execute | ✅ |
| TJ-008 | Agent | Request resource | Check availability → Create request → Wait for grant → Use → Release | ✅ |
| TJ-009 | DevOps | Scale horizontally | Monitor load → Spin up agents → Distribute tasks → Drain on scale-down | ✅ |
| TJ-010 | Security | Isolate execution | Create namespace → Set resource limits → Run agent → Verify isolation | ✅ |
| TJ-011 | Agent | Compose workflows | Chain multiple tasks → Handle data flow → Manage dependencies → Execute | ✅ |
| TJ-012 | Team | Incident response | Detect SLO breach → Create incident → Escalate → Execute runbook → Verify | ✅ |

### heliosCLI Journeys (8 total)

**Health:** ✅ 100% Deployed

| ID | Actor | Goal | Journey | Status |
|----|-------|------|---------|--------|
| HJ-001 | Developer | Install CLI | Download binary → Verify signature → Extract → Add to PATH → Test | ✅ |
| HJ-002 | Agent | Run harness | Load agent config → Execute within sandbox → Capture output → Store logs | ✅ |
| HJ-003 | Agent | Sandbox execution | Create container → Mount volumes → Set limits → Execute → Cleanup | ✅ |
| HJ-004 | Engineer | Load plugin | Write plugin → Register → Validate → Add to CLI → Test in harness | ✅ |
| HJ-005 | Operator | Monitor health | Check process health → Query metrics → Alert on threshold → Auto-restart | ✅ |
| HJ-006 | Security | Enforce isolation | Validate container state → Check resource limits → Verify no escapes | ✅ |
| HJ-007 | Developer | Debug locally | Run agent locally → Check logs → Set breakpoints → Inspect state | ✅ |
| HJ-008 | Team | Deploy to prod | Test in staging → Verify sandboxing → Deploy to prod → Monitor → Rollback if needed | ✅ |

---

## Journey Coverage by Repo

### Summary Table

| Repo | Total | Deployed | Draft | Coverage | Health |
|------|-------|----------|-------|----------|--------|
| phenotype-infrakit | 10 | 10 | 0 | 100% | ✅ |
| AgilePlus | 6 | 3 | 3 | 50% | ⚠️ |
| platforms/thegent | 12 | 12 | 0 | 100% | ✅ |
| heliosCLI | 8 | 8 | 0 | 100% | ✅ |
| **TOTAL** | **36** | **33** | **3** | **92%** | ✅ |

### By Actor Type

| Actor Type | Count | Examples |
|-----------|-------|----------|
| Developer | 5 | Install, debug, add crate, setup, create plugin |
| Agent | 12 | Execute task, run harness, load plugin, hotload, auto-merge |
| Operator | 4 | Monitor health, monitor agents, monitor CLI health, scale |
| Manager | 3 | Track progress, view dashboard, track execution |
| Engineer | 3 | Implement FR, add MCP tool, deploy |
| DevOps | 2 | Deploy, scale |
| Security | 2 | Audit logs, enforce isolation |
| QA | 1 | Trace FR↔Test |
| PM | 1 | Create spec |
| Team | 2 | Incident response (2 variants) |
| Stakeholder | 1 | Review progress |

---

## Journey Template & Structure

### Standard Journey Format

```markdown
#### UJ-REPO-NNN: [Actor] — [Goal]

**Actor:** [Role/Type]
**Goal:** [Primary objective]

**Flow:**
1. [First step]
2. [Second step]
...
N. [Final step]

**Success Criteria:**
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

**Failure Modes:**
- **Error 1:** [Description] → Recovery: [How to recover]
- **Error 2:** [Description] → Recovery: [How to recover]

**Metrics:**
- Duration: [Expected time]
- Success rate: [% target]
- SLA: [Response/completion time]

**Evidence:**
- Logs: [Where to find logs]
- Traces: [Tracing references]
- Artifacts: [Generated files]

**Related FRs:**
- FR-REPO-001: [Feature trace]
- FR-REPO-002: [Feature trace]

**Status:** ✅ Deployed / 🔧 Draft / ⏳ Review
```

---

## Actor Personas

### Developer

**Skills:** Programming, CLI, Git, local debugging
**Tools:** IDE, Terminal, local test environment
**Goal:** Productivity, ease of setup, quick feedback loops

**Journeys:**
- UJ-INFRA-001: Set up workspace
- UJ-INFRA-002: Add new crate
- UJ-HELIOS-001: Install CLI
- UJ-HELIOS-007: Debug locally

### Agent

**Skills:** Automation, no GUI interaction, API-driven
**Tools:** CLI, APIs, scripts
**Goal:** Reliability, quick execution, error recovery

**Journeys:**
- UJ-INFRA-003: Run tests
- UJ-INFRA-005: Debug failures
- UJ-INFRA-008: Rollback safely
- TJ-001 through TJ-012: All thegent journeys
- HJ-002, HJ-003: Harness execution
- AJ-004: Auto-merge specs

### Operator

**Skills:** Infrastructure, monitoring, incident response
**Tools:** Dashboard, monitoring systems, CLI
**Goal:** System health, quick anomaly detection, rapid mitigation

**Journeys:**
- UJ-INFRA-006: Monitor health
- TJ-004: Monitor agents
- TJ-009: Scale horizontally
- HJ-005: Monitor CLI health

### Manager

**Skills:** Project management, reporting, communication
**Tools:** Dashboard, spreadsheets, presentations
**Goal:** Visibility, progress tracking, stakeholder communication

**Journeys:**
- UJ-INFRA-007: Track progress
- TJ-006: Track execution
- AJ-005: View dashboard
- AJ-006: Review progress

---

## Journey Validation Criteria

### Deployed Journey Requirements

✅ All deployed journeys MUST have:
1. **Clear actor definition** — Who is performing the journey?
2. **Explicit goal** — What outcome are we aiming for?
3. **Step-by-step flow** — Each step testable and repeatable
4. **Success criteria** — How do we know it worked?
5. **Failure modes** — What can go wrong? How to recover?
6. **Metrics & SLAs** — Quantifiable targets
7. **Evidence trail** — Logs, traces, artifacts
8. **Related FRs** — Links to functional requirements
9. **Implementation** — Actually working end-to-end
10. **Tests** — At least 1 test per journey

### Draft Journey Requirements

🔧 Draft journeys in progress (AJ-004, AJ-005, AJ-006):
- [ ] Actor defined
- [ ] Goal articulated
- [ ] Flow drafted
- [ ] FRs identified
- [ ] Awaiting implementation & testing

---

## Journey Dependencies

### Cross-Journey Dependencies

```
phenotype-infrakit UJs (Foundation)
├─ UJ-001 → UJ-002 (setup before adding)
└─ UJ-003 depends on UJ-001 (setup before testing)

AgilePlus UJs
├─ AJ-001 → AJ-002 → AJ-003 (spec → implement → test)
└─ AJ-004 depends on AJ-001-003 (auto-merge needs working specs)

platforms/thegent UJs (Platform)
├─ TJ-002 depends on TJ-001 (load plugin after can execute)
└─ TJ-005 depends on TJ-001 (MCP tools used in execution)

heliosCLI UJs (Tool)
├─ HJ-001 → HJ-002 (install before running)
└─ HJ-003 depends on HJ-002 (sandbox within harness execution)
```

### Cross-Repository Journey Chains

```
Developer Setup (Cross-Repo):
phenotype-infrakit UJ-001 (workspace)
        ↓
heliosCLI HJ-001 (install CLI)
        ↓
heliosCLI HJ-002 (run harness)
        ↓
platforms/thegent TJ-001 (execute task)

Full Spec→Test Cycle (Cross-Repo):
AgilePlus AJ-001 (create spec)
        ↓
AgilePlus AJ-002 (implement)
        ↓
phenotype-infrakit UJ-003 (run tests)
        ↓
AgilePlus AJ-003 (trace FR↔Test)
```

---

## Metrics & SLAs

### Journey Success Rates

| Journey | Target SLR | Current | Status |
|---------|----------|---------|--------|
| UJ-INFRA-001 | 100% | 98% | ⚠️ (slight setup issues) |
| UJ-INFRA-003 (tests) | 100% | 100% | ✅ |
| AJ-004 (auto-merge) | 95% | N/A (draft) | 🔧 |
| TJ-001 (execute task) | 99.9% | 99.8% | ✅ |

### Journey Durations

| Journey | Target | Actual | SLA |
|---------|--------|--------|-----|
| UJ-001 (setup) | <15 min | 12 min | ✅ |
| AJ-001 (create spec) | <30 min | 28 min | ✅ |
| TJ-001 (execute) | <5 sec | 4.2 sec | ✅ |
| AJ-004 (auto-merge) | <5 min | N/A | 🔧 |

---

## Roadmap

### Q2 2026 Targets

| Journey | Current | Target | Action |
|---------|---------|--------|--------|
| AJ-004 | 🔧 Draft | ✅ Deployed | Implement auto-merge service |
| AJ-005 | 🔧 Draft | ✅ Deployed | Build health dashboard |
| AJ-006 | 🔧 Draft | ✅ Deployed | Create progress tracker |
| Overall | 92% | 100% | Complete AgilePlus journeys |

---

## Related Documents

- `USER_JOURNEYS.md` (per-repo) — Detailed journey definitions
- `FUNCTIONAL_REQUIREMENTS.md` — FRs per journey
- `PLAN.md` — Implementation phase mapping
- `ADR.md` — Architecture decisions for journeys

---

**Registry Owner:** Product Manager
**Last Updated:** 2026-04-01
**Next Review:** 2026-04-15
