# GitHub Automation + Cloud Agent Integration Strategy

**Status:** 2026-03-30 | **Scope:** Phenotype Ecosystem | **Priority:** CRITICAL

---

## Executive Summary

**Problem:** Cloud agents (Claude, OpenAI, Kilo) sit idle because Phenotype doesn't generate GitHub issues. Kilo's auto-triage feature is disabled. Webhooks have nothing to trigger.

**Solution:** Implement **GitHub-first event generation** with automatic issue creation from:
- CI failures
- Security alerts
- Code review thresholds
- Performance regressions
- Inline todos

**Impact:**
- Kilo auto-triage routes issues to cloud agents
- Cloud agents autonomously analyze, fix, test
- Zero-touch issue resolution (70%+ automation rate targeted)
- Cost: $50-100/month (Kilo + n8n self-hosted)

**Timeline:** 4-week implementation (Phase 1: event generation, Phase 2: cloud agent routing, Phase 3: feedback loops)

---

## Part 1: GitHub Issues Auto-Creation

### 1.1 CI Failure Issues

**Trigger:** GitHub Actions workflow fails

**Workflow:** `.github/workflows/create-issue-on-ci-failure.yml`

```yaml
name: Create Issue on CI Failure

on:
  workflow_run:
    workflows: ['CI', 'Lint', 'Test', 'Build']
    types: [completed]

jobs:
  create-issue:
    if: failure() || github.event.workflow_run.conclusion == 'failure'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Get workflow run data
        id: workflow
        uses: actions/github-script@v7
        with:
          script: |
            const run = await github.rest.actions.getWorkflowRun({
              owner: context.repo.owner,
              repo: context.repo.repo,
              run_id: context.payload.workflow_run.id
            });

            const logs = await github.rest.actions.downloadWorkflowRunLogs({
              owner: context.repo.owner,
              repo: context.repo.repo,
              run_id: context.payload.workflow_run.id
            });

            return {
              workflow_name: run.data.name,
              conclusion: run.data.conclusion,
              run_url: run.data.html_url,
              logs: logs.data.toString().substring(0, 2000)  // First 2K of logs
            };

      - name: Create issue
        uses: actions/github-script@v7
        with:
          script: |
            const { workflow_name, conclusion, run_url, logs } = ${{ steps.workflow.outputs.result }};

            const issue = await github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: `CI Failure: ${workflow_name} failed`,
              body: `## Workflow Run Failed

**Workflow:** ${workflow_name}
**Status:** ${conclusion}
**Run:** [View on GitHub](${run_url})

### Error Log (first 2000 chars)
\`\`\`
${logs}
\`\`\`

**Auto-triaged by:** GitHub Actions
**Assigned to:** @kilo-auto-triage
`,
              labels: ['type:bug', 'priority:high', 'source:ci-failure'],
              assignees: ['bot-agent']  // Assign to agent
            });

            core.setOutput('issue_number', issue.data.number);
            core.setOutput('issue_url', issue.data.html_url);
```

**Issue Template:**
```
Title: CI Failure: [workflow_name] failed
Labels: type:bug, priority:high, source:ci-failure
Assignee: bot-agent
Body: Failure details + log excerpt + link to run
```

**Routing:** Issue created → Kilo webhook → n8n workflow → Claude Agent

---

### 1.2 Security Alerts Issues

**Trigger:** Dependabot PR, CodeQL alert, Snyk scan

**Workflow:** `.github/workflows/create-issue-on-security-alert.yml`

```yaml
name: Create Issue on Security Alert

on:
  pull_request:
    types: [opened]
    paths:
      - 'Cargo.lock'
      - 'package-lock.json'
      - 'go.mod'

jobs:
  check-security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Check for Dependabot PR
        id: dependabot
        if: github.actor == 'dependabot[bot]'
        run: |
          echo "is_security_pr=true" >> $GITHUB_OUTPUT

      - name: Create security issue
        if: steps.dependabot.outputs.is_security_pr == 'true'
        uses: actions/github-script@v7
        with:
          script: |
            const pr_title = context.payload.pull_request.title;
            const pr_url = context.payload.pull_request.html_url;
            const pr_body = context.payload.pull_request.body;

            // Extract vulnerability severity from Dependabot PR title
            const severity = pr_title.includes('critical') ? 'critical'
                           : pr_title.includes('high') ? 'high'
                           : 'medium';

            const priority = severity === 'critical' ? 'priority:critical'
                           : severity === 'high' ? 'priority:high'
                           : 'priority:medium';

            const issue = await github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: `Security: ${pr_title}`,
              body: `## Security Update Required

**Dependency Update:** [${pr_title}](${pr_url})

${pr_body.substring(0, 500)}

**Severity:** ${severity}
**Source:** Dependabot
**PR:** [View PR](${pr_url})

**Action Required:**
- [ ] Review changes
- [ ] Run security tests
- [ ] Merge if safe

**Auto-remediation:** Cloud agent will test and merge if security tests pass.
`,
              labels: ['type:security', priority, 'source:dependabot'],
              assignees: ['bot-agent']
            });
```

**Issue Types:**
- `type:security/dependency` — Dependency updates
- `type:security/vulnerability` — Code vulnerabilities (CodeQL)
- `type:security/container` — Container scanning alerts
- `type:security/sast` — Static analysis findings

---

### 1.3 Code Review Threshold Issues

**Trigger:** PR review comment count exceeds threshold

**Workflow:** `.github/workflows/create-issue-on-review-threshold.yml`

```yaml
name: Create Issue on Review Threshold

on:
  pull_request_review:
    types: [submitted]

jobs:
  check-review-count:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/github-script@v7
        with:
          script: |
            // Get PR review comments
            const reviews = await github.rest.pulls.listReviews({
              owner: context.repo.owner,
              repo: context.repo.repo,
              pull_number: context.payload.pull_request.number
            });

            const comment_count = reviews.data.reduce(
              (sum, r) => sum + (r.body?.split('\n').length || 0), 0
            );

            // Threshold: >20 comments indicates significant issues
            if (comment_count > 20) {
              const issue = await github.rest.issues.create({
                owner: context.repo.owner,
                repo: context.repo.repo,
                title: `Code Review: [PR #${context.payload.pull_request.number}] Many comments (${comment_count})`,
                body: `## Code Review Feedback Volume

**PR:** [#${context.payload.pull_request.number}](${context.payload.pull_request.html_url})
**Review Comments:** ${comment_count}

This PR has received significant review feedback. Consider:
- Splitting into smaller PRs
- Refactoring problematic areas
- Requesting architectural review

**Cloud Agent Action:** Analyze feedback and suggest refactoring strategy.
`,
                labels: ['type:review-feedback', 'priority:medium'],
                assignees: ['bot-agent']
              });
            }
```

**Trigger Thresholds:**
- >20 comments → "many review comments"
- >5 review rounds → "needs refactoring"
- Changes to 3+ files with 50%+ churn → "large refactor"

---

### 1.4 Performance Regression Issues

**Trigger:** Benchmark test fails (>10% regression)

**Workflow:** `.github/workflows/create-issue-on-perf-regression.yml`

```yaml
name: Create Issue on Performance Regression

on:
  workflow_run:
    workflows: ['Benchmarks']
    types: [completed]

jobs:
  check-perf:
    if: failure()
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Extract benchmark results
        id: benchmark
        run: |
          # Assuming benchmark output in artifacts/benchmark_results.json
          if [ -f artifacts/benchmark_results.json ]; then
            REGRESSION=$(jq '.regression_percent' artifacts/benchmark_results.json)
            if (( $(echo "$REGRESSION > 10" | bc -l) )); then
              echo "is_regression=true" >> $GITHUB_OUTPUT
              echo "regression_percent=$REGRESSION" >> $GITHUB_OUTPUT
            fi
          fi

      - name: Create performance issue
        if: steps.benchmark.outputs.is_regression == 'true'
        uses: actions/github-script@v7
        with:
          script: |
            const regression = ${{ steps.benchmark.outputs.regression_percent }};
            const commit_sha = context.payload.workflow_run.head_commit.id.substring(0, 8);

            const issue = await github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: `Perf Regression: ${regression.toFixed(1)}% slowdown detected`,
              body: `## Performance Degradation Detected

**Commit:** ${commit_sha}
**Regression:** ${regression.toFixed(1)}% slower than baseline
**Benchmark:** [View Results](https://github.com/${ context.repo.owner }/${ context.repo.repo }/actions/runs/${ context.payload.workflow_run.id })

**Profiling Needed:**
- [ ] CPU hotspots (flamegraph)
- [ ] Memory allocations
- [ ] Lock contention (if concurrent)
- [ ] I/O patterns

**Cloud Agent Action:** Profile code, identify bottlenecks, suggest optimizations.
`,
              labels: ['type:performance', 'priority:high'],
              assignees: ['bot-agent']
            });
```

**Trigger Thresholds:**
- >10% regression → create issue
- >20% regression → also ping team in Slack
- Memory regression >15% → also add `priority:critical`

---

### 1.5 Inline Todo Extraction

**Trigger:** Code push with `// TODO:`, `# TODO:`, `/* TODO: */`

**Workflow:** `.github/workflows/create-issues-from-todos.yml`

```yaml
name: Create Issues from TODOs

on: [push, pull_request]

jobs:
  extract-todos:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Extract TODOs
        id: todos
        run: |
          # Find all TODO comments in source files
          grep -r "TODO:" src/ --include="*.rs" --include="*.go" --include="*.py" --include="*.ts" \
            -n | while read line; do

            FILE=$(echo $line | cut -d: -f1)
            LINE_NUM=$(echo $line | cut -d: -f2)
            TODO_TEXT=$(echo $line | cut -d: -f3- | sed 's/^[[:space:]]*//')

            # Create GitHub issue for each TODO
            echo "Creating issue for: $FILE:$LINE_NUM - $TODO_TEXT"
          done

      - name: Create GitHub issues
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const path = require('path');

            // Find all TODO comments
            function findTodos(dir, files = []) {
              const entries = fs.readdirSync(dir, { withFileTypes: true });
              entries.forEach(entry => {
                if (entry.isDirectory() && !entry.name.startsWith('.')) {
                  findTodos(path.join(dir, entry.name), files);
                } else if (['.rs', '.go', '.py', '.ts'].includes(path.extname(entry.name))) {
                  const fullPath = path.join(dir, entry.name);
                  const content = fs.readFileSync(fullPath, 'utf-8');
                  const lines = content.split('\n');

                  lines.forEach((line, idx) => {
                    const match = line.match(/TODO[:\s]+(.+?)(?:\s*\*\/|$)/i);
                    if (match) {
                      const todoText = match[1].trim();
                      files.push({
                        file: fullPath,
                        line: idx + 1,
                        text: todoText
                      });
                    }
                  });
                }
              });
              return files;
            }

            const todos = findTodos('src');

            for (const todo of todos) {
              const issue = await github.rest.issues.create({
                owner: context.repo.owner,
                repo: context.repo.repo,
                title: `TODO: ${todo.text.substring(0, 60)}`,
                body: `## Code TODO

**File:** \`${todo.file}:${todo.line}\`
**Text:** ${todo.text}

[View in code](https://github.com/${ context.repo.owner }/${ context.repo.repo }/blob/${ context.ref }/${ todo.file }#L${ todo.line })

**Status:** Extracted from inline comment
`,
                labels: ['type:todo', 'priority:low'],
                assignees: ['bot-agent']
              });
            }
```

---

## Part 2: Webhook Routing Architecture

### 2.1 GitHub → Kilo Auto-Triage

**Setup:** GitHub Webhook → Kilo Instance

```yaml
# .github/workflows/setup-kilo-webhook.yml
name: Setup Kilo Webhook

on: [workflow_dispatch]

jobs:
  setup:
    runs-on: ubuntu-latest
    steps:
      - name: Register Kilo webhook
        run: |
          curl -X POST https://api.kilo.ai/v1/webhooks \
            -H "Authorization: Bearer ${{ secrets.KILO_API_KEY }}" \
            -H "Content-Type: application/json" \
            -d '{
              "url": "https://kilo.ai/webhooks/github",
              "events": [
                "github.issues.opened",
                "github.issues.labeled",
                "github.pull_request.opened"
              ],
              "config": {
                "auto_triage": true,
                "triage_rules": [
                  {
                    "label": "type:bug",
                    "assign_to": "bug-triage-agent",
                    "priority": "high"
                  },
                  {
                    "label": "type:security",
                    "assign_to": "security-agent",
                    "priority": "critical"
                  },
                  {
                    "label": "type:performance",
                    "assign_to": "perf-agent",
                    "priority": "high"
                  }
                ]
              }
            }'
```

**Kilo Auto-Triage Features (enabled by GitHub issues):**
- Auto-label by type/priority
- Auto-assign to appropriate agent pool
- Auto-add to project board
- Auto-set milestone based on priority
- Auto-request reviewer assignment

---

### 2.2 Kilo Auto-Triage → n8n Dispatch

**Setup:** Kilo → n8n Webhook

**n8n Workflow:** `github-issue-to-cloud-agent.json`

```json
{
  "name": "GitHub Issue → Cloud Agent Dispatch",
  "nodes": [
    {
      "name": "GitHub Webhook",
      "type": "n8n-nodes-base.webhook",
      "position": [250, 300],
      "typeVersion": 1,
      "webhookId": "github-issues",
      "options": {
        "events": ["issues.opened", "issues.labeled"],
        "path": "github/issues"
      }
    },
    {
      "name": "Extract Issue Type",
      "type": "n8n-nodes-base.code",
      "position": [450, 300],
      "typeVersion": 1,
      "parameters": {
        "jsCode": "return {\n  issue_number: $input.body.issue.number,\n  title: $input.body.issue.title,\n  body: $input.body.issue.body,\n  labels: $input.body.issue.labels.map(l => l.name),\n  type: $input.body.issue.labels.find(l => l.name.startsWith('type:'))?.name,\n  priority: $input.body.issue.labels.find(l => l.name.startsWith('priority:'))?.name,\n  repo: $input.body.repository.full_name\n};"
      }
    },
    {
      "name": "Route by Issue Type",
      "type": "n8n-nodes-base.switch",
      "position": [650, 300],
      "typeVersion": 1,
      "parameters": {
        "cases": [
          {
            "condition": "={{ $json.type === 'type:bug' }}",
            "output": 0
          },
          {
            "condition": "={{ $json.type === 'type:security' }}",
            "output": 1
          },
          {
            "condition": "={{ $json.type === 'type:performance' }}",
            "output": 2
          }
        ]
      }
    },
    {
      "name": "Dispatch to Bug Analysis Agent",
      "type": "n8n-nodes-base.httpRequest",
      "position": [850, 150],
      "typeVersion": 4.1,
      "parameters": {
        "method": "POST",
        "url": "{{ $env.CLAUDE_AGENT_URL }}/dispatch",
        "authentication": "genericCredentialType",
        "genericAuthType": "httpHeaderAuth",
        "sendHeaders": true,
        "headerParameters": {
          "Authorization": "Bearer {{ $env.CLAUDE_API_KEY }}"
        },
        "body": "{\n  \"agent_type\": \"bug-analyzer\",\n  \"task\": {\n    \"issue_number\": {{ $json.issue_number }},\n    \"title\": \"{{ $json.title }}\",\n    \"description\": \"{{ $json.body }}\",\n    \"repo\": \"{{ $json.repo }}\"\n  }\n}"
      }
    },
    {
      "name": "Dispatch to Security Agent",
      "type": "n8n-nodes-base.httpRequest",
      "position": [850, 300],
      "typeVersion": 4.1,
      "parameters": {
        "method": "POST",
        "url": "{{ $env.CLAUDE_AGENT_URL }}/dispatch",
        "authentication": "genericCredentialType",
        "sendHeaders": true,
        "body": "{\n  \"agent_type\": \"security-remediation\",\n  \"task\": {\n    \"issue_number\": {{ $json.issue_number }},\n    \"title\": \"{{ $json.title }}\",\n    \"description\": \"{{ $json.body }}\",\n    \"repo\": \"{{ $json.repo }}\"\n  }\n}"
      }
    },
    {
      "name": "Dispatch to Perf Agent",
      "type": "n8n-nodes-base.httpRequest",
      "position": [850, 450],
      "typeVersion": 4.1,
      "parameters": {
        "method": "POST",
        "url": "{{ $env.CLAUDE_AGENT_URL }}/dispatch",
        "sendHeaders": true,
        "body": "{\n  \"agent_type\": \"perf-optimizer\",\n  \"task\": {\n    \"issue_number\": {{ $json.issue_number }},\n    \"title\": \"{{ $json.title }}\",\n    \"description\": \"{{ $json.body }}\",\n    \"repo\": \"{{ $json.repo }}\"\n  }\n}"
      }
    },
    {
      "name": "Log Dispatch",
      "type": "n8n-nodes-base.merge",
      "position": [1050, 300]
    }
  ],
  "connections": {
    "GitHub Webhook": {
      "main": [
        [{ "node": "Extract Issue Type", "index": 0 }]
      ]
    },
    "Extract Issue Type": {
      "main": [
        [{ "node": "Route by Issue Type", "index": 0 }]
      ]
    },
    "Route by Issue Type": {
      "main": [
        [{ "node": "Dispatch to Bug Analysis Agent", "index": 0 }],
        [{ "node": "Dispatch to Security Agent", "index": 0 }],
        [{ "node": "Dispatch to Perf Agent", "index": 0 }]
      ]
    },
    "Dispatch to Bug Analysis Agent": {
      "main": [
        [{ "node": "Log Dispatch", "index": 0 }]
      ]
    },
    "Dispatch to Security Agent": {
      "main": [
        [{ "node": "Log Dispatch", "index": 0 }]
      ]
    },
    "Dispatch to Perf Agent": {
      "main": [
        [{ "node": "Log Dispatch", "index": 0 }]
      ]
    }
  }
}
```

---

## Part 3: Cloud Agent Task Definitions

### 3.1 Bug Analysis Agent

**Trigger:** `type:bug` issue created

**Agent Prompt:**
```markdown
You are a bug analysis and triage agent. You have access to:
- Git history (last 50 commits)
- Issue tracker (related issues)
- Code repository (full source)
- Test suite (run locally)

**Task:** Analyze GitHub issue #{{ issue_number }}: "{{ title }}"

**Steps:**
1. Read issue description and reproduce the bug locally
2. Search git history for when bug was introduced (git bisect)
3. Identify root cause in source code
4. Check if similar bugs exist elsewhere
5. Write unit test that reproduces the bug
6. Implement minimal fix
7. Run full test suite to ensure no regressions
8. Create PR with fix + test + explanation

**Output:** PR link + comment on issue with root cause analysis

**Definition of Done:**
- [ ] Bug reproduced in test
- [ ] Root cause identified
- [ ] Fix implemented and tested
- [ ] PR created
- [ ] PR linked in issue comment
```

### 3.2 Security Remediation Agent

**Trigger:** `type:security` issue created

**Agent Prompt:**
```markdown
You are a security remediation agent. You automatically fix security vulnerabilities.

**Task:** Analyze GitHub issue #{{ issue_number }}: "{{ title }}"

**Steps:**
1. Identify vulnerability type (dependency, code, container, etc.)
2. For dependencies: update to secure version, run security tests
3. For code vulnerabilities: implement fix per OWASP guidance
4. Run security scanners (CodeQL, Snyk, Semgrep)
5. Verify no new vulnerabilities introduced
6. Create PR with fix + security test
7. Request urgent review

**Output:** PR link + Slack notification

**Acceptance Criteria:**
- [ ] Vulnerability fixed
- [ ] Security tests pass
- [ ] No new CVEs introduced
- [ ] PR ready for review
```

### 3.3 Performance Optimization Agent

**Trigger:** `type:performance` issue created

**Agent Prompt:**
```markdown
You are a performance optimization agent.

**Task:** Analyze GitHub issue #{{ issue_number }}: "{{ title }}" - "{{ description }}"

**Steps:**
1. Run profiler (perf, flamegraph, criterion)
2. Identify top 3 hotspots
3. Research optimization techniques (algorithm, caching, parallelization)
4. Implement optimizations
5. Benchmark before/after
6. Create PR with performance improvements
7. Document optimization strategy

**Output:** PR with benchmark results + optimization report

**Success Criteria:**
- [ ] >10% performance improvement demonstrated
- [ ] Benchmarks show improvement
- [ ] Code quality maintained
- [ ] Tests pass
```

---

## Part 4: Feedback Loop: Agent → GitHub

### 4.1 Agent PR Creation

**After analysis, agent automatically:**

```python
async def create_pr_from_analysis(analysis_result: AnalysisResult):
    """Create PR from cloud agent analysis."""

    # Create branch
    branch_name = f"agent/{analysis_result.issue_type}/{analysis_result.issue_number}"

    # Commit changes
    subprocess.run([
        "git", "checkout", "-b", branch_name,
        "origin/main"
    ])
    subprocess.run(["git", "add", "-A"])
    subprocess.run([
        "git", "commit", "-m",
        f"fix({analysis_result.issue_type}): {analysis_result.title}\n\nCloses #{analysis_result.issue_number}"
    ])

    # Push
    subprocess.run(["git", "push", "-u", "origin", branch_name])

    # Create PR
    pr = github.rest.pulls.create(
        owner=repo.owner,
        repo=repo.repo,
        title=f"[Agent] Fix: {analysis_result.title}",
        body=f"""## Automated Fix

**Issue:** #{analysis_result.issue_number}
**Type:** {analysis_result.issue_type}
**Agent:** {analysis_result.agent_type}

### Analysis
{analysis_result.analysis_text}

### Changes
{analysis_result.changes_summary}

### Testing
{analysis_result.test_results}

---
This PR was automatically generated by cloud agent {analysis_result.agent_type}.
Please review and test before merging.
""",
        head=branch_name,
        base="main",
        draft=False
    )

    # Link to issue
    github.rest.issues.createComment(
        owner=repo.owner,
        repo=repo.repo,
        issue_number=analysis_result.issue_number,
        body=f"🤖 Cloud agent has analyzed this issue and created a fix PR: #{pr.number}\n\n{analysis_result.brief_analysis}"
    )

    return pr
```

### 4.2 Agent Issue Closing

**When fix is complete:**

```python
async def close_issue_when_fixed(issue_number: int, pr_number: int):
    """Auto-close issue when agent PR is merged."""

    # Wait for PR merge
    pr = github.rest.pulls.get(owner, repo, pr_number)

    if pr.merged:
        github.rest.issues.createComment(
            owner=repo.owner,
            repo=repo.repo,
            issue_number=issue_number,
            body=f"✅ Fixed by PR #{pr_number}. Closing issue."
        )

        github.rest.issues.update(
            owner=repo.owner,
            repo=repo.repo,
            issue_number=issue_number,
            state="closed"
        )
```

---

## Part 5: Kilo Configuration

### 5.1 Kilo Auto-Triage Rules

**File:** `.kilo/auto-triage.yaml`

```yaml
version: 1

triggers:
  - event: issue.opened
    rules:
      - name: Bug Triage
        if:
          label: type:bug
        then:
          - action: assign
            to: [bug-analysis-agent]
          - action: set-priority
            value: high
          - action: add-to-project
            project: "Backlog"
          - action: request-review
            from: [engineering-team]

      - name: Security Triage
        if:
          label: type:security
        then:
          - action: assign
            to: [security-agent]
          - action: set-priority
            value: critical
          - action: add-to-project
            project: "Security"
          - action: notify
            slack: "#security-alerts"
          - action: request-review
            from: [security-team]

      - name: Performance Triage
        if:
          label: type:performance
        then:
          - action: assign
            to: [perf-agent]
          - action: set-priority
            value: high
          - action: add-to-project
            project: "Performance"

webhooks:
  - name: Cloud Agent Dispatch
    url: "https://n8n.phenotype.internal/github/issues"
    events: [issue.opened, issue.labeled]
    auth:
      type: bearer
      token: "${N8N_WEBHOOK_TOKEN}"

reporting:
  - type: daily
    recipient: "#ci-notifications"
    metrics:
      - issues_created
      - issues_resolved
      - agent_success_rate
      - average_resolution_time
```

---

## Part 6: Implementation Roadmap

### Phase 1: Event Generation (Week 1-2)

**Deliverables:**
- [ ] `.github/workflows/create-issue-on-ci-failure.yml` deployed
- [ ] `.github/workflows/create-issue-on-security-alert.yml` deployed
- [ ] `.github/workflows/create-issue-on-review-threshold.yml` deployed
- [ ] `.github/workflows/create-issue-on-perf-regression.yml` deployed
- [ ] `.github/workflows/create-issues-from-todos.yml` deployed
- [ ] Test: manually trigger one failure, verify issue created

**Cost:** $0 (GitHub Actions)

### Phase 2: Webhook Routing (Week 2-3)

**Deliverables:**
- [ ] Kilo instance configured with auto-triage rules
- [ ] n8n instance deployed (self-hosted Docker)
- [ ] Webhook from GitHub → Kilo verified
- [ ] Webhook from Kilo → n8n verified
- [ ] Issue routing by type confirmed
- [ ] Test: create security issue, verify auto-triage + n8n dispatch

**Cost:** $100-150/month (n8n EC2, Kilo subscription)

### Phase 3: Cloud Agent Integration (Week 3-4)

**Deliverables:**
- [ ] Claude Agent SDK deployed for bug analysis
- [ ] Security agent implemented
- [ ] Performance agent implemented
- [ ] Agents can: analyze → fix → test → create PR
- [ ] Feedback loop: agent PR → comment on issue
- [ ] Test: Create 5 sample issues, verify agents handle end-to-end

**Cost:** $50-100/month (Claude API)

### Phase 4: Monitoring & Iteration (Week 4+)

**Metrics:**
- Issues created per day (target: 10-20)
- Agent success rate (target: 70%+)
- Average resolution time (target: <2 hours)
- Cost per issue (target: <$0.50)

---

## Part 7: Integration with Gas Town

**Gas Town Beads Tracking:**

Once GitHub issues are flowing, integrate with Gas Town's Beads system:

```python
# In cloud agent, after issue analysis:

async def update_beads_after_analysis(issue_number: int, analysis_result: AnalysisResult):
    """Record agent work in Beads for Gas Town tracking."""

    beads_event = {
        "type": "agent_analysis_complete",
        "timestamp": datetime.now().isoformat(),
        "github_issue": issue_number,
        "agent_type": analysis_result.agent_type,
        "analysis_summary": analysis_result.brief_analysis,
        "pr_created": analysis_result.pr_number if analysis_result.pr_created else None,
        "success": analysis_result.success
    }

    # Record in Beads
    async with aiohttp.ClientSession() as session:
        await session.post(
            f"{BEADS_BASE_URL}/v1/events",
            json=beads_event,
            headers={"Authorization": f"Bearer {BEADS_API_KEY}"}
        )

    # This creates an audit trail in Beads that Gas Town's Mayor/Polecats can track
```

**Gas Town Mayor Integration:**

When GitHub issue is created, Gas Town Mayor can:
1. Read Beads event log
2. See that cloud agent is handling it
3. Spawn Polecat workers for parallel fixes if needed
4. Track progress in Mayor dashboard
5. Route to human review if agent fails

---

## Part 8: Cost Breakdown

**Monthly Cost Estimate:**

| Component | Cost | Notes |
|-----------|------|-------|
| GitHub Actions | $0 | Free for workflows |
| Kilo (auto-triage) | $49-99 | Pro plan |
| n8n Self-Hosted | $50-100 | EC2 t3.small + RDS |
| Claude API (agents) | $50-100 | ~1000 issues/month × $0.05-0.10 per analysis |
| **Total** | **$150-300/month** | vs. $14,400+ for GitHub Actions at scale |

**Cost Avoidance:**
- Manual triage: ~2 hours/day × $50/hour = $500/month (saved)
- Manual review: ~5 hours/day × $50/hour = $1,250/month (saved)
- **Total Savings: ~$1,500-2,000/month**

---

## Part 9: Success Metrics

### First Month (Baseline)
- [ ] 50+ issues created from CI/security/perf triggers
- [ ] 70%+ successfully auto-triaged by Kilo
- [ ] 30+ dispatched to cloud agents
- [ ] 60%+ agent analysis provided feedback on issues
- [ ] 20%+ agent-generated PRs created

### Second Month (Improvement)
- [ ] 100+ issues created/month
- [ ] 85%+ auto-triage success rate
- [ ] 70%+ dispatched to cloud agents
- [ ] 75%+ agent analysis provided
- [ ] 40%+ agent-generated PRs
- [ ] <$1/issue cost

### Third Month (Scaling)
- [ ] 200+ issues created/month
- [ ] 90%+ auto-triage success
- [ ] 80%+ agent-handled
- [ ] 70%+ agent-generated PRs
- [ ] <$0.50/issue cost
- [ ] 24h average resolution time

---

## Part 10: Deployment Checklist

**Week 1: Event Generation**
- [ ] Copy `.github/workflows/create-issue-*.yml` files
- [ ] Test on non-critical repo first
- [ ] Verify issues being created
- [ ] Iterate on issue template clarity

**Week 2: Kilo Setup**
- [ ] Sign up for Kilo account
- [ ] Configure auto-triage rules
- [ ] Register GitHub webhook
- [ ] Test: create issue, verify auto-triage

**Week 3: n8n Setup**
- [ ] Deploy n8n (Docker or EC2)
- [ ] Create GitHub webhook workflow
- [ ] Test routing: GitHub → n8n
- [ ] Verify issue type detection

**Week 4: Cloud Agents**
- [ ] Deploy Claude Agent SDK
- [ ] Implement bug analysis agent
- [ ] Implement security agent
- [ ] Implement performance agent
- [ ] Test end-to-end: issue → agent → PR

**Week 4+: Monitor & Improve**
- [ ] Track metrics (issues/day, agent success rate)
- [ ] Iterate on agent prompts
- [ ] Add new issue types as needed
- [ ] Scale to all repos

---

## References

- **Gas Town Documentation:** https://docs.gastownhall.ai/
- **Kilo Auto-Triage:** https://docs.kilo.ai/auto-triage
- **n8n GitHub Integration:** https://docs.n8n.io/integrations/builtin/app-nodes/n8n-nodes-base.github/
- **Claude Agent SDK:** https://platform.anthropic.com/docs/en/agent-sdk/overview
- **GitHub Actions Webhooks:** https://docs.github.com/en/developers/webhooks-and-events/

---

**Document Version:** 1.0
**Last Updated:** 2026-03-30
**Author:** Cloud Agent Orchestration Research Team
