# Review Comment Templates

These templates are used by the CI review automation system to post consistent, actionable review comments.

---

## Template 1: Critical Security Issue

**Severity:** 🔴 CRITICAL
**Blocking:** Yes
**Approval Required:** Security Lead

```markdown
🔴 **CRITICAL — {ISSUE_TITLE}**

**Location:** `{FILE_PATH}:{LINE_NUMBER}`

**Issue Description:**
{DETAILED_EXPLANATION_OF_ISSUE}

**Risk Assessment:**
- Severity: Critical
- Impact: {POTENTIAL_IMPACT}
- Likelihood: {LIKELIHOOD}

**How to Fix:**
1. {STEP_1}
2. {STEP_2}
3. {STEP_3}

**Reference Material:**
- {OWASP_LINK_OR_BEST_PRACTICE}
- {INTERNAL_POLICY_LINK}
- {CODE_EXAMPLE_OR_PR_LINK}

**Approval SLA:** 2 hours
**Review Required:** Security Lead

---
Bot: Copilot Code Review v1.0
```

---

## Template 2: High-Priority Logic Error

**Severity:** 🟠 HIGH
**Blocking:** Yes
**Approval Required:** Tech Lead

```markdown
🟠 **HIGH — {ISSUE_TITLE}**

**Location:** `{FILE_PATH}:{LINE_NUMBER}`

**Problem:**
{CLEAR_DESCRIPTION_OF_LOGIC_ERROR}

**Current Code:**
\`\`\`{LANGUAGE}
{EXISTING_CODE_SNIPPET}
\`\`\`

**Suggested Fix:**
\`\`\`{LANGUAGE}
{FIXED_CODE_SNIPPET}
\`\`\`

**Why This Matters:**
{EXPLANATION_OF_IMPACT_AND_CORRECTNESS}

**Test Cases to Add:**
- [ ] Test case for {EDGE_CASE_1}
- [ ] Test case for {EDGE_CASE_2}
- [ ] Test case for {EDGE_CASE_3}

**Approval SLA:** 4 hours
**Review Required:** Tech Lead

---
Bot: CodeRabbit Code Review v1.0
```

---

## Template 3: Medium Priority — Code Quality Issue

**Severity:** 🟡 MEDIUM
**Blocking:** No
**Approval Required:** Acknowledgment

```markdown
🟡 **MEDIUM — {ISSUE_TITLE}**

**Location:** `{FILE_PATH}:{LINE_NUMBER}`

**Observation:**
{DESCRIPTION_OF_QUALITY_ISSUE}

**Impact:**
- Maintainability: {RATING}
- Readability: {RATING}
- Performance: {RATING}

**Suggestion:**
{RECOMMENDED_IMPROVEMENT}

**Example:**
\`\`\`{LANGUAGE}
{BETTER_APPROACH_CODE}
\`\`\`

**Resources:**
- {STYLE_GUIDE_LINK}
- {BEST_PRACTICE_LINK}

**Note:** This is non-blocking but should be addressed in the current PR if possible.

---
Bot: CodeRabbit Code Review v1.0
```

---

## Template 4: Low Priority — Nitpick/Suggestion

**Severity:** 🟢 LOW
**Blocking:** No
**Approval Required:** None

```markdown
🟢 **LOW — {ISSUE_TITLE}**

**Location:** `{FILE_PATH}:{LINE_NUMBER}`

**Comment:**
{HELPFUL_SUGGESTION_OR_NITPICK}

**Suggested Change:**
\`\`\`{LANGUAGE}
{IMPROVED_CODE_OPTION}
\`\`\`

**Why:** {BRIEF_JUSTIFICATION}

**Optional:** Feel free to address in a follow-up PR if preferred.

---
Bot: Bugbot Quality Check v1.0
```

---

## Template 5: Test Coverage Issue

**Severity:** 🟠 HIGH
**Blocking:** Yes for <80% coverage
**Approval Required:** Tech Lead

```markdown
🟠 **TEST COVERAGE BELOW THRESHOLD**

**Current Coverage:** {CURRENT_COVERAGE}%
**Threshold:** {THRESHOLD}%
**Gap:** {GAP_PERCENTAGE}%

**Untested Code Paths:**
- {FUNCTION_1}: {COVERAGE}% covered
- {FUNCTION_2}: {COVERAGE}% covered
- {FUNCTION_3}: {COVERAGE}% covered

**Required Tests:**
Add test cases covering:
1. {NORMAL_CASE}
2. {EDGE_CASE_1}
3. {EDGE_CASE_2}
4. {ERROR_CASE}

**Example Test:**
\`\`\`{LANGUAGE}
{TEST_TEMPLATE}
\`\`\`

**Metrics Command:**
\`\`\`bash
{COMMAND_TO_CHECK_COVERAGE}
\`\`\`

**Approval SLA:** 4 hours
**Review Required:** Tech Lead

---
Bot: Coverage Analyzer v1.0
```

---

## Template 6: Secrets Detected (Blocking)

**Severity:** 🔴 CRITICAL
**Blocking:** YES (Cannot merge)
**Approval Required:** Security Lead (to override)

```markdown
🔴 **CRITICAL — SECRETS DETECTED**

⚠️ **THIS PR CANNOT MERGE UNTIL SECRETS ARE REMOVED** ⚠️

**Detection Results:**
- Scanner: TruffleHog
- Verified Secrets: {COUNT}
- Confidence: {CONFIDENCE}%

**Detected Secrets:**
- Type: {SECRET_TYPE} (e.g., AWS Key, GitHub Token)
- Location: `{FILE}:{LINE_NUMBER}`
- Pattern: {PATTERN_DETECTED}

**Immediate Actions Required:**
1. Remove the secret from the code immediately
2. Regenerate the secret in the source system
3. Verify no commits contain the exposed secret
4. Force-push (or create new clean branch) with secret removed

**Tools:**
- Remove from history: `git filter-branch` or `git-filter-repo`
- Verify removal: `trufflehog git file://.`
- Report to security: Contact @security-team

**Reference:**
- [GitHub: Removing Sensitive Data](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository)
- [OWASP: Secrets Management](https://owasp.org/www-project-application-security-verification-standard/)

**Approval SLA:** Immediate — escalate to security lead
**Review Required:** Security Lead (to clear override)

---
Bot: TruffleHog Secret Scanner v3.x
```

---

## Template 7: Merge Readiness Report (Informational)

**Severity:** 🟢 INFORMATIONAL
**Blocking:** No
**Approval Required:** N/A

```markdown
🟢 **MERGE READINESS REPORT**

**Overall Status:** ✅ READY TO MERGE

**Readiness Score:** {SCORE}%

**Gate Status:**
- ✅ Draft Mode: Not in draft
- ✅ Status Checks: All passing (8/8)
- ✅ Approvals: 2 approvals received
- ✅ Merge Conflicts: None detected
- ✅ Bot Comments: 0 CRITICAL issues

**Timeline:**
- Opened: {DATE} ({TIME_OPEN})
- Last Updated: {DATE} ({TIME_SINCE_UPDATE})
- Ready Since: {DATE} ({TIME_READY})

**Reviewers:**
- @reviewer-1 (Approved {TIME_AGO})
- @reviewer-2 (Approved {TIME_AGO})

**Next Steps:**
- [ ] Squash and merge when ready
- [ ] Or add `ready` label for auto-merge

**Auto-Merge Option:**
If you add the `ready` label, this PR will be automatically merged when all conditions are met.

---
Bot: CI Review Coordinator v1.0
Updated: {TIMESTAMP}
```

---

## Template 8: Merge Blocked — Pending Review

**Severity:** 🔴 BLOCKING
**Blocking:** Yes
**Approval Required:** Varies by issue

```markdown
🔴 **MERGE BLOCKED — PENDING REVIEW**

**Readiness Score:** {SCORE}%
**Status:** ⏳ NOT READY

**Blocking Issues:**
1. ❌ {BLOCKING_ISSUE_1}
   - Action Required: {SPECIFIC_ACTION}
   - Responsible: {OWNER}
   - SLA: {HOURS} hours

2. ❌ {BLOCKING_ISSUE_2}
   - Action Required: {SPECIFIC_ACTION}
   - Responsible: {OWNER}
   - SLA: {HOURS} hours

**Progress:**
- Draft Mode: ✅ Not in draft
- Status Checks: 🟡 {PASSING}/{TOTAL} passing
  - Failed: {FAILED_CHECK_NAME}
- Approvals: 🟡 {COUNT}/{REQUIRED} needed
- Conflicts: ❌ Merge conflicts present
- Bot Issues: 🟡 {COUNT} CRITICAL comments

**What's Needed:**
1. {ACTION_1}
2. {ACTION_2}
3. {ACTION_3}

**Timeline:**
- Escalation Time: {TIME_UNTIL_ESCALATION}
- SLA Expires: {EXPIRATION_TIME}

**Help:**
Unclear what's needed? Review the [CI Review Automation Strategy](docs/governance/CI_REVIEW_AUTOMATION_STRATEGY.md) or ask in #ci-automation.

---
Bot: CI Review Coordinator v1.0
Updated: {TIMESTAMP}
```

---

## Template 9: Override Acknowledgment

**Severity:** 🟡 INFORMATIONAL
**Blocking:** No
**Approval Required:** Tech Lead

```markdown
✅ **OVERRIDE ACKNOWLEDGED**

**Override Reason:** {REASON_SELECTED}
- [x] Bot false positive
- [ ] Temporary exception (deadline)
- [ ] External dependency

**Justification:**
{DETAILED_EXPLANATION}

**Approved By:** @{TECH_LEAD}
**Approval Time:** {TIMESTAMP}

**Tracking:**
- JIRA Ticket: {JIRA_LINK} (if applicable)
- Follow-up Sprint: {SPRINT} (if applicable)

**Label Applied:** `override-acknowledged`

This PR may now proceed to merge pending all other checks.

---
Bot: CI Review Coordinator v1.0
```

---

## Template 10: Auto-Merge Executed

**Severity:** 🟢 INFORMATIONAL
**Blocking:** No
**Approval Required:** N/A

```markdown
✅ **AUTO-MERGED SUCCESSFULLY**

**Merge Details:**
- PR: #{PR_NUMBER}
- Method: Squash and merge
- Commit: {COMMIT_SHA}
- Branch: `{BRANCH_NAME}` → `main`

**Timeline:**
- Opened: {DATE}
- Ready: {DATE}
- Merged: {DATE}
- Time-to-Merge: {DURATION}

**Stats:**
- Commits: {COUNT}
- Files Changed: {COUNT}
- Additions: +{COUNT}
- Deletions: -{COUNT}

**Next Steps:**
- Monitor deployment (if applicable)
- Review [release notes](releases/latest)
- Check for any post-merge failures

---
Bot: CI Review Coordinator v1.0
Merge Time: {TIMESTAMP}
```

---

## Usage Instructions

### For Bot Developers

When implementing new review checks, use these templates as a baseline:

1. **Choose severity level** (🔴 CRITICAL, 🟠 HIGH, 🟡 MEDIUM, 🟢 LOW)
2. **Include location** (file, line number, code snippet)
3. **Explain the issue** (why it matters, what the risk is)
4. **Provide actionable fix** (clear steps, code example)
5. **Set expectations** (SLA, approval required, blocking status)

### For PR Authors

When you see a bot comment:

1. **Read the severity** — 🔴 CRITICAL must be fixed before merge
2. **Understand the issue** — Click links to learn more
3. **Follow the fix** — Use suggested code if provided
4. **Verify the fix** — Run tests, check CI passes
5. **Confirm resolution** — Reply to comment when fixed

### For Reviewers

When approving PRs:

1. **Check bot comments** — Are they all addressed?
2. **Review suggested fixes** — Are they correct?
3. **Override if needed** — Comment with reason + tech lead approval
4. **Approve when clear** — This signals readiness to merge

---

**Template Version:** 1.0
**Last Updated:** 2026-03-30
**Maintenance:** Review quarterly or after major CI workflow changes
