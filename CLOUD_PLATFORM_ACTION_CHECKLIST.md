# Cloud Platform Implementation: Action Checklist

**Use this to move from research to implementation**

---

## ✅ Phase 0: Decision & Alignment (Today - 2 hours)

### 1. Share Research with Team
- [ ] Send `EXECUTIVE_BRIEF_CLOUD_AGENT_PLATFORM.md` to leadership
  - Time: 5 min read
  - Purpose: Get buy-in on 18:1 ROI
- [ ] Send `QUICK_START_CLOUD_AGENT_PLATFORM.md` to full team
  - Time: 5 min read
  - Purpose: Overview + next steps
- [ ] Send `MASTER_CLOUD_AGENT_PLATFORM_STRATEGY.md` to technical leads
  - Time: 30 min read
  - Purpose: Architecture + decision framework

### 2. Team Discussion (30 minutes)
- [ ] Schedule: "Cloud Agent Platform Go/No-Go Discussion"
- [ ] Attendees: Leadership + technical leads + engineering team
- [ ] Agenda:
  - Overview (5 min)
  - Q&A (15 min)
  - Decision: Go or No-Go? (10 min)

### 3. Decision Gate
- [ ] **Go approved?** YES / NO
- [ ] If YES → Proceed to Phase 1
- [ ] If NO → Document concerns, schedule follow-up

---

## ✅ Phase 1: Week 1 Planning (If Go) - 3 hours

### 1. Assign Week 1 Work
- [ ] Assign to Backend Engineer:
  - `.github/workflows/create-issue-on-ci-failure.yml`
  - `.github/workflows/create-issue-on-security-alert.yml`
  - `.github/workflows/create-issue-on-perf-regression.yml`
  - `.github/workflows/create-issue-on-review-threshold.yml`
  - `.github/workflows/create-issues-from-todos.yml`
  - Estimated effort: 6-8 hours
  - Deliverable: All workflows deployed + tested in staging repo

### 2. Assign Infrastructure Work
- [ ] Assign to Cloud Engineer:
  - Setup `.ai-routing.yaml` (provider routing config)
  - Setup `.ai-config.json` template
  - Implement PolyrepoRouter (from provider-routing research)
  - Estimated effort: 8-12 hours
  - Deliverable: Routing infrastructure ready for Week 2

### 3. Assign Directory Reorganization
- [ ] Assign to Architecture Lead:
  - Create `canonical/`, `infrastructure/`, `experimental/` dirs
  - Create `PROJECT.yml` files for Tier-1 projects
  - Update `REPOS_INDEX.md` with metadata
  - Estimated effort: 6-10 hours
  - Deliverable: Directory structure organized + navigation updated

### 4. Kick-Off Meeting (1 hour)
- [ ] Schedule: "Week 1 Kick-Off: Cloud Agent Platform"
- [ ] Attendees: All Week 1 assignees
- [ ] Agenda:
  - Sync on overall architecture (10 min)
  - Review assignments + deliverables (20 min)
  - Q&A + dependency mapping (20 min)
  - Timeline + daily check-ins (10 min)

### 5. Setup Communication Channels
- [ ] Create Slack channel: `#cloud-agents-dev`
- [ ] Create daily standup: 9am (15 min)
- [ ] Setup monitoring: Cost tracking spreadsheet / dashboard

---

## ✅ Phase 1: Week 1 Execution - Implementation

### Backend Engineer: GitHub Workflows
**Deliverable:** 5 workflows deployed + tested

**Workflow 1: CI Failure**
- [ ] Copy template from `GITHUB_AUTOMATION_AND_CLOUD_AGENT_INTEGRATION.md` (Part 1.1)
- [ ] Create `.github/workflows/create-issue-on-ci-failure.yml`
- [ ] Test: Manually trigger CI failure → verify issue created
- [ ] Validate: Issue has correct labels (type:bug, priority:high)

**Workflow 2: Security Alert**
- [ ] Copy template from doc (Part 1.2)
- [ ] Create `.github/workflows/create-issue-on-security-alert.yml`
- [ ] Test: Create Dependabot PR → verify issue created
- [ ] Validate: Issue has labels (type:security)

**Workflow 3: Performance Regression**
- [ ] Copy template from doc (Part 1.4)
- [ ] Create `.github/workflows/create-issue-on-perf-regression.yml`
- [ ] Test: Manually trigger perf failure → verify issue created

**Workflow 4: Code Review Threshold**
- [ ] Copy template from doc (Part 1.3)
- [ ] Create `.github/workflows/create-issue-on-review-threshold.yml`
- [ ] Test: Create PR with 20+ review comments → verify issue created

**Workflow 5: Inline TODOs**
- [ ] Copy template from doc (Part 1.5)
- [ ] Create `.github/workflows/create-issues-from-todos.yml`
- [ ] Test: Add TODO comment → push → verify issue created

**Validation:**
- [ ] All 5 workflows deployed
- [ ] All 5 tested in non-critical repo
- [ ] GitHub Actions logs show successful runs
- [ ] Issues created with correct labels/priorities
- [ ] Ready for Week 2 (Kilo routing)

---

### Cloud Engineer: Provider Routing Infrastructure
**Deliverable:** Routing system ready for Week 2

**Step 1: Configuration Schema**
- [ ] Create `.ai-routing.yaml` (template in MASTER document)
- [ ] Configure Tier-1 projects (phenotype-infrakit, AgilePlus, thegent)
- [ ] Set primary provider per repo (Claude Opus for critical, Sonnet for others)
- [ ] Set fallback providers (Groq, OpenAI as fallback)
- [ ] Set token budgets per repo (5M, 3M, 2M tokens/month)

**Step 2: Provider Routing Implementation**
- [ ] Copy PolyrepoRouter from PROVIDER_ROUTING_IMPLEMENTATION_GUIDE.md
- [ ] Implement in Python (src/routing/router.py)
- [ ] Implement budget tracking (src/routing/budget.py)
- [ ] Add provider client initialization (Claude, OpenAI, Groq)

**Step 3: Testing**
- [ ] Unit tests for router (test_router_selection.py)
- [ ] Unit tests for budget tracking (test_budget_manager.py)
- [ ] Test: Route code review task → verify Groq selected
- [ ] Test: Route security analysis → verify Claude selected
- [ ] Test: Verify budget tracking records usage

**Step 4: Documentation**
- [ ] Create `ROUTING_SETUP.md` (how to configure per-repo providers)
- [ ] Document budget caps + monitoring
- [ ] Create troubleshooting guide

**Validation:**
- [ ] Router implementation complete
- [ ] Budget tracking working
- [ ] All tests passing
- [ ] Ready for Week 2 (n8n integration)

---

### Architecture Lead: Directory Reorganization
**Deliverable:** Directory structure organized + navigation updated

**Step 1: Create Directory Structure**
- [ ] Create `/canonical/` dir
- [ ] Create `/infrastructure/` dir
- [ ] Create `/experimental/` dir
- [ ] Create `/archived/` dir
- [ ] Create `.worktrees/{canonical,infrastructure,experimental}/` subdirs

**Step 2: Create Metadata**
- [ ] Create `canonical/PROJECT.yml` (Tier-1 metadata)
- [ ] Create `infrastructure/PROJECT.yml` (Tier-2 metadata)
- [ ] Create `experimental/PROJECT.yml` (Tier-3 metadata)
- [ ] Create `phenotype-infrakit/PROJECT.yml` (per-project metadata)
- [ ] Create `AgilePlus/PROJECT.yml` (per-project metadata)
- [ ] Create `platforms/thegent/PROJECT.yml` (per-project metadata)

**Step 3: Create Navigation**
- [ ] Create `REPOS_INDEX.md` (master project list)
- [ ] Create `PROJECT_DISCOVERY.md` (onboarding guide)
- [ ] Create `WORKSPACE.code-workspace` (VS Code multi-root)

**Step 4: Create Scripts**
- [ ] Create `scripts/project-discovery.sh` (find all projects)
- [ ] Create `scripts/tree-command.sh` (formatted tree output)
- [ ] Create `scripts/project-metadata.sh` (query metadata)

**Step 5: Update Documentation**
- [ ] Update `CLAUDE.md` with new directory structure
- [ ] Update `AGENTS.md` with navigation guidance
- [ ] Create `MIGRATION_LOG.md` (what changed + why)

**Validation:**
- [ ] All new directories created
- [ ] All metadata files populated
- [ ] Navigation scripts working
- [ ] New agents can find projects within 2 minutes
- [ ] Ready for Week 2+ (seamless integration)

---

## ✅ Week 1 Check-In (End of Week)

**Friday 5pm Status Review:**
- [ ] All 5 GitHub workflows deployed + tested
- [ ] Provider routing implementation complete + tested
- [ ] Directory reorganization complete + documented
- [ ] All blocking issues resolved
- [ ] Team morale: High/Medium/Low?

**Go/No-Go for Week 2:**
- [ ] Success rate >90%? YES / NO
- [ ] No blocking bugs? YES / NO
- [ ] Team confident? YES / NO
- [ ] **Proceed to Week 2?** YES / NO

---

## ✅ Phase 2: Week 2 Planning (If Week 1 Success)

### Plan Week 2 (Kilo + n8n Deployment)
- [ ] Assign to DevOps:
  - Kilo account setup + auto-triage rules
  - n8n instance deployment (Docker or EC2)
  - GitHub → Kilo webhook registration
  - Kilo → n8n webhook configuration
  - Estimated: 8-12 hours

- [ ] Assign to Backend Engineer:
  - n8n workflow creation (github-issue-to-cloud-agent-dispatch.json)
  - Webhook testing + debugging
  - Estimated: 4-6 hours

### Week 2 Deliverables
- [ ] Kilo auto-triage live (issues labeled + assigned)
- [ ] n8n workflow created + tested
- [ ] GitHub → Kilo → n8n dispatch chain working
- [ ] Sample issue creation + routing verified

---

## ✅ Phase 3: Weeks 3-4 Planning (If Week 2 Success)

### Plan Cloud Agent Implementation
- [ ] Assign to Cloud Engineer (full time):
  - Claude Agent SDK setup
  - 5 agent implementations (bug, security, perf, review, refactor)
  - Feedback loop (agent → PR → comment → issue close)
  - Estimated: 24-32 hours

### Week 3-4 Deliverables
- [ ] Bug analysis agent working (70%+ success on test issues)
- [ ] Security remediation agent working (85%+ success)
- [ ] Performance optimization agent working (60%+ success)
- [ ] Code review agent working (80%+ success)
- [ ] Feedback loop tested (issue → agent → PR → merged)

---

## ✅ Phase 4: Weeks 5+ (If Phase 3 Success)

### Plan Scaling + Optimization
- [ ] Gas Town integration (Beads logging)
- [ ] Monitoring dashboard (Grafana)
- [ ] Cost tracking (per-repo, per-agent)
- [ ] Scale to Tier-1 projects (3 repos)
- [ ] Iterate + improve based on metrics
- [ ] Scale to Tier-2, then Tier-3 projects

---

## ✅ Success Criteria Checklist

### Week 1 Gate
- [ ] 5 GitHub workflows deployed
- [ ] Provider routing infrastructure ready
- [ ] Directory reorganized
- [ ] No critical bugs blocking Week 2

### Week 2 Gate
- [ ] 50+ issues created/week
- [ ] Kilo auto-triage working
- [ ] n8n routing working
- [ ] Ready for agent implementation

### Week 4 Gate
- [ ] 5 agents implemented
- [ ] 70%+ success rate on bugs
- [ ] 85%+ success rate on security
- [ ] Full feedback loop working

### Week 8 Gate
- [ ] 70%+ autonomous resolution rate
- [ ] Cost tracking accurate
- [ ] Zero security breaches
- [ ] Team confident in platform
- [ ] Ready for full rollout

---

## ✅ Key Contact Points

| Role | Assigned To | Email | Slack |
|------|-----------|-------|-------|
| **Project Lead** | [Your Name] | [Email] | [Handle] |
| **Backend Engineer** | [Name] | [Email] | [Handle] |
| **Cloud Engineer** | [Name] | [Email] | [Handle] |
| **DevOps** | [Name] | [Email] | [Handle] |
| **Architecture Lead** | [Name] | [Email] | [Handle] |

---

## ✅ Daily Check-In Template

**Time:** 9am, 15 minutes
**Format:** Standup + blockers

**Each person reports:**
1. What I completed yesterday
2. What I'm working on today
3. Blockers / help needed
4. Confidence: 🟢 On track / 🟡 At risk / 🔴 Blocked

**Example:**
```
Backend Engineer:
- Completed: CI failure workflow + testing
- Working: Security alert workflow
- Blocker: Need access to Dependabot PRs in test repo
- Confidence: 🟢 On track
```

---

## ✅ Weekly Status Report

**Time:** Friday 5pm
**Format:** Email to leadership

**Template:**
```
Week N Status Report: Cloud Agent Platform

✅ Completed This Week:
- GitHub workflows: 5/5 deployed
- Provider routing: Router implementation done, budget tracking done
- Directory reorganization: Structure created, metadata populated

🔄 In Progress:
- Integration testing (CI workflow)
- Kilo setup planning

⚠️ Blockers:
- [List any blocking issues]

📊 Metrics:
- Issues created: 47
- Agent success rate: N/A (Week 1)
- Cost: $0 (infrastructure setup)
- Team confidence: 🟢 High

🎯 Week 2 Plan:
- Kilo account setup
- n8n deployment
- Webhook integration
```

---

## ✅ Decision Tree: If Something Goes Wrong

**Workflow fails to create issues:**
1. Check GitHub Actions logs
2. Verify issue template is valid YAML
3. Check GitHub API token permissions
4. If still failing: Contact Backend Engineer + review workflow syntax

**Provider routing not selecting correct provider:**
1. Check `.ai-routing.yaml` syntax
2. Verify provider API keys are set
3. Check budget tracking (may have hit limits)
4. If still failing: Contact Cloud Engineer + review routing logic

**Kilo auto-triage not working:**
1. Check webhook delivery in Kilo dashboard
2. Verify webhook URL is correct
3. Check Kilo auth token
4. If still failing: Contact DevOps + Kilo support

**n8n workflow not firing:**
1. Check n8n workflow for errors
2. Verify webhook from Kilo is configured correctly
3. Test webhook manually with curl
4. If still failing: Contact Backend Engineer + review n8n logs

---

## ✅ Final Decision: Ready to Start?

**Checklist before Week 1:**
- [ ] Leadership approved (18:1 ROI accepted)
- [ ] Team bought in (understand goals + timeline)
- [ ] Budget approved ($200/month)
- [ ] Assignments made (Backend, Cloud, DevOps, Architect)
- [ ] Slack channel created (#cloud-agents-dev)
- [ ] Daily standups scheduled (9am)
- [ ] Weekly reports scheduled (Friday 5pm)
- [ ] All documents shared with team
- [ ] Week 1 kick-off meeting scheduled
- [ ] Template code copied from research documents

**If all checked:**
```
✅ READY TO START WEEK 1

→ Begin GitHub Actions workflow implementation
→ Deploy provider routing infrastructure
→ Reorganize directory structure
→ Check in daily at 9am
→ Report status Friday 5pm
→ Go/No-Go decision end of week
```

**If not all checked:**
```
⚠️ NOT READY YET

→ Address unchecked items
→ Schedule follow-up discussion
→ Document blockers
→ Reschedule Week 1 start date
```

---

**Printed:** 2026-03-30
**Last Updated:** 2026-03-30
**Status:** Ready for Team Use

🚀 **Questions? Check the research documents. Ready to go? Use this checklist. Let's build!**
