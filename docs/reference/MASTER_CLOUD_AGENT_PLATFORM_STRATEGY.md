# Master Cloud Agent Platform Strategy

**Phenotype Ecosystem — Complete Cloud Agent Orchestration Blueprint**

**Status:** 2026-03-30 | **Scope:** All 30+ repos | **Investment:** $150-300/month | **ROI:** $1,500-2,000/month savings

---

## Executive Summary

**The Gap:** You have 30+ repos with no GitHub issues, no webhooks, and cloud agents sitting idle. Kilo's auto-triage feature is unused. Code review is rate-limited and manual.

**The Solution:** Implement a **complete cloud agent platform** with:
1. **GitHub-first event generation** (CI failures, security alerts, performance regressions, code reviews)
2. **Intelligent webhook routing** (GitHub → Kilo → n8n → Cloud agents)
3. **Multi-provider agent dispatch** (Claude for reasoning, Groq for speed, OpenAI for specific tasks)
4. **Autonomous issue resolution** (70%+ of issues auto-fixed, tested, PR'd)
5. **Gas Town integration** (Mayor/Polecats for parallel work coordination)

**Impact:**
- **70%+ of issues resolved autonomously** (no human triage needed)
- **2-3x faster code review** (parallel agents vs. sequential humans)
- **Zero rate-limiting** (cloud agents handle overflow from CodeRabbit/Snyk)
- **$1,500-2,000/month savings** vs. manual processes
- **24h average issue resolution** (from creation to closed PR)

---

## Architecture Overview

```
GitHub Events (CI, Security, Perf)
         ↓
    [Event Generator Workflows]
         ↓
GitHub Issues Created (labeled by type/priority)
         ↓
    [Kilo Webhook] → Auto-triage + label assignment
         ↓
    [n8n Workflow] → Route by issue type
         ↓
Cloud Agents (Claude/Groq/OpenAI)
    ├─ Bug Analysis Agent
    ├─ Security Remediation Agent
    ├─ Performance Optimization Agent
    ├─ Code Review Agent
    └─ Refactoring Agent
         ↓
    [Autonomous Action]
    ├─ Analyze issue
    ├─ Implement fix
    ├─ Run tests
    ├─ Create PR
    └─ Post analysis comments
         ↓
GitHub (PR merged, issue closed)
         ↓
    [Beads Event Log] → Gas Town tracks progress
         ↓
Gas Town Mayor/Polecats (coordinate multi-agent work)
```

---

## Part 1: Provider Selection Matrix

### 1.1 Which Provider for Which Task?

**Code Analysis & Reasoning:**
```
Task: Bug analysis, security remediation, performance optimization
Provider: Claude Opus 4 (best reasoning)
Cost: $0.015/1K input, $0.075/1K output
Use case: Complex analysis, multi-step reasoning
```

**High-Volume Code Review:**
```
Task: 50+ PR reviews/day, quick surface-level analysis
Provider: Groq (sub-100ms latency)
Cost: $0.0001/1K tokens (100x cheaper)
Use case: Parallel reviews, real-time feedback
```

**Specific Code Generation:**
```
Task: Generate tests, refactor code snippets
Provider: OpenAI GPT-4 (good for code)
Cost: $0.01/1K input, $0.03/1K output
Use case: Well-scoped code generation tasks
```

**Cost-Sensitive Tasks:**
```
Task: Batch analysis, documentation generation
Provider: DeepSeek v3 (cheapest)
Cost: $0.0001/1K input, $0.0003/1K output
Use case: High-volume, low-risk tasks
```

### 1.2 Provider-Per-Repo Routing Configuration

**File:** `.ai-routing.yaml` (deploy at repos root)

```yaml
version: 1

repositories:
  phenotype-infrakit:
    tier: tier-1
    primary_provider: claude-opus-4
    fallback_providers: [claude-sonnet-4, gpt-4-turbo]
    monthly_budget: 5000000  # tokens
    task_routing:
      code_review: gpt-4-turbo
      security: claude-opus-4
      performance: groq
      testing: deepseek-v3

  AgilePlus:
    tier: tier-1
    primary_provider: claude-opus-4
    fallback_providers: [claude-sonnet-4]
    monthly_budget: 3000000
    task_routing:
      code_review: gpt-4-turbo
      testing: groq

  heliosCLI:
    tier: tier-2
    primary_provider: claude-sonnet-4
    fallback_providers: [gpt-4-turbo, groq]
    monthly_budget: 2000000
    task_routing:
      code_review: groq
      documentation: deepseek-v3

  # ... other repos
```

**Cost Impact:**
- **Before:** Single provider, token waste, no optimization → $500-1000/month
- **After:** Provider routing, budget tracking, tiering → $150-300/month
- **Savings:** 65-70% cost reduction

---

## Part 2: GitHub Automation Framework

### 2.1 Event Sources

**Automated Issue Creation:**
```
✓ CI Failures        → type:bug label → Bug Analysis Agent
✓ Security Alerts    → type:security label → Security Agent
✓ Perf Regressions   → type:performance label → Perf Agent
✓ Code Review        → type:review-feedback label → Code Agent
✓ Inline TODOs       → type:todo label → Backlog Agent
```

**Implementation:** 5 GitHub Actions workflows (see GitHub automation doc)

### 2.2 Webhook Chain

```
GitHub Issue Created
    ↓
    [Kilo Webhook] ← Register at kilo.ai
    ├─ Label detection (type:*, priority:*)
    ├─ Auto-assignment (to agent pool)
    ├─ Add to project board
    ├─ Set priority/milestone
    └─ Trigger n8n webhook
         ↓
    [n8n Workflow] ← Self-hosted @ n8n.phenotype.internal
    ├─ Extract issue metadata
    ├─ Route by issue type
    ├─ Enrich with context (repo, files, tests)
    └─ Dispatch to cloud agent
         ↓
    [Cloud Agent] ← Claude/Groq/OpenAI
    ├─ Analyze
    ├─ Implement fix
    ├─ Test
    ├─ Create PR
    └─ Comment on issue
         ↓
GitHub (PR merge, issue close)
```

---

## Part 3: Cloud Agent Task Definitions

### 3.1 Bug Analysis Agent

```
Input: GitHub issue #N (type:bug)
Process:
  1. Fetch issue details + related PRs
  2. Reproduce locally (git clone, build, test)
  3. Git bisect to find when bug was introduced
  4. Identify root cause in source code
  5. Write failing test that reproduces bug
  6. Implement minimal fix
  7. Run full test suite
  8. Create PR with fix + test

Output: PR link, comment on issue with root cause
Cost: ~$0.10-0.30 per bug analysis
Time: 5-15 minutes (depending on complexity)
Success Rate Target: 70%+ (rest require human review)
```

### 3.2 Security Remediation Agent

```
Input: GitHub issue #N (type:security)
Process:
  1. Identify vulnerability type
  2. For dependencies: update to secure version
  3. For code: implement fix per OWASP
  4. Run security scanners (CodeQL, Snyk, Semgrep)
  5. Verify no regressions
  6. Create PR with fix + security tests

Output: PR link, security report comment
Cost: ~$0.15-0.50 per vulnerability
Time: 10-30 minutes
Success Rate: 85%+ (most deps/code vulns fixable)
```

### 3.3 Performance Optimization Agent

```
Input: GitHub issue #N (type:performance)
Process:
  1. Run profiler (perf, flamegraph, criterion)
  2. Identify top 3 hotspots
  3. Implement optimizations (algorithm, caching, parallelization)
  4. Benchmark before/after
  5. Create PR with improvements

Output: PR with benchmark results, optimization report
Cost: ~$0.20-0.50 per optimization
Time: 15-45 minutes
Success Rate: 60%+ (depends on complexity)
```

### 3.4 Code Review Agent

```
Input: GitHub PR #N
Process:
  1. Fetch diff + context
  2. Analyze for:
     - Logic correctness
     - Security issues
     - Performance problems
     - Code style/idioms
     - Test coverage
  3. Post detailed review comment
  4. Request changes if needed

Output: Review comment + suggested fixes (if applicable)
Cost: ~$0.05-0.15 per PR
Time: 2-10 minutes
Success Rate: 80%+ (catches most common issues)
```

---

## Part 4: Kilo Auto-Triage Configuration

**File:** `.kilo/auto-triage.yaml`

```yaml
triage_rules:
  - name: Bug Triage
    if:
      - label: type:bug
      - priority: high
    then:
      - assign: bug-analysis-agent
      - add_to_project: "Bug Backlog"
      - dispatch_to: n8n-bug-workflow

  - name: Security Triage
    if:
      - label: type:security
    then:
      - assign: security-agent
      - priority: critical
      - add_to_project: "Security"
      - notify_slack: "#security-alerts"
      - dispatch_to: n8n-security-workflow

  - name: Performance Triage
    if:
      - label: type:performance
    then:
      - assign: perf-agent
      - priority: high
      - dispatch_to: n8n-perf-workflow

webhooks:
  - name: n8n-dispatch
    url: "https://n8n.phenotype.internal/github/issues"
    events: [issue.opened, issue.labeled]
```

---

## Part 5: Implementation Timeline

### Week 1: GitHub Event Generation (est. 6-8 hours)

**Deliverables:**
- [ ] `.github/workflows/create-issue-on-ci-failure.yml`
- [ ] `.github/workflows/create-issue-on-security-alert.yml`
- [ ] `.github/workflows/create-issue-on-perf-regression.yml`
- [ ] `.github/workflows/create-issue-on-review-threshold.yml`
- [ ] `.github/workflows/create-issues-from-todos.yml`
- [ ] Test each workflow in a non-critical repo

**Validation:** Manually trigger CI failure, verify issue created with correct labels

**Cost:** $0

### Week 2: Kilo + n8n Setup (est. 8-12 hours)

**Deliverables:**
- [ ] Kilo account configured
- [ ] Auto-triage rules deployed (`.kilo/auto-triage.yaml`)
- [ ] n8n instance deployed (Docker on EC2)
- [ ] GitHub → Kilo webhook registered
- [ ] Kilo → n8n webhook registered
- [ ] Issue routing tested (security issue → security workflow)

**Validation:** Create test security issue, verify auto-triage + n8n dispatch

**Cost:** $100-150/month (Kilo + n8n infrastructure)

### Week 3: Cloud Agent Integration (est. 12-20 hours)

**Deliverables:**
- [ ] Claude Agent SDK deployed
- [ ] Bug analysis agent implemented
- [ ] Security remediation agent implemented
- [ ] Performance optimization agent implemented
- [ ] Code review agent implemented
- [ ] Agent → GitHub feedback loop (create PR, post comment)
- [ ] Provider routing configured (`.ai-routing.yaml`)
- [ ] Budget tracking enabled (Supabase or local)

**Validation:** Create 5 sample issues, verify agents handle end-to-end

**Cost:** $50-100/month (Claude API)

### Week 4: Gas Town Integration + Monitoring (est. 8-12 hours)

**Deliverables:**
- [ ] Beads event log integration
- [ ] Gas Town Mayor dashboard reading GitHub issue events
- [ ] Monitoring dashboards (issues/day, agent success rate, cost per issue)
- [ ] Slack alerts (high-priority issues, agent failures)
- [ ] Daily report generation (cost, issues resolved, agent performance)

**Validation:** Verify Beads event log populating, Gas Town dashboard reading events

**Cost:** $0 (uses existing Beads infrastructure)

---

## Part 6: Cost Breakdown

### Monthly Operating Cost

| Component | Cost | Notes |
|-----------|------|-------|
| Kilo Pro (auto-triage) | $49-99 | Scales with repos |
| n8n Self-Hosted | $50-100 | EC2 t3.small + RDS |
| Claude API | $50-100 | ~1000 issues × $0.05-0.10 |
| Groq API (optional) | $10-20 | For high-volume code review |
| GitHub Actions | $0 | Free for workflows |
| Supabase (budget tracking) | $0-25 | Optional monitoring |
| **Total** | **$160-345/month** | |

### Comparison to Alternative Approaches

| Approach | Cost | Issues Resolved | Manual Work |
|----------|------|-----------------|-------------|
| **Cloud Agent Platform (proposed)** | $200/month | 70%+ autonomously | 30% human review |
| GitHub Actions + CodeRabbit | $500/month | 20% (review only) | 80% manual triage |
| Manual + Dedicated DevOps | $10,000+/month | 50% (slow) | 50% human |
| Do Nothing | $0 | 0% | 100% manual |

### ROI Calculation

**Savings Breakdown:**
- Manual triage: 2 hrs/day × $50/hr × 20 days = **$2,000/month**
- Manual code review: 3 hrs/day × $50/hr × 20 days = **$3,000/month**
- Manual issue analysis: 1 hr/day × $50/hr × 20 days = **$1,000/month**
- **Total Potential Savings: $6,000/month**

**After 70% Automation:**
- **Actual Savings: $6,000 × 0.70 = $4,200/month**
- **Platform Cost: $200/month**
- **Net Savings: $4,000/month** (or $48,000/year)
- **ROI: 20:1** (for every $1 spent, save $20)

---

## Part 7: Risk Mitigation

### Risk 1: Agent Over-Confidence (Merging Bad Fixes)

**Mitigation:**
- All agent PRs created as `draft: true`
- Require manual approval before merge
- Agents run full test suite before creating PR
- Automated security scanning on agent PRs
- Target: <5% of agent PRs need fixes

### Risk 2: Webhook Failures / Lost Events

**Mitigation:**
- Kilo has built-in retry logic (3x with exponential backoff)
- n8n logs all workflows, can replay on failure
- GitHub → Kilo failover to direct n8n webhook
- Alerts on webhook delivery failures
- Daily reconciliation: count created issues vs. expected

### Risk 3: Budget Overruns

**Mitigation:**
- Per-repo token budgets enforced (`.ai-routing.yaml`)
- Hard stop at 95% of monthly budget
- Daily alerts when approaching 80%
- Automatic fallback to cheaper providers
- Weekly cost report with breakdown by repo/agent

### Risk 4: Agent Quality Issues

**Mitigation:**
- Start with small test set (10 issues/week)
- Monitor success rate (target: 70%+ for bugs, 85%+ for security)
- If success rate <60%, pause and debug
- Feedback loop: failed PRs → agent prompt improvement
- Human review on all merges for first month

---

## Part 8: Monitoring & Metrics

### Key Performance Indicators

```
Operational:
  - Issues created/day (target: 20-50)
  - Issues resolved by agents/day (target: 70%+ of created)
  - Average resolution time (target: <24h)
  - Agent success rate (target: 70%+)
  - PR merge rate from agents (target: 60%+)

Financial:
  - Cost per issue (target: <$0.50)
  - Cost per resolved issue (target: <$0.70)
  - Cost per PR merged (target: <$1.00)
  - Monthly cost (budget: $200-300)

Quality:
  - Bug detection accuracy (target: 90%+)
  - Security fix correctness (target: 95%+)
  - Test coverage improvement (target: +10%)
  - False positive rate (target: <10%)
```

### Dashboard

**Slack bot reports (daily 9am):**
```
📊 Cloud Agent Daily Report

Issues Created: 27 (target: 20-50) ✓
  - Bugs: 15
  - Security: 3
  - Performance: 2
  - Review Feedback: 4
  - TODOs: 3

Issues Resolved: 19 (70% resolution rate) ✓
  - Bugs fixed: 11 (agent PRs merged: 9)
  - Security fixed: 3 (all merged)
  - Performance: 2 (1 merged, 1 in review)

Avg Resolution Time: 8.2 hours ✓
Agent Success Rate: 73% (11/15 bugs) ✓
Cost This Month: $156 (budget: $300) ✓

Failed Agents (requiring review):
  - Bug #1234: Root cause unclear, needs human analysis
  - Bug #1245: Test fails after fix, needs refactor
```

**Web Dashboard (Grafana or AgilePlus):**
- Real-time issue/agent status
- Cost trends over time
- Success rate by agent type
- Resolution time percentiles (p50, p90, p99)

---

## Part 9: Gas Town Integration

### 9.1 Why Gas Town?

Gas Town is a **multi-agent orchestration framework** that perfectly complements your cloud agent platform:

**Without Gas Town:** One agent handles one issue serially
```
Issue #1 (bug) → Agent analyzes → 30 min
Issue #2 (security) → Agent analyzes → 30 min
Issue #3 (perf) → Agent analyzes → 30 min
Total time: 90 min
```

**With Gas Town Mayor/Polecats:** Multiple agents handle issues in parallel
```
Mayor (orchestrator) receives 3 issues
  ├─ Spawn Polecat-1 (bug analyzer) for Issue #1
  ├─ Spawn Polecat-2 (security agent) for Issue #2
  └─ Spawn Polecat-3 (perf agent) for Issue #3
Total time: 30 min (3x faster)
```

### 9.2 Integration Points

**Beads Event Log (Gas Town's persistence):**
```python
# After agent analysis completes:
beads_event = {
    "type": "agent_work_complete",
    "issue_number": 123,
    "agent_type": "bug_analyzer",
    "result": "fix_implemented",
    "pr_number": 456,
    "analysis_time_seconds": 450
}

# Post to Beads
POST /beads/v1/events
Authorization: Bearer <BEADS_API_KEY>
Body: beads_event
```

**Mayor Dashboard (read from Beads):**
- Mayor queries Beads for all agent work
- Displays: issues analyzed, agents running, PRs created
- Can spawn additional agents if issues exceed throughput

**Polecat Workers (use cloud agent SDK):**
- Each Polecat is an instance of Claude Agent SDK
- Mayor assigns tasks via Beads work queue
- Polecats update Beads with progress/completion

---

## Part 10: Full Implementation Checklist

### Pre-Deployment (Week 0)
- [ ] Get team buy-in on cloud agent strategy
- [ ] Allocate budget ($200-300/month)
- [ ] Set up monitoring dashboard
- [ ] Document runbooks for debugging

### Phase 1: GitHub Event Generation (Week 1)
- [ ] Deploy 5 GitHub Actions workflows
- [ ] Test CI failure → issue creation
- [ ] Test security alert → issue creation
- [ ] Test perf regression → issue creation
- [ ] Iterate on issue templates

### Phase 2: Kilo + n8n (Week 2)
- [ ] Set up Kilo account
- [ ] Configure auto-triage rules
- [ ] Deploy n8n (Docker)
- [ ] Register webhooks
- [ ] Test routing: issue → triage → n8n dispatch

### Phase 3: Cloud Agents (Week 3)
- [ ] Deploy Claude Agent SDK
- [ ] Implement all 5 agent types
- [ ] Add provider routing
- [ ] Configure budget tracking
- [ ] Test end-to-end: issue → agent → PR → merged

### Phase 4: Gas Town + Monitoring (Week 4)
- [ ] Integrate Beads event logging
- [ ] Set up Grafana/dashboard
- [ ] Configure Slack alerts
- [ ] Deploy daily reporting

### Post-Launch Ops (Ongoing)
- [ ] Monitor metrics daily
- [ ] Review failed agents weekly
- [ ] Improve agent prompts based on failures
- [ ] Scale to all 30 repos by Week 8
- [ ] Expand to Phenotype ecosystem partners by Week 12

---

## Appendix A: Quick Start

### 1. Copy GitHub workflow files
```bash
cd repos
curl -o .github/workflows/create-issue-on-ci-failure.yml \
  https://raw.githubusercontent.com/KooshaPari/phenotype-infrakit/main/.github/workflows/create-issue-on-ci-failure.yml
# ... repeat for other 4 workflows
```

### 2. Create `.ai-routing.yaml`
```bash
# Copy template from docs/reference/
cp docs/reference/MASTER_CLOUD_AGENT_PLATFORM_STRATEGY.md
# Edit .ai-routing.yaml section
```

### 3. Deploy n8n
```bash
docker run -d \
  -p 5678:5678 \
  -e DB_TYPE=postgres \
  -e DB_POSTGRESDB_HOST=postgres \
  n8nio/n8n:latest
```

### 4. Start cloud agents
```bash
python agent-platform/main.py \
  --config .ai-routing.yaml \
  --providers claude,openai,groq \
  --listen 0.0.0.0:8000
```

---

## References

1. **AI Provider APIs** — `docs/research/AI_CODE_PLATFORMS_COMPARISON_2026.md`
2. **Code Review Tools** — `docs/research/CODE_REVIEW_TOOLS_COMPARISON.md`
3. **Cloud Orchestration** — `docs/reference/ORCHESTRATION_INDEX.md`
4. **GitHub Automation** — `docs/reference/GITHUB_AUTOMATION_AND_CLOUD_AGENT_INTEGRATION.md`
5. **Gas Town** — `docs/research/GASTOWN_RESEARCH_SUMMARY.md`

---

**Next Steps:**

1. **Review** this document with team (30 min)
2. **Decide** approval to proceed (15 min)
3. **Assign** Phase 1 work (see checklist above)
4. **Launch** Week 1: GitHub event generation
5. **Monitor** daily metrics
6. **Iterate** weekly on agent prompts

**Questions?** See individual research documents or reach out to cloud agent team.

---

**Document Version:** 1.0
**Last Updated:** 2026-03-30
**Author:** Cloud Agent Orchestration Research Team
**Status:** Ready for Implementation
