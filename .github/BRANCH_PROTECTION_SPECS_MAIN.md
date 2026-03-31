# specs/main Branch Protection Policy

**Version:** 1.0
**Effective Date:** 2026-03-31
**Status:** ACTIVE

---

## Overview

The `specs/main` branch is the **authoritative Single Source of Truth (SSOT)** for all Phenotype ecosystem specifications:
- Functional Requirements (FR-XXX-NNN)
- Architecture Decision Records (ADR-NNN)
- Implementation Plans (PLAN-NNN)
- User Journeys (UJ-NNN)

All specs must originate from feature branches and pass validation before merging to specs/main.

---

## Protection Rules

### 1. Require Pull Request Reviews

**Configuration:**
- Require 1 approval before merge
- Approve from users with write access to phenotype-infrakit, AgilePlus, or platforms/thegent
- Dismiss stale reviews on new commits: **Enabled**

**Rationale:** Ensures at least one human or automated reviewer validates spec changes before they become canonical.

### 2. Status Checks Required

**Required Checks:**
- `ci-ssot-validation` — Validate FR/ADR/PLAN/UJ structure
- `ci-fr-test-coverage` — Ensure FR↔Test traceability
- Any additional checks defined per-repo

**Rationale:** Automated validation prevents invalid specs from being merged.

### 3. Branch Requirements

**Must Pass Before Merge:**
- Require branches to be up-to-date with base branch (specs/main)
- Require status checks to pass before merge
- Include administrators in restrictions

**Rationale:** Prevents stale branches from merging; ensures all checks re-run against latest specs/main.

### 4. Merge Configuration

**Merge Method:** Squash and Rebase
- Converts all commits in PR to a single commit
- Rebases onto specs/main
- Result: Linear history (no merge commits)

**Auto-delete Head Branches:** Enabled
- Automatically deletes the feature branch after merge
- Prevents accumulation of stale branches

**Rationale:** Linear history makes it easy to bisect, understand commit order, and audit changes.

### 5. Force Push Restrictions

**Allowed For:**
- SSOT Service (GitHub App) only — emergency conflict resolution
- No other users or CI systems

**Rationale:** Prevents accidental history rewrites while allowing controlled conflict recovery.

---

## Branching Strategy

### Feature Branches

**Pattern:** `specs/agent-<agent-name>-<task-id>`

**Examples:**
- `specs/agent-phenosdk-decomposer-fr001`
- `specs/agent-agileplus-merger-wp13`
- `specs/agent-thegent-specs-fr-004`

**Rules:**
1. Create from `specs/main`
2. Make spec changes (add/edit FUNCTIONAL_REQUIREMENTS.md, ADR.md, etc.)
3. Commit with `Spec-Traces: FR-XXX-NNN` in message
4. Push to origin
5. Create PR to specs/main
6. Wait for CI validation
7. Merge automatically if clean, or create issue if conflicts

### Temporary Agent Branches

**Pattern:** `specs/agent-test-<purpose>`

**Lifetime:** Short-lived test branches, deleted after merge

**Example:** `specs/agent-test-schema-validation`

---

## Commit Message Format

**Required Format:**

```
<type>(<scope>): <subject>

<body>

Spec-Traces: <FR-XXX, ADR-YYY, ...>
Related-Issues: #123, #456
Co-Authored-By: <agent-name> <agent@phenotype.local>
```

**Components:**

| Component | Description | Example |
|-----------|-------------|---------|
| type | Change type: `specs`, `chore`, `fix` | `specs` |
| scope | Affected module/repo | `infra`, `agile`, `thegent` |
| subject | Brief change description (imperative) | `Add FR-INFRA-001 event sourcing` |
| body | Detailed explanation | Multi-line context for the change |
| Spec-Traces | **REQUIRED** — FRs/ADRs traced | `FR-INFRA-001, ADR-001` |
| Related-Issues | Optional — GitHub issue links | `#123, #456` |
| Co-Authored-By | Agent attribution | `phenosdk-decomposer <agent@...>` |

**Validation:**
- CI validates that `Spec-Traces:` field exists
- CI validates that referenced FRs/ADRs exist
- Commits without Spec-Traces are rejected

---

## Review Process

### Automatic Review (No Conflicts)

1. **Push to specs/agent-*** branch
2. **CI runs validation:**
   - `ci-ssot-validation` (structure check)
   - `ci-fr-test-coverage` (traceability check)
3. **If passing:** Auto-merge to specs/main within 5 minutes
4. **If failing:** Validation error comment on PR

### Manual Review (Conflicts)

1. **Push to specs/agent-*** branch
2. **CI detects merge conflict**
3. **Auto-create issue:** "Merge conflict: specs/agent-XXX"
4. **Manual resolution:**
   - Review conflict details
   - Resolve conflicts manually
   - Push updated branch
   - Merge manually or wait for auto-merge

### Scheduled Review (Daily)

**Time:** 9am UTC

**Actions:**
- Review all pending PRs to specs/main
- Approve validated specs
- Request changes for invalid specs
- Document decisions in review log

---

## Access Control

### Merge Permissions

**Who can merge to specs/main:**
- Automated systems (SSOT merge orchestrator)
- SSOT admin role (manual override)
- Code owners (phenotype-infrakit, AgilePlus, thegent admins)

**Who can push to specs/agent-*** branches:**
- Any user with write access to the repo
- CI systems
- Agents (with GitHub App authorization)

### Force Push Permissions

**Who can force-push to specs/main:**
- SSOT Service (GitHub App) — emergency only
- No other users or processes

**Recovery Procedure:**
1. SSOT service detects unresolvable merge conflict
2. Force-pushes to revert conflicting commit
3. Creates issue: "Emergency revert: [commit hash]"
4. Notifies team

---

## Validation Workflow

### Pre-Merge Validation (CI/CD)

**Steps:**

1. **Spec Structure Validation**
   - Check FUNCTIONAL_REQUIREMENTS.md format
   - Verify all FR-XXX-NNN IDs are unique
   - Validate ADR headers and format
   - Check PLAN.md phase structure
   - Validate USER_JOURNEYS.md actor definitions

2. **Commit Message Validation**
   - Verify `Spec-Traces:` field exists
   - Verify referenced FRs/ADRs exist in FUNCTIONAL_REQUIREMENTS.md
   - Check that all commits are traced (100%)

3. **FR↔Test Traceability**
   - Extract all FRs from FUNCTIONAL_REQUIREMENTS.md
   - Scan test files for `Traces to: FR-XXX-NNN` comments
   - Verify every FR has ≥1 test
   - Verify every test traces to ≥1 FR
   - Fail if coverage <100%

4. **Merge Conflict Detection**
   - Attempt dry-run merge against specs/main
   - Detect conflicts early
   - Report conflict location and details

5. **Dependency Validation**
   - Check for circular FR dependencies
   - Verify plan phases have correct ordering
   - Validate all cross-repo references

**Pass/Fail Logic:**
- PASS: All checks pass → Auto-merge enabled
- FAIL: Any check fails → Block merge, report error

---

## Maintenance & Monitoring

### Daily Health Check

**Run:** 9am UTC

**Checks:**
- specs/main clean (no unmerged commits >24h)
- All agent branches either merged or have issues
- FR↔Test coverage 100%
- Spec versions up-to-date

**Action:**
- Generate health score (see SSOT_HEALTH_DASHBOARD.md)
- Alert if score drops below 50

### Weekly Cleanup

**Run:** Mondays 10am UTC

**Actions:**
- Delete merged agent branches (auto-delete enabled, manual cleanup)
- Archive resolved conflict issues
- Update SPECS_REGISTRY.md version numbers
- Generate traceability matrix report

### Monthly Audit

**Run:** 1st of each month

**Scope:**
- Verify all specs in FUNCTIONAL_REQUIREMENTS.md have tests
- Check for orphan FRs (specs without implementation)
- Review access logs for spec/main changes
- Audit merge conflict trends
- Validate specification standards compliance

---

## Rollback & Recovery

### Scenario: Accidental Merge of Invalid Spec

**Steps:**
1. Identify problematic commit in specs/main
2. Contact SSOT admin
3. SSOT admin creates revert branch: `specs/recovery-<date>`
4. SSOT admin reverts problematic commit
5. SSOT admin merges recovery branch to specs/main
6. Root cause analysis performed
7. Preventive validation rule added to CI

### Scenario: Merge Conflict Blocking Auto-Merge

**Steps:**
1. CI detects conflict, creates issue
2. Manual review: `specs/` maintainer reviews conflict
3. Resolve conflict in agent branch
4. Push resolved branch
5. CI re-runs validation
6. Auto-merge proceeds

### Scenario: CI Failure (e.g., Validation Timeout)

**Steps:**
1. Retry CI job (manual re-run or wait for next push)
2. If persists: Contact CI administrator
3. Update validation rules or increase timeouts
4. Resume merge after fix

---

## Policy Updates

**Approval Required:** SSOT Working Group

**Change Process:**
1. Propose change in GitHub discussion
2. Document impact and rationale
3. Circulate for 24h review period
4. Update this policy document
5. Communicate change to all agents
6. Validate new rules with test branch

**Last Updated:** 2026-03-31
**Next Review:** 2026-04-30

---

## Related Documents

- `.commit-template` — Commit message format template
- `SSOT_PHASE1_IMPLEMENTATION_PLAN.md` — Phase 1 execution roadmap
- `SSOT_PHASE1_AGENT_WORKFLOW.md` — Agent branching and commit procedures
- `SSOT_HEALTH_DASHBOARD.md` — Real-time health metrics
- `FUNCTIONAL_REQUIREMENTS.md` — Master spec file
- `ADR.md` — Architecture decision records
- `PLAN.md` — Implementation plans
- `USER_JOURNEYS.md` — User workflow definitions

---

**Policy Owner:** Platform Architect
**Enforcement:** GitHub branch protection rules + CI/CD validation
**Questions/Issues:** Post in GitHub discussions or contact SSOT admin
