# Phase 1 Team Communication Guide

**Purpose:** Provide templates, talking points, and communication strategies for Phase 1 rollout
**Audience:** Team leads, engineers, security team, platform team
**Duration:** 2026-03-30 → 2026-04-13

---

## Daily Standup Template (Async Slack)

**Channel:** `#security-qa-phase1`
**Time:** EOD (5 PM each day) or EOW (Friday EOD for weekly)
**Format:** Slack thread

### Template

```
📊 PHASE 1 STANDUP — [DATE] [DAY]
═════════════════════════════════════

✅ COMPLETED TODAY:
   • [Specific repos/tasks completed]
   • [Metrics: e.g., "SAST deployed to 3 repos"]
   • [Any blockers resolved]

🟡 IN PROGRESS:
   • [Current deployment/activity]
   • [Expected completion time]
   • [Owner: @name]

🔴 BLOCKERS:
   • [List any issues]
   • [Impact: e.g., "Blocks SAST Tier 2"]
   • [Mitigation: "Escalating to security team"]
   • [ETA to resolve]

📊 METRICS (Daily):
   • SAST: X/30 repos (↑ from Y)
   • Snyk: A/30 repos (↑ from B)
   • Sentry: C/30 repos (↑ from D)
   • Linting: E/30 repos (↑ from F)

👥 NEXT SHIFT OWNER: @name
🔗 TRACKER: [Link to PHASE1_TRACKER.yml]

Questions? React with 👍 or reply in thread.
```

### Examples

**Monday, March 31**
```
📊 PHASE 1 STANDUP — Monday, March 31
═════════════════════════════════════

✅ COMPLETED TODAY:
   • Followed up with security team on Snyk token (escalated)
   • Followed up with platform team on Sentry DSNs (escalated)
   • Started SAST Tier 2 deployment

🟡 IN PROGRESS:
   • SAST Tier 2 deployment (phenotype-shared, agent-wave)
   • Sentry SDK testing in phenotype-infrakit (waiting for DSN)

🔴 BLOCKERS:
   • SNYK_TOKEN not received (CRITICAL)
     Impact: Cannot deploy Snyk to any repo
     Mitigation: Escalated to security-lead; target 2026-04-01
   • Sentry DSNs not provided (MEDIUM)
     Impact: SDKs ready but cannot initialize
     Mitigation: Escalated to platform-lead; bulk provisioning target 2026-04-01

📊 METRICS (Daily):
   • SAST: 3/30 repos (↑ from 3)
   • Snyk: 0/30 repos (blocked)
   • Sentry: 1/30 repos (↑ from 0, SDK only)
   • Linting: 0/30 repos (frameworks ready)

👥 NEXT SHIFT OWNER: @infra-team
🔗 TRACKER: PHASE1_TRACKER.yml

Questions? React with 👍 or reply in thread.
```

---

## Weekly Summary Email

**To:** Team + Engineering Lead + Security Lead
**Subject:** `PHASE 1 Weekly Summary — Week [N]`
**Cadence:** Every Friday EOD

### Template

```
PHASE 1 WEEKLY SUMMARY — Week [N]
═══════════════════════════════════════════════════════════

📊 METRICS SNAPSHOT
───────────────────

Tool       │ Current    │ Target   │ % to Target │ Status
───────────┼────────────┼──────────┼─────────────┼────────
SAST       │  X/30     │  30/30   │    TBD%     │ 🟢
Snyk       │  Y/30     │  30/30   │    TBD%     │ 🔴 Blocked
Sentry     │  Z/30     │  30/30   │    TBD%     │ 🟡
Linting    │  W/30     │  30/30   │    TBD%     │ 🟡

Overall Progress: (X+Y+Z+W)/120 tools deployed

🟢 WINS
───────
  ✅ [Major achievement 1]
  ✅ [Major achievement 2]
  ✅ Team adoption training completed

🔴 BLOCKERS (ACTIVE)
──────────────────────
  1. Snyk token missing (CRITICAL)
     - Owner: security-team
     - Target Resolution: 2026-04-01
     - Impact: -3 days if unresolved
     - Escalation: [contact info]

  2. Sentry DSNs missing (MEDIUM)
     - Owner: platform-team
     - Target Resolution: 2026-04-01
     - Impact: -2 days if unresolved
     - Escalation: [contact info]

📋 THIS WEEK'S FOCUS
────────────────────
  • [Tool 1: Tier X deployment]
  • [Tool 2: Tier Y deployment]
  • [Blocker resolution]

🎯 NEXT WEEK'S PLAN
───────────────────
  • [Tool 1: Tier X+1 deployment]
  • [Tool 2: Tier Y+1 deployment]
  • [Quality validation]

🚦 GO/NO-GO GATE
────────────────
  Result: [🟢 GO | 🟡 CONDITIONAL | 🔴 NO-GO]

  Criteria Met:
    ☑ SAST deployment: X/12 repos
    ☑ Tool integration: Y%
    ☑ Zero blocking issues: [status]
    ☑ Team satisfaction: [status]

  Decision: [Proceed to next week / Reassess / Defer]

📞 ESCALATIONS
──────────────
  None at this time. (Or list any escalations needed.)

---

Questions? Reply-all with your feedback.
Team Lead: @name
```

### Example: Week 1 Summary

```
PHASE 1 WEEKLY SUMMARY — Week 1 (Mar 30 - Apr 6)
═══════════════════════════════════════════════════════════

📊 METRICS SNAPSHOT
───────────────────

Tool       │ Current    │ Target   │ % to Target │ Status
───────────┼────────────┼──────────┼─────────────┼────────
SAST       │  12/30    │  30/30   │    40%      │ 🟢 On Track
Snyk       │   0/30    │  30/30   │     0%      │ 🔴 Blocked
Sentry     │   3/30    │  30/30   │    10%      │ 🟡 On Track (if DSN)
Linting    │   3/30    │  30/30   │    10%      │ 🟡 On Track

Overall Progress: 18/120 tools deployed (15%)

🟢 WINS
───────
  ✅ SAST deployed to Tier 1 + Tier 2 (12 repos)
  ✅ Linting frameworks adopted on Tier 1 (3 repos)
  ✅ Sentry SDK tested on phenotype-infrakit
  ✅ Team training sessions completed (attendance: 95%)
  ✅ Zero critical security issues discovered in SAST scan

🔴 BLOCKERS (ACTIVE)
──────────────────────
  1. Snyk token missing (CRITICAL)
     - Owner: security-team (security-lead@company.com)
     - Target Resolution: 2026-04-01 09:00 AM
     - Impact: -3 days if unresolved (16 hrs lost to Snyk Tier 1-2)
     - Action: Escalate again if not received by EOD 2026-03-31
     - Escalation: engineering-lead@company.com

  2. Sentry DSNs missing (MEDIUM)
     - Owner: platform-team (platform-lead@company.com)
     - Target Resolution: 2026-04-01 12:00 PM
     - Impact: -2 days if unresolved (19.5 hrs lost to Sentry Tier 1-3)
     - Action: Bulk DSN provisioning required
     - Escalation: devops-lead@company.com

📋 THIS WEEK'S FOCUS
────────────────────
  • SAST Tier 2 deployment: ✅ COMPLETE
  • Linting Tier 1 adoption: ✅ COMPLETE
  • Snyk & Sentry blocker resolution: 🔴 PENDING

🎯 NEXT WEEK'S PLAN
───────────────────
  • SAST Tier 3 deployment (if on schedule)
  • Snyk Tier 1-2 deployment (if token arrives)
  • Sentry Tier 1-2 deployment (if DSNs arrive)
  • Linting Tier 2 adoption
  • Quality validation & metrics review

🚦 GO/NO-GO GATE — WEEK 1
────────────────────────────
  Result: 🟡 CONDITIONAL GO

  Criteria Met:
    ✅ SAST deployment: 12/12 Tier 1+2 repos
    ✅ Tool integration: 100% for SAST & Linting
    ✅ Linting adoption: 3/3 Tier 1 repos
    ⏳ Zero blocking issues: Snyk & Sentry tokens pending
    ⏳ Team satisfaction: Survey scheduled Week 2

  Decision: Proceed to Week 2 with contingency plan for blocker recovery

📞 ESCALATIONS
──────────────
  ⚠️ CRITICAL: Follow up with security-team on Snyk token (EOD 2026-03-31)
  ⚠️ CRITICAL: Follow up with platform-team on Sentry DSNs (EOD 2026-03-31)

---

Questions? Reply-all with your feedback.
Phase 1 Lead: @infra-team-lead
Status: Week 1 ✅ On Track (Conditional GO)
```

---

## Gate Decision Announcement

**Channel:** `#security-qa-phase1` (all-hands notification)
**Format:** Slack post with reactions + email follow-up

### Template

```
⚠️ PHASE 1 GATE DECISION — [GATE NAME]
═════════════════════════════════════════════════════

🚦 RESULT: [🟢 GO | 🟡 CONDITIONAL GO | 🔴 NO-GO]

GATE CRITERIA SCORECARD
──────────────────────────────────────────────────────

Criteria                        │ Target   │ Actual   │ ✅/❌
────────────────────────────────┼──────────┼──────────┼───────
Repo Coverage (SAST)            │  X/30   │  Y/30   │  [✅/❌]
Repo Coverage (Snyk)            │  A/30   │  B/30   │  [✅/❌]
Repo Coverage (Sentry)          │  C/30   │  D/30   │  [✅/❌]
Repo Coverage (Linting)         │  E/30   │  F/30   │  [✅/❌]
False Positive Rate             │  <5%    │  M%     │  [✅/❌]
Event Capture Rate              │  >95%   │  N%     │  [✅/❌]
Zero Critical Blockers          │  Yes    │  [Y/N]  │  [✅/❌]
Team Satisfaction               │  >4.0   │  S/5    │  [✅/❌]

DECISION RATIONALE
───────────────────
[Explain why GO/CONDITIONAL/NO-GO]
- [Supporting data point 1]
- [Supporting data point 2]
- [Risk/concern if applicable]

IF CONDITIONAL:
  Conditions: [List specific items needed]
  Recovery Plan: [Action if conditions not met by XXX date]

IF NO-GO:
  Critical Issues: [List blockers]
  Remediation Plan: [Actions required]
  Re-assessment Date: [Date]

NEXT STEPS
──────────
1. [Action 1]
2. [Action 2]
3. [Action 3]

CONTACTS FOR QUESTIONS
──────────────────────
  General: reply in thread
  Escalation: @engineering-lead
  Security Issues: @security-lead

---

React with ✅ if you acknowledge this decision.
```

### Example: Week 1 Gate

```
⚠️ PHASE 1 GATE DECISION — WEEK 1 (April 6)
═════════════════════════════════════════════════════════

🚦 RESULT: 🟡 CONDITIONAL GO

GATE CRITERIA SCORECARD
──────────────────────────────────────────────────────

Criteria                        │ Target   │ Actual   │ ✅/❌
────────────────────────────────┼──────────┼──────────┼───────
Repo Coverage (SAST)            │ 12/30   │ 12/30   │  ✅
Repo Coverage (Snyk)            │  3/30   │  0/30   │  ❌
Repo Coverage (Sentry)          │  3/30   │  1/30   │  ⏳
Repo Coverage (Linting)         │  3/30   │  3/30   │  ✅
False Positive Rate             │  <5%    │  3%     │  ✅
Event Capture Rate              │  >95%   │  TBD    │  ⏳
Zero Critical Blockers          │  Yes    │  No     │  ❌
Team Satisfaction               │  >4.0   │  4.2    │  ✅

DECISION RATIONALE
───────────────────
✅ SAST deployment on track (Tier 1+2 complete, 40% progress)
✅ Linting adoption on track (Tier 1 complete, frameworks ready for T2)
✅ SAST quality exceeds targets (3% FP rate vs. 5% target)
✅ Team satisfaction survey: 4.2/5.0 (exceeds 4.0 target)
❌ Snyk deployment blocked (token not received)
⏳ Sentry deployment awaiting DSN provisioning (SDK ready)

CONDITIONAL GO:
  Conditions:
    1. Snyk token must be provided by 2026-04-01 09:00 AM
    2. Sentry DSNs must be provided by 2026-04-01 12:00 PM
    3. If either not resolved, activate fallback plan (see Recovery Plan)

  Recovery Plan (if blockers not resolved by 2026-04-01):
    • Skip Snyk/Sentry in Week 2; focus on SAST Tier 3 + Linting T2-3
    • Deploy Snyk/Sentry in Phase 1.5 (2026-04-15-18)
    • Compress timeline: parallel deployment of Snyk/Sentry T1-3 (3 days vs. 10)
    • Estimated impact: 5-day timeline extension to 2026-04-18

NEXT STEPS
──────────
1. Final escalation for Snyk token (2026-03-31 EOD)
2. Final escalation for Sentry DSNs (2026-03-31 EOD)
3. If both received: Deploy immediately (2026-04-01)
4. If either delayed: Activate fallback plan & notify team
5. Begin Week 2 prep for SAST Tier 3 + other tools

CONTACTS FOR QUESTIONS
──────────────────────
  General: reply in thread
  Snyk Issues: @security-lead (security-lead@company.com)
  Sentry Issues: @platform-lead (platform-lead@company.com)
  Overall: @engineering-lead (engineering-lead@company.com)

---

React with ✅ if you acknowledge this decision.
Contingency owners: Please confirm receipt.
```

---

## "How To" Guides for Each Tool

### 1. Using Semgrep Results (SAST)

**Audience:** Developers who need to fix SAST findings
**Document:** Post in Slack / Pin in channel

```
🔍 HOW TO: Interpret & Fix Semgrep Results
═══════════════════════════════════════════════════════

FINDING SEMGREP RESULTS
──────────────────────────
1. GitHub Code Scanning tab on your repo
2. Filter by "Semgrep" (not CodeQL)
3. Sort by severity: Critical → High → Medium

UNDERSTANDING A FINDING
───────────────────────
Each finding shows:
  • Issue: [Description & CWE reference]
  • File: [Code file with line number]
  • Severity: Critical | High | Medium | Low
  • Fix: [Suggested code change]

FIXING AN ISSUE
────────────────
1. Click the finding in Code Scanning
2. Review the code context (usually simple fix)
3. Click "Suggest a fix" → GitHub auto-creates PR
4. Merge PR once approved

FALSE POSITIVE?
─────────────────
If finding is not a real issue:
1. Click "Dismiss" on the finding
2. Choose reason: "Inaccurate", "Not needed", "False positive"
3. Add comment explaining why (helps tune our rules)

NEED HELP?
───────────
  • Questions: Post in #security-qa-phase1
  • False positive patterns: Discuss with @sast-team
  • Bulk fixes: Tag @sast-team for guidance

STATS TO KNOW
──────────────
  • Average fix time: 15-30 min per finding
  • Our false positive rate: 3% (target: <5%)
  • Help & context: Always available in finding descriptions
```

### 2. Using Sentry Dashboard

**Audience:** Engineers who monitor errors
**Document:** Post in Slack / Wiki

```
📊 HOW TO: Use Sentry Dashboard
═══════════════════════════════════════════════════════

ACCESSING SENTRY
─────────────────
1. Go to: [sentry-org.sentry.io/projects/]
2. Select your repo/project from list
3. You'll see: Errors, Performance, Releases, Sessions

VIEWING YOUR ERRORS
────────────────────
1. Click "Errors" tab
2. Filter by date, release, or environment
3. Sort by: Frequency (most common) or Recency (newest)

TRIAGING AN ERROR
──────────────────
For each error:
  • Count: How many users affected
  • Error message: What went wrong
  • Stack trace: Where in code
  • Release: Which version introduced it
  • Users: Who experienced it

ASSIGNING & TRACKING
──────────────────────
1. Click error → Details view
2. Assign to owner: @engineer
3. Add comment: "Expected from PR #123; auto-resolves with 2.1.0"
4. Set status: Unresolved → Resolved once fixed
5. Or: Ignored if expected behavior

PERFORMANCE METRICS
────────────────────
1. Click "Performance" tab
2. Filter slow transactions (>1s by default)
3. Common causes: DB queries, API calls, heavy processing
4. Action: Optimize code or increase resource allocation

RELEASE TRACKING
─────────────────
1. Click "Releases" tab
2. See which version introduced which errors
3. Use to correlate errors with deployments

NEED HELP?
───────────
  • Dashboard issues: Post in #security-qa-phase1
  • Error questions: Ask @sentry-team or @backend-team
  • Performance tips: See [performance guide wiki]
```

### 3. Using Snyk Dashboard

**Audience:** Developers & platform team
**Document:** Post in Slack / Wiki (once deployed)

```
🛡️ HOW TO: Use Snyk Dependency Scanner
═══════════════════════════════════════════════════════

(This guide will be posted once Snyk is deployed and DSNs are configured.)

ACCESSING SNYK
────────────────
1. Go to: [snyk-org.snyk.io/projects/]
2. Select your repo
3. View: Open issues by severity

UNDERSTANDING AN ISSUE
───────────────────────
Each issue shows:
  • Package: Which dependency is affected
  • Severity: Critical | High | Medium | Low
  • Fix: Upgrade to version X.Y.Z
  • References: CVE links + impact description

FIXING AN ISSUE
──────────────────
1. Click issue → Details
2. Recommended fix shown (often: upgrade package)
3. Option A: Auto-PR: Snyk creates fix PR automatically
4. Option B: Manual: Update package.json/Cargo.toml yourself
5. Merge PR & re-scan to confirm fix

AUTO-REMEDIATION PRS
─────────────────────
Snyk automatically creates PRs for fixable issues:
  • You'll see: "Snyk: Upgrade [package] from X to Y"
  • Review: Check release notes for breaking changes
  • Merge: Green checkmark = safe to merge
  • Skip: If breaking changes, discuss with team

IGNORE POLICIES
─────────────────
For issues you can't fix:
  1. Click "Ignore" on the issue
  2. Reason: "Not applicable", "Patch available later", etc.
  3. Duration: Set expiry (e.g., 30 days)
  4. Added to: .snykignore file (version controlled)

NEED HELP?
───────────
  • Snyk questions: Post in #security-qa-phase1
  • Dependency strategy: Ask @platform-team
  • Security concerns: Ask @security-lead
```

### 4. Setting Up Pre-Commit Hooks (Linting)

**Audience:** All developers
**Document:** POST BEFORE linting deployment

```
🔧 HOW TO: Set Up Pre-Commit Hooks
═══════════════════════════════════════════════════════

OVERVIEW
─────────
Pre-commit hooks automatically format & lint your code before you commit.
This prevents lint failures in CI/CD.

INSTALLATION
───────────────
1. Install pre-commit framework:
   ```
   # macOS with Homebrew
   brew install pre-commit

   # Or via pip
   pip install pre-commit
   ```

2. Install repo's hooks:
   ```
   cd /path/to/repo
   pre-commit install
   ```

3. Done! Hooks are now active.

HOW IT WORKS
─────────────
When you run `git commit`:
  1. Pre-commit checks changed files
  2. Runs linters (ruff, clippy, eslint, etc.)
  3. If issues found: Auto-fixes (if possible)
  4. If fixes made: You see list of fixed files
  5. Run `git add` again for fixed files
  6. Try commit again (usually passes)

COMMON SCENARIOS
──────────────────

Scenario 1: Pre-commit auto-fixes code
  $ git commit
  ✅ [pre-commit] Fixed formatting in src/main.rs
  $ git add src/main.rs
  $ git commit
  ✅ Commit successful

Scenario 2: Pre-commit finds unfixable issues
  $ git commit
  ❌ [pre-commit] Linting error in src/lib.rs (line 42)
  Fix: Remove unused variable `x`
  $ vim src/lib.rs
  $ git add src/lib.rs
  $ git commit
  ✅ Commit successful

Scenario 3: Skip hooks (emergency only)
  $ git commit --no-verify
  ⚠️ WARNING: CI/CD will still fail if linting issues exist!
  Use only for temporary hotfixes; fix before merge.

TROUBLESHOOTING
─────────────────
Problem: "pre-commit not found"
  Solution: `pip install pre-commit`

Problem: "Hook timeout after 10s"
  Solution: Some checks slow. Run manually: `pre-commit run --all-files`

Problem: "Conflict with IDE formatter"
  Solution: IDE should use same .prettier/.ruff config. See repo docs.

UPDATING HOOKS
────────────────
When we update lint rules:
  ```
  pre-commit autoupdate
  ```

NEED HELP?
───────────
  • Setup issues: Post in #security-qa-phase1
  • Lint questions: Ask @devex-team
  • CI/CD failures: Post stack trace in #dev-support
```

---

## FAQ: Top 10 Questions

### Q1: Do I need to fix ALL SAST findings before my PR merges?

**A:** No, but critical/high severity findings block merge. Medium/low findings can be:
- Fixed immediately (preferred)
- Opened as a separate issue (marked with label `tech-debt`)
- Dismissed if false positive (with justification)

**Who to ask:** @sast-team

---

### Q2: What if Snyk or Sentry isn't deployed yet for my repo?

**A:** Both are rolling out:
- **Snyk:** Tier 1 (phenotype-infrakit, AgilePlus, heliosCLI) → Others by 2026-04-13
- **Sentry:** Tier 1 starting 2026-03-31 → Others by 2026-04-13

Check deployment matrix: `docs/reports/PHASE1_DEPLOYMENT_MATRIX.md`

**Who to ask:** @phase1-lead

---

### Q3: Why is pre-commit blocking my commit with formatting errors?

**A:** Pre-commit linter (ruff, prettier, clippy) found code style issues. Two options:
1. **Auto-fix:** Run `pre-commit run --all-files` → Re-add files → Retry commit
2. **Manual fix:** Edit file → Re-add → Retry commit

This is working as intended! It prevents style issues from reaching CI/CD.

**Who to ask:** @devex-team

---

### Q4: Can I disable pre-commit hooks?

**A:** Temporarily yes: `git commit --no-verify`

**BUT:** CI/CD will still run linting and block your PR if issues exist. Better to fix locally.

**Who to ask:** @devex-team

---

### Q5: What if there's a Sentry error I can't reproduce?

**A:** Check:
1. Stack trace (shows file + line)
2. User info (who reported it)
3. Release version (which code caused it)
4. Breadcrumbs (what happened before error)

If still unclear: Post in #security-qa-phase1 with Sentry link.

**Who to ask:** @sentry-team or @backend-team

---

### Q6: How do I know if a Snyk finding is safe to ignore?

**A:** Evaluate:
1. **CVSS Score:** <7 = lower risk; >7 = higher risk
2. **Exploitability:** How easy is it to exploit?
3. **Your usage:** Does your code path trigger the vulnerability?
4. **Fix available:** Is upgrade safe (check release notes for breaking changes)?

When in doubt: Ask @security-team or @platform-team

---

### Q7: Can I ignore SAST findings instead of fixing them?

**A:** **Critical/High:** No, must fix (blocking merge)

**Medium:** Can dismiss if:
- False positive (with explanation)
- Not applicable (with explanation)
- Intentional design choice (comment in code + justify)

**Low:** Can be dismissed or tracked as tech-debt

All dismissals are tracked and reviewed quarterly.

**Who to ask:** @sast-team

---

### Q8: My Sentry DSN seems wrong—what should I do?

**A:** Contact @sentry-team with:
1. Your repo name
2. Current DSN (if any)
3. Error you're seeing (e.g., "events not captured")

They'll re-provision DSN or help debug.

**Who to ask:** @sentry-team

---

### Q9: Should I commit the `.snykignore` file?

**A:** **Yes!** Version-control `.snykignore`:
- Tracks which issues your repo intentionally ignores
- Shared across team
- Helps document architectural decisions

Push to main branch after review.

**Who to ask:** @platform-team or @snyk-team (once deployed)

---

### Q10: What if linting/SAST/Sentry tooling breaks my build?

**A:** Report immediately to:
- **Pre-commit/linting:** @devex-team
- **SAST:** @sast-team
- **Sentry:** @sentry-team
- **Snyk:** @snyk-team

Include:
- Repo name
- Error message (full stack trace)
- Expected vs. actual behavior

**Priority:** Critical bugs = same-day fix.

---

## Escalation Matrix

| Issue Type | Primary Contact | Escalation | Timeline |
|------------|-----------------|-----------|----------|
| SAST findings (false positive) | @sast-team | @engineering-lead | 1 day |
| SAST blocking PR | @infra-team | @engineering-lead | Immediate |
| Snyk deployment blocked | @snyk-team | @security-lead | 1 day |
| Snyk issue not understood | @platform-team | @security-lead | 1 day |
| Sentry DSN missing/wrong | @sentry-team | @platform-lead | 1 day |
| Sentry events not captured | @sentry-team | @platform-lead | Same day |
| Pre-commit hook broken | @devex-team | @engineering-lead | Same day |
| Pre-commit tool conflict | @devex-team | @engineering-lead | 1 day |
| General Phase 1 question | @phase1-lead | @engineering-lead | 1 day |
| Phase 1 blocker (critical) | @phase1-lead | @ciso | Immediate |

**How to escalate:**
1. Post in #security-qa-phase1 with `@escalation` mention
2. Include: issue, who you contacted, response time needed
3. Slack will notify escalation owner

---

## Team Training Checklist

**To be completed by each team member:**

- [ ] Watched SAST training (5 min video)
- [ ] Read: "How to fix Semgrep findings" guide
- [ ] Set up pre-commit hooks locally (`pre-commit install`)
- [ ] Ran `pre-commit run --all-files` on one repo (manual test)
- [ ] Reviewed: "How to use Sentry" guide (when live)
- [ ] Reviewed: "How to use Snyk" guide (when live)
- [ ] Completed: Linting best practices (15 min doc)
- [ ] Asked questions in #security-qa-phase1 (encouraged!)

**Managers:** Track completion via Google Form (link TBD)
**Deadline:** By end of Week 1 for Tier 1 repos; by 2026-04-13 for all

---

## Acknowledgement Email

**Send to all team members upon Phase 1 kickoff:**

```
Subject: ✅ Phase 1 Kickoff: Security & QA Tools Deployment

Hi Team,

Phase 1 is underway! We're rolling out 4 security & QA tools across our
30 repositories over the next 14 days.

📋 YOUR RESPONSIBILITIES:
─────────────────────────
1. Set up pre-commit hooks (see guide: [link])
2. Fix SAST findings in your code (if applicable)
3. Watch for Snyk + Sentry deployments in your repo
4. Ask questions in #security-qa-phase1 (no dumb questions!)

📊 TIMELINE
────────────
  Week 1 (Mar 30 - Apr 6):  Tier 1+2 SAST + Linting setup
  Week 2 (Apr 7 - 10):      Tier 3 SAST + Snyk/Sentry T1-2
  Week 3 (Apr 11 - 13):     Final rollout + validation

🔗 KEY LINKS
──────────────
  Dashboard: [docs/reports/PHASE1_STATUS_DASHBOARD.md]
  Tracker: [PHASE1_TRACKER.yml]
  Standup: #security-qa-phase1 (daily, EOD)
  FAQs: [This guide]

❓ QUESTIONS?
────────────
  • Slack in #security-qa-phase1
  • Email: phase1-lead@team.internal
  • Escalation: See matrix above

✅ PLEASE ACKNOWLEDGE:
────────────────────
React with 👍 below to confirm you received this message.

Thanks for making Phase 1 a success!
—
Phase 1 Leadership Team
```

---

**Generated:** 2026-03-30
**Last Updated:** 2026-03-30
**Review & Update:** Weekly (Fridays)

---

## Appendix: Quick Links

**Documents:**
- [PHASE1_TRACKER.yml](../PHASE1_TRACKER.yml)
- [Status Dashboard](./docs/reports/PHASE1_STATUS_DASHBOARD.md)
- [Deployment Matrix](./docs/reports/PHASE1_DEPLOYMENT_MATRIX.md)
- [Success Metrics](./docs/reference/PHASE1_SUCCESS_METRICS.md)
- [Rollout Timeline](./docs/reports/PHASE1_ROLLOUT_TIMELINE.md)

**Slack Channels:**
- #security-qa-phase1 (main channel)
- #security (escalations)
- #platform (Snyk/Sentry issues)
- #dev-support (linting issues)

**External Resources:**
- Semgrep docs: https://semgrep.dev/docs/
- Snyk docs: https://docs.snyk.io/
- Sentry docs: https://docs.sentry.io/
- Pre-commit docs: https://pre-commit.com/
