# Cloud Platform Integration Roadmap

**Complete Implementation Path: GitHub Automation + Cloud Agents + Gas Town + Directory Organization**

**Status:** 2026-03-30 | **Timeline:** 8-12 weeks | **Teams:** 2-3 engineers + 1 DevOps | **Cost:** $150-300/month

---

## Executive Overview

**Four parallel workstreams converge into a unified cloud agent platform:**

```
Workstream 1: GitHub Automation        Workstream 2: Cloud Agents
   └─ Event Generation                    └─ Provider Routing
   └─ Kilo Auto-Triage                    └─ Agent Implementation
   └─ n8n Workflows                       └─ Budget Tracking
              ↓                                    ↓
   ┌─────────────────────────────────────────────┐
   │     Integrated Platform (Week 4+)           │
   │  GitHub → Kilo → n8n → Cloud Agents        │
   │         ↓ (PRs, comments)                   │
   │      GitHub (closed loop)                   │
   └─────────────────────────────────────────────┘
              ↓
   Workstream 3: Gas Town Integration
   └─ Beads Event Logging
   └─ Mayor/Polecats Coordination
   └─ Multi-Agent Orchestration
              ↓
   Workstream 4: Directory Reorganization
   └─ Tier Taxonomy
   └─ Project Metadata
   └─ Navigation Helpers
```

**Parallel Execution Strategy:**
- **Workstream 1-2:** Run in parallel (no dependencies)
- **Workstream 3:** Start Week 4 (depends on 1+2 complete)
- **Workstream 4:** Can start Week 1 (independent)

**Critical Path:** Workstream 1 + 2 → Workstream 3 → Complete platform (8-10 weeks wall-clock, not serial)

---

## Phase 1: Foundation (Weeks 1-2)

### Workstream 1a: GitHub Event Generation

**Owner:** Backend Engineer
**Effort:** 6-8 hours
**Deliverables:**
- [ ] `.github/workflows/create-issue-on-ci-failure.yml` deployed
- [ ] `.github/workflows/create-issue-on-security-alert.yml` deployed
- [ ] `.github/workflows/create-issue-on-perf-regression.yml` deployed
- [ ] `.github/workflows/create-issue-on-review-threshold.yml` deployed
- [ ] `.github/workflows/create-issues-from-todos.yml` deployed
- [ ] Test in non-critical repo (manually trigger CI failure, verify issue)

**Cost:** $0

**Validation:** 50+ issues created/week from automated triggers

---

### Workstream 2a: Provider Routing Infrastructure

**Owner:** Cloud Engineer
**Effort:** 8-12 hours
**Deliverables:**
- [ ] `.ai-routing.yaml` global config created
- [ ] Per-repo `.ai-config.json` templates generated (for Tier-1, 2 projects)
- [ ] PolyrepoRouter Python implementation deployed
- [ ] Budget tracking (in-memory or Supabase) configured
- [ ] Cost calculator script (`scripts/estimate-cost.py`)
- [ ] Test: Route code review task to Groq, security analysis to Claude

**Cost:** $0 (infrastructure setup, no running costs yet)

**Validation:** Task routing working, budget tracking recording usage

---

### Workstream 4a: Directory Structure Phase 1

**Owner:** Architecture Lead
**Effort:** 4-6 hours
**Deliverables:**
- [ ] Create `/canonical/`, `/infrastructure/`, `/experimental/` dirs
- [ ] Create `REPOS_INDEX.md` (master navigation)
- [ ] Create `PROJECT_DISCOVERY.md` (onboarding guide)
- [ ] Create `DIRECTORY_STRUCTURE.md` (ASCII tree + rationale)
- [ ] Start creating `PROJECT.yml` files (start with Tier-1)
- [ ] Create `WORKSPACE.code-workspace` (VS Code multi-root)

**Cost:** $0

**Validation:** All new agents can quickly find projects via REPOS_INDEX.md

---

## Phase 2: Integration (Weeks 2-3)

### Workstream 1b: Kilo + n8n Deployment

**Owner:** DevOps / Backend Engineer
**Effort:** 8-12 hours
**Deliverables:**
- [ ] Kilo account provisioned + auto-triage rules configured
- [ ] n8n instance deployed (Docker or EC2 t3.small)
- [ ] GitHub → Kilo webhook registered
- [ ] Kilo → n8n webhook configured
- [ ] n8n workflow created: `github-issue-to-cloud-agent-dispatch.json`
- [ ] Test: Create security issue → auto-triage → n8n dispatch

**Cost:** +$100-150/month (Kilo Pro + n8n infrastructure)

**Validation:** Issue created → auto-triage label applied → n8n webhook fired

---

### Workstream 2b: Cloud Agent SDK Deployment

**Owner:** Cloud Engineer / Backend Engineer
**Effort:** 12-16 hours
**Deliverables:**
- [ ] Claude Agent SDK environment setup
- [ ] Groq API integration (if using)
- [ ] OpenAI API integration (if using)
- [ ] DeepSeek integration (if using)
- [ ] Agent base class implemented (template)
- [ ] Logging/error handling middleware
- [ ] Test harness for agent testing
- [ ] Deployment infrastructure (Lambda or server)

**Cost:** +$50-100/month (Claude API estimated)

**Validation:** Test agent can analyze a mock issue, produce output

---

### Workstream 4b: Directory Structure Phase 2

**Owner:** Architecture Lead
**Effort:** 6-8 hours
**Deliverables:**
- [ ] Complete all `PROJECT.yml` files (Tier-1, 2, 3)
- [ ] Create tier-level `PROJECT.yml` files (governance metadata)
- [ ] Create `.worktrees/` subdirs for each tier
- [ ] Validation CI workflow: `.github/workflows/validate-project-metadata.yml`
- [ ] Documentation: Update CLAUDE.md with new structure
- [ ] Create helper scripts: `project-discovery.sh`, `tree-command.sh`, `project-metadata.sh`

**Cost:** $0

**Validation:** All projects have machine-readable metadata, discovery script works

---

## Phase 3: Agent Implementation (Weeks 3-4)

### Workstream 2c: Cloud Agent Task Implementations

**Owner:** Cloud Engineers (1-2 agents)
**Effort:** 16-24 hours
**Deliverables:**

**Agent 1: Bug Analysis Agent**
- [ ] Fetch issue + related PRs
- [ ] Git bisect to find bug introduction
- [ ] Root cause analysis
- [ ] Write failing test
- [ ] Implement fix
- [ ] Run full test suite
- [ ] Create PR with explanation

**Agent 2: Security Remediation Agent**
- [ ] Identify vulnerability type
- [ ] Update dependencies OR implement code fix
- [ ] Run security scanners
- [ ] Verify no regressions
- [ ] Create PR + security tests

**Agent 3: Performance Optimization Agent**
- [ ] Run profiler (perf, flamegraph)
- [ ] Identify hotspots
- [ ] Implement optimizations
- [ ] Benchmark before/after
- [ ] Create PR + results

**Agent 4: Code Review Agent**
- [ ] Fetch PR diff + context
- [ ] Analyze for correctness, security, perf, style
- [ ] Post detailed review comment
- [ ] Suggest fixes if needed

**Agent 5: Refactoring Agent**
- [ ] Identify code smells
- [ ] Propose refactoring (extract methods, consolidate logic)
- [ ] Implement refactor
- [ ] Verify tests pass
- [ ] Create PR

**Cost:** Agent 1-4 included in budget; Agent 5 = additional $20-50/month

**Validation:** Each agent tested on 3-5 sample issues, 70%+ success rate

---

### Workstream 1c: Feedback Loop Implementation

**Owner:** Backend Engineer
**Effort:** 6-8 hours
**Deliverables:**
- [ ] Agent → PR creation workflow
- [ ] Agent → GitHub comment posting
- [ ] Agent → Issue status update (on PR merge, close issue)
- [ ] Error handling (if agent fails, create comment instead of PR)
- [ ] Test: Agent PR created → comment posted → issue closed on merge

**Cost:** $0

**Validation:** Full end-to-end: issue → agent → PR merge → issue closed

---

## Phase 4: Gas Town Integration (Weeks 4-5)

### Workstream 3: Gas Town Integration

**Owner:** Cloud Engineer / Architecture Lead
**Effort:** 8-12 hours
**Deliverables:**
- [ ] Beads event logging integrated into cloud agents
- [ ] `.events.jsonl` file populated after each agent run
- [ ] Gas Town Mayor configured to read GitHub issues + Beads logs
- [ ] Mayor dashboard connected to issue stream
- [ ] Polecat workers capable of spawning via Claude Agent SDK
- [ ] Test: Issue created → Beads event logged → Mayor dashboard updated

**Cost:** $0 (uses existing Beads infrastructure)

**Validation:** Beads events populating, Mayor dashboard showing issues + agent status

---

## Phase 5: Monitoring & Operations (Weeks 5-8)

### Workstream All: Monitoring, Dashboards, Documentation

**Owner:** DevOps + Full Team
**Effort:** 12-20 hours
**Deliverables:**
- [ ] Grafana dashboards (issues/day, agent success rate, cost, resolution time)
- [ ] Slack integration (daily reports, alerts)
- [ ] PagerDuty alerts (agent failures, budget overruns)
- [ ] Weekly review process (failed agents, prompt improvements)
- [ ] Documentation: agent playbooks, troubleshooting guide
- [ ] Onboarding: new agents learn how to use platform
- [ ] Cost tracking dashboard (per-repo, per-agent)

**Cost:** $0 (existing infrastructure)

**Validation:** Team can monitor platform health without manual intervention

---

## Phase 6: Scaling & Rollout (Weeks 6-12)

### Scale Across All Projects

**Timeline:**
- **Week 6:** Tier-1 projects (phenotype-infrakit, AgilePlus, thegent)
- **Week 7-8:** Tier-2 projects (heliosCLI, phenotype-docs)
- **Week 9-10:** Tier-3 projects (agent-wave, phench, bifrost)
- **Week 11-12:** Ecosystem partners + external repos

**Validation per Tier:**
- Tier-1: 70%+ success rate required before moving to Tier-2
- Tier-2: 65%+ success rate required before moving to Tier-3
- Tier-3: 60%+ success rate acceptable

**Feedback Loop:**
- Week 6: Collect data, improve agent prompts
- Week 7-8: Iterate on failed issues, adjust routing
- Week 9-10: Expand scope, add new agent types
- Week 11-12: Fine-tune for external partners

---

## Detailed Weekly Breakdown

### Week 1
```
Mon-Tue:   Workstream 1a (GitHub workflows)
Wed-Thu:   Workstream 2a (Provider routing setup)
Fri:       Workstream 4a (Directory structure prep)

Deliverable: 5 workflows deployed, routing config created, navigation docs drafted
Status: Ready for testing
```

### Week 2
```
Mon-Tue:   Test Workstream 1a in staging
Wed-Thu:   Workstream 1b (Kilo + n8n deployment)
Fri:       Workstream 4a finalization

Deliverable: Kilo + n8n live, 50+ issues created in test, directories organized
Status: Ready for agent implementation
```

### Week 3
```
Mon-Tue:   Workstream 2b (Agent SDK setup)
Wed-Thu:   Workstream 2c (Bug + Security agents)
Fri:       Code review + test

Deliverable: 2 agents working end-to-end, tested on 3-5 sample issues
Status: 70%+ success on bugs, 85%+ success on security
```

### Week 4
```
Mon-Tue:   Workstream 2c cont. (Perf + Code Review agents)
Wed:       Workstream 1c (Feedback loop)
Thu-Fri:   Workstream 3 (Gas Town integration)

Deliverable: All 5 agents working, full feedback loop, Beads integration
Status: Ready for Tier-1 rollout
```

### Weeks 5-8
```
Parallel:
- Monitoring dashboard setup
- Iterate on agents (weekly reviews)
- Scale to Tier-2, then Tier-3
- Documentation + runbooks

Success Metric: 70%+ autonomous resolution rate across Tier-1
```

### Weeks 9-12
```
Continue:
- Expand to all 30+ projects
- Partner with external teams
- Optimize for cost + performance
- Annual review + roadmap 2027

Success Metric: 50+ issues/day created, 35+ auto-resolved/day
```

---

## Risk Management

### Risk 1: GitHub Workflow Failures

**Mitigation:**
- Test each workflow in staging repo first
- Have manual fallback (create issue manually)
- Alert on workflow failures
- Disable problematic workflows immediately

**Contingency:** If workflows fail, revert to Phase 1a and debug before proceeding

---

### Risk 2: Agent Confidence (Merging Bad Fixes)

**Mitigation:**
- All agent PRs created as drafts
- Require manual approval for merge (first month)
- Full test suite must pass before PR creation
- Security scanning on all agent PRs

**Contingency:** If agent success rate <60%, pause deployment and improve prompts

---

### Risk 3: Budget Overruns

**Mitigation:**
- Hard cap at 95% of monthly budget per repo
- Alerts at 80%
- Daily cost reporting
- Automatic fallback to cheaper providers

**Contingency:** If cost exceeds budget, pause agent dispatch and debug

---

### Risk 4: Webhook Failures / Lost Events

**Mitigation:**
- Kilo has built-in retries
- n8n logs all workflows
- Daily reconciliation: count issues vs. expected
- Alerts on webhook delivery failures

**Contingency:** Manual replay of failed webhooks via CLI

---

### Risk 5: Directory Reorganization Breaks Git

**Mitigation:**
- Don't move; copy projects first, verify, then remove
- Update all `.git/config` references
- Test all worktrees before committing
- Document migration in MIGRATION_LOG.md

**Contingency:** If git breaks, revert to original structure

---

## Success Criteria

### End of Week 2
- [ ] 5 GitHub workflows deployed and tested
- [ ] 50+ issues created per week from automated triggers
- [ ] Kilo auto-triage working (labeling, assigning)
- [ ] n8n webhook firing on new issues
- [ ] All projects have machine-readable metadata

**Go/No-Go Decision:** Should we proceed to Phase 3?

### End of Week 4
- [ ] 5 cloud agents implemented and tested
- [ ] 70%+ success rate on bugs
- [ ] 85%+ success rate on security issues
- [ ] 60%+ success rate on performance
- [ ] Full end-to-end: issue → agent → PR → merged
- [ ] Beads event logging integrated

**Go/No-Go Decision:** Should we scale to Tier-1 projects?

### End of Week 8
- [ ] 70%+ autonomous resolution rate (issues closed without human review)
- [ ] Cost tracking accurate (< ±5%)
- [ ] Zero security breaches or bad merges
- [ ] Team trained on platform
- [ ] Documentation complete

**Go/No-Go Decision:** Should we expand to all 30 projects?

### End of Week 12
- [ ] 50-100 issues created/day
- [ ] 35+ issues resolved autonomously/day
- [ ] <$1 cost per resolved issue
- [ ] $4,000+/month savings
- [ ] 20:1 ROI (for every $1 spent, save $20)

**Success Declaration:** Cloud agent platform at scale

---

## Budget Summary

| Component | Cost/Month | Total (Year 1) | Notes |
|-----------|-----------|----------------|-------|
| Kilo Pro | $49-99 | $588-1,188 | Auto-triage |
| n8n Self-Hosted | $50-100 | $600-1,200 | EC2 + RDS |
| Claude API | $50-100 | $600-1,200 | ~1000 issues/mo |
| Groq API (optional) | $10-20 | $120-240 | For scale |
| Monitoring/Other | $0-25 | $0-300 | Supabase, etc. |
| **Total** | **$160-345** | **$1,908-4,128** | |

**Cost Avoidance:**
- Manual triage: 2 hrs/day × $50/hr × 240 days = **$24,000**
- Manual code review: 3 hrs/day × $50/hr × 240 days = **$36,000**
- Manual issue analysis: 1 hr/day × $50/hr × 240 days = **$12,000**
- **Total Savings (70% automation): $38,640**

**Net Year 1 Benefit:** $38,640 - $2,000 = **$36,640** (18x ROI)

---

## Team Structure

**Minimal Core Team:**
- **Cloud Engineer (1):** Weeks 1-12 continuous
  - Provider routing, agent implementation, Gas Town integration
  - 40% capacity initially, 20% maintenance Week 8+
- **Backend Engineer (1):** Weeks 1-4
  - GitHub workflows, Kilo/n8n setup, feedback loop
  - Then transitions to monitoring/operations
- **DevOps (0.5):** Weeks 2-12
  - Infrastructure (n8n, monitoring), cost tracking
  - On-call for production issues

**Recommended Parallel Work:**
- Agents start work on agents (Workstream 2b) while backend finalizes GitHub automation (Workstream 1b)
- No serialization; run in true parallel

---

## Communication Plan

**Weekly Team Standup (15 min):**
- What shipped this week?
- What's blocked?
- Metrics (issues created, resolved, success rate, cost)

**Stakeholder Updates (Monthly):**
- Executive: ROI, cost, issues resolved
- Product: Feature velocity impact
- Ops: System health, alerts, incidents

**Slack Channels:**
- `#cloud-agents-dev` — Development team
- `#cloud-agents-alerts` — Production alerts
- `#cloud-agents-metrics` — Daily metrics reports

---

## References

1. **MASTER_CLOUD_AGENT_PLATFORM_STRATEGY.md** — Complete platform blueprint
2. **GITHUB_AUTOMATION_AND_CLOUD_AGENT_INTEGRATION.md** — GitHub workflows + webhooks
3. **DIRECTORY_REORGANIZATION_AND_TRAVERSABILITY.md** — Directory structure + metadata
4. **PROVIDER_PER_REPO_ROUTING.md** — Provider routing code (in progress)
5. **AI_CODE_PLATFORMS_COMPARISON_2026.md** — Provider evaluation
6. **CLOUD_WORKFLOW_ORCHESTRATION_RESEARCH.md** — n8n + Kilo + Gas Town

---

## Decision Point: Go/No-Go?

**Recommendation:** **GO**

**Rationale:**
1. ✅ Research complete (9 comprehensive documents)
2. ✅ Team capacity available (1 cloud engineer, 1 backend engineer)
3. ✅ ROI clear (18x Year 1, $36k+ net benefit)
4. ✅ Technical feasibility confirmed (no blocker risks)
5. ✅ Parallel execution possible (Workstreams 1-2-4 independent)
6. ✅ Gas Town integration adds significant value (parallel execution)
7. ✅ Directory reorganization improves future onboarding

**Next Step:** Kick-off meeting with team, assign Week 1 work, start parallel execution.

---

**Document Version:** 1.0
**Last Updated:** 2026-03-30
**Status:** Ready for Implementation
**Next Review:** End of Week 2 (Go/No-Go decision)
