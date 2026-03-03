---
audience: [agents]
---

# Governance Constraints & Safety Boundaries

AgilePlus enforces strict governance rules to ensure traceability, safety, and compliance. These rules apply to all agents and are non-negotiable.

## Core Invariants

### 1. Spec is Source of Truth

The specification document (`spec.md`) is the authoritative source for what should be built.

**Rule:** Never contradict or circumvent the specification.

```markdown
✗ FORBIDDEN: Spec says "Email validation only", agent adds phone number field
✓ CORRECT: Agent implements exactly what spec says, no more, no less
```

**Verification:** Harness compares implementation against spec requirements.

### 2. Work in Isolated Worktrees

All agent work happens in git worktrees, never directly in the main repository.

**Rule:** Never commit to `main`, `develop`, or any long-lived branch.

```bash
✗ FORBIDDEN: git commit -m "..." && git push origin main
✓ CORRECT: Work in .worktrees/001-login-WP01, commit to feat/001-login-WP01
```

**Enforcement:** VcsPort only allows branching from worktrees. Direct main commits fail at pre-commit hook.

### 3. One Work Package at a Time

Agents must complete, test, and submit one work package before starting the next.

**Rule:** Do not start WP02 until WP01 is in `FOR_REVIEW` state.

**Rationale:** Prevents merge conflicts, ensures dependencies are satisfied, maintains audit trail.

```bash
✗ FORBIDDEN: Simultaneously edit WP01 and WP02 branches
✓ CORRECT: Complete WP01 → Submit PR → Await review → Then start WP02
```

**Enforcement:** StoragePort prevents transitioning WP from `PLANNED` to `DOING` if previous WPs not in `DONE`.

### 4. No Force Pushes

All history must be preserved. Force pushes destroy traceability and break the audit chain.

**Rule:** Never use `git push --force`, `git push -f`, or equivalent.

```bash
✗ FORBIDDEN: git push --force origin feat/001-login-WP01
✓ CORRECT: Fix commits via rebase (then git push normally)
```

**Enforcement:** Pre-commit hook rejects `--force` flags. Repository has `receive.denyForcePushes = true`.

### 5. All Changes are Audited

Every change must be traceable to a specific work package and commit message.

**Rule:** No anonymous or undocumented changes.

**Format:** Every commit message must reference the WP:

```
WP01: Implement login form component
WP01: Add tests for email validation
WP02: Refactor authentication service
```

**Enforcement:** Audit log captures all commits with actor, timestamp, WP ID, and commit hash.

## File Scope Boundaries

Each work package has an explicit file scope: the list of files the agent is authorized to modify.

```json
{
  "wp_id": "WP01",
  "file_scope": [
    "src/components/LoginForm.tsx",
    "src/components/__tests__/LoginForm.test.tsx",
    "src/hooks/useLogin.ts",
    "src/styles/LoginForm.module.css"
  ]
}
```

### Strict Enforcement

**Rule:** Modifications outside `file_scope` are forbidden (except exceptions below).

```bash
✗ FORBIDDEN: Modify src/services/auth.ts (not in scope)
✗ FORBIDDEN: Edit docs/architecture.md (not in scope)
✓ CORRECT: Only edit files listed in file_scope
```

### Allowed Exceptions

**Lock files** (automatic dependency changes):
- `package-lock.json`, `pnpm-lock.yaml`, `bun.lockb`
- `Cargo.lock`
- `requirements.txt` (if using pip)

These are auto-generated; modifying them as a side effect of dependency installation is OK.

```bash
✓ OK: npm install lodash → package-lock.json updated automatically
✗ NOT OK: Manually edit package-lock.json to remove unrelated dependency
```

**Configuration files** (only if WP explicitly requires):

The WP deliverables might say "Update tsconfig.json for strict mode". Only then:

```bash
✓ OK: Edit tsconfig.json (if in deliverables)
✗ NOT OK: Edit .eslintrc.json (not in deliverables)
```

**Manifest files** (only if WP explicitly requires):

Rarely, a WP might require updating:
- `package.json` (add dependencies)
- `tsconfig.json` (compiler options)
- `.env.example` (document new env vars)

Only if listed in deliverables.

## Forbidden Git Operations

| Command | Why Forbidden | What to Do Instead |
|---------|---------------|--------------------|
| `git push --force` | Destroys history | `git rebase`, then `git push` normally |
| `git reset --hard` | Loses work | `git reflog` to recover |
| `git clean -f` | Deletes files | `git restore` to undo changes |
| `git revert` (on main) | Only humans merge | Revert only in your WP branch |
| `git merge main` (main→WP) | Pollutes history | Keep WP branch linear from base |

Example: How to fix a mistake

```bash
# WRONG: git reset --hard HEAD~1 (destroys history)
# RIGHT: git revert HEAD (creates new commit documenting the revert)
git revert HEAD
git commit -m "WP01: Revert previous commit (description of why)"
git push origin feat/001-login-WP01
```

## Governance Validation Gates

Before transitioning states, AgilePlus checks governance gates:

### SPECIFY → PLAN Gate

Validates the specification:
- [ ] Spec file exists and is valid markdown
- [ ] Contains mission, acceptance criteria, FRs
- [ ] FRs are traceable (FR-AUTH-001, FR-UI-002, etc.)

**If violated:** Cannot move to PLAN state. Agent must fix spec.

### PLAN → IMPLEMENT Gate

Validates the plan:
- [ ] Work packages defined (WP01, WP02, etc.)
- [ ] Dependencies declared
- [ ] File scope assigned
- [ ] Deliverables listed

**If violated:** Cannot move to IMPLEMENT. Fix the plan.

### IMPLEMENT → REVIEW Gate

Validates implementation:
- [ ] All WP01 deliverables exist
- [ ] Tests pass locally
- [ ] No files outside file_scope modified
- [ ] All commits reference WP ID
- [ ] PR created (if enabled)

**If violated:** Cannot move to REVIEW. Agent must fix violations.

### REVIEW → DONE Gate

Validates review completion:
- [ ] Code review approved (or waived)
- [ ] CI tests pass
- [ ] No governance violations outstanding
- [ ] Audit chain is valid

**If violated:** Cannot merge. Fix review comments or governance issues.

## Pre-commit Hooks

Every commit runs validation:

```bash
$ git commit -m "WP01: Implement component"

# Hook 1: Validate commit message
# ✓ Message starts with WP ID
# ✓ Message is descriptive

# Hook 2: Check file scope
# ✓ Modified files are in WP01 scope
# ✗ ERROR: src/services/auth.ts is outside scope
#   REJECTED: Cannot commit
```

### Bypassing Hooks

**Never skip hooks** (`--no-verify`). This is a governance violation.

```bash
✗ git commit --no-verify -m "..."  # FORBIDDEN
✓ git commit -m "..."              # Use proper process
```

If a hook is broken (legitimate issue), contact a human to fix it first.

## Secret & Credential Leaks

Agents must **never** commit secrets:

```bash
✗ FORBIDDEN: Commit API keys, tokens, passwords, SSH keys
✗ FORBIDDEN: Commit .env files with real values
✗ FORBIDDEN: Commit private keys, certificates
```

**Prevention:** Pre-commit hook scans for common secret patterns:

```bash
# These trigger rejection:
- "api_key": "sk-..."
- "password": "hunter2"
- BEGIN RSA PRIVATE KEY
- AWS_SECRET_ACCESS_KEY
```

**What to do instead:**

```bash
# ✓ Commit .env.example with placeholder values
GITHUB_TOKEN=xxx_YOUR_TOKEN_HERE_xxx
```

## Review Checklist

Before marking a WP as complete, agents must verify:

```markdown
## Pre-Submission Checklist

- [ ] **Spec compliance:** Implementation matches spec exactly
- [ ] **Deliverables:** All files in deliverables list exist
- [ ] **Tests:** `npm run test` passes with >80% coverage
- [ ] **Linting:** `npm run lint` passes (no errors)
- [ ] **File scope:** No modifications outside authorized files
- [ ] **Commits:** All commits reference WP ID (WP01:, WP01 fix, etc.)
- [ ] **Clean state:** `git status` shows "nothing to commit"
- [ ] **Branch:** Working in correct worktree/branch
- [ ] **History:** Commits are linear (no merge commits in middle)
- [ ] **No secrets:** No .env, keys, tokens, credentials committed
```

If any box is unchecked, fix it before submitting.

## Escalation Protocol

If governance rules conflict with task requirements:

### Example 1: Spec Says X, Tests Require Y

**Scenario:** Spec says "Email validation only", but tests require phone number field.

**Resolution:**
1. **Stop work** on implementation
2. **Document conflict** in WP comment with link to spec section
3. **Create blocker task** describing the conflict
4. **Wait for human decision**
5. **Resume** after conflict is resolved

**Output:**
```json
{
  "success": false,
  "status": "blocked",
  "error": "Spec vs test conflict",
  "conflict_details": {
    "spec": "Section 2.1: 'Email validation only'",
    "test": "TestEmailPhoneValidation.ts requires both fields",
    "resolution_needed": "Clarify spec or update tests"
  },
  "remediation": "Human must reconcile spec and tests"
}
```

### Example 2: File Scope Too Restrictive

**Scenario:** WP says "Build LoginForm", but needs to refactor auth.ts (outside scope).

**Resolution:**
1. **Stop** before modifying auth.ts
2. **Document** why refactoring is needed (code smell, test failure, etc.)
3. **Create new WP** for refactoring with proper scope
4. **Continue** with original WP, using old code
5. **Wait** for approval to do refactoring in new WP

**Output:**
```json
{
  "success": false,
  "status": "blocked",
  "error": "File scope violation needed for task",
  "violation": {
    "required_file": "src/services/auth.ts",
    "reason": "Refactoring needed to add session tokens",
    "solution": "Create WP03 for auth.ts refactoring"
  },
  "remediation": "Create new WP with expanded scope"
}
```

### Example 3: Dependency Not Approved

**Scenario:** Need `zod` for email validation, but not in approved list.

**Resolution:**
1. **Stop** before adding dependency
2. **Document** why it's needed (RFC 5322 email validation)
3. **Request exception** from governance team
4. **Continue** with alternative if available
5. **Wait** for approval before adding dependency

**Output:**
```json
{
  "success": false,
  "status": "blocked",
  "error": "Dependency not approved: zod",
  "reason": "Email RFC 5322 validation requires proper parser",
  "alternatives_tried": [
    "regex (insufficient)",
    "native browser validation (insufficient)"
  ],
  "remediation": "Request governance exception for zod"
}
```

## Non-Repudiation & Audit

Every agent action is logged immutably:

```jsonl
{"timestamp":"2025-01-16T14:22:00Z","actor":"claude-code","action":"commit","wp_id":"WP01","commit_sha":"abc123","message":"WP01: Implement form","files":["src/components/LoginForm.tsx"],"verified":true,"prev_hash":"...","hash":"..."}
{"timestamp":"2025-01-16T14:23:15Z","actor":"claude-code","action":"transition","wp_id":"WP01","from":"DOING","to":"FOR_REVIEW","verified":true,"prev_hash":"...","hash":"..."}
```

This audit trail is:
- **Append-only:** Cannot be modified or deleted
- **Cryptographically chained:** Hash of each entry includes hash of previous entry
- **Tamper-evident:** Any modification breaks the chain
- **Non-repudiable:** Agent cannot deny what it did

## Summary

**Remember:**
1. **Follow spec exactly** — it's the source of truth
2. **Stay in worktrees** — never touch main directly
3. **Respect file scope** — only edit authorized files
4. **Preserve history** — no force pushes or hard resets
5. **Document everything** — commit messages, blockers, decisions
6. **Escalate conflicts** — don't work around governance rules
7. **No secrets** — ever commit credentials, keys, or tokens

**When in doubt:** Stop, document, and ask a human.

## Transition Rules: What Agents Can Trigger

Agents interact with the state machine through the `agileplus governance` subcommands. Not all transitions are agent-triggerable:

| Transition | Agent Can Trigger? | Reason |
|------------|-------------------|--------|
| `created → specified` | No | Requires human spec authorship |
| `specified → researched` | Yes | Research is agent work |
| `researched → planned` | Yes | Plan generation is agent work |
| `planned → implementing` | Yes | Assigning a WP to start |
| WP `planned → doing` | Yes | Agent starts WP work |
| WP `doing → review` | Yes | Agent submits completed work |
| WP `review → done` | No | Requires human or CI approval |
| WP `review → doing` | No | Human requests changes |
| `implementing → validated` | Yes, if all CI passes | Automated governance check |
| `validated → shipped` | No | Requires human approval |
| `shipped → retrospected` | No | Human-led retrospective |
| Any → `cancelled` | No | Human decision only |

Agents must check `agileplus governance check <transition>` before attempting any transition:

```bash
agileplus governance check implementing->validated
# ✓ All WPs in Done state
# ✓ TestResult evidence present for all FRs
# ✓ LintResult evidence present
# ✓ SecurityScan evidence present
# Ready to transition
```

If any check fails, the governance command returns non-zero and prints which requirements are unmet.

## Hash-Chain Integrity: Agent Obligations

Agents must never attempt to modify or tamper with the audit chain. Specifically:

```bash
# FORBIDDEN: Cannot delete the audit database
✗ rm .agileplus/agileplus.db

# FORBIDDEN: Cannot modify audit entries
✗ sqlite3 .agileplus/agileplus.db "UPDATE audit_entries SET actor='human:alice' WHERE id=5"

# FORBIDDEN: Cannot reset git history to remove evidence
✗ git push --force origin main

# CORRECT: Read-only access to audit trail
✓ agileplus events query --feature user-auth
```

Verification runs automatically before any `validated → shipped` transition:

```bash
agileplus events audit-verify --feature user-authentication
# If PASS: "Audit chain intact (12 entries verified)"
# If FAIL: "CRITICAL: Hash mismatch at entry 5 — chain tampered!"
#          Transition blocked. Alert sent to governance team.
```

A tampered chain blocks the entire feature from shipping and triggers an alert.

## Dependency Constraint Enforcement

When an agent is working on WP02 that depends on WP01:

```
Rule: An agent may not start WP02 until WP01 is in Done state.

Enforcement flow:
  1. Agent calls: agileplus wp transition WP02 --to doing
  2. Engine checks: is WP01 in Done state?
  3. If WP01 is Planned/Doing/Review → error:
       "WP02 is blocked by WP01 (state: Doing). Cannot start."
  4. If WP01 is Done → success:
       "WP02 transitioned to Doing. Worktree created."
```

Agents must not attempt to circumvent this by:
- Checking out the branch manually without going through `implement`
- Editing files that belong to WP01's scope while WP01 is not Done
- Assuming WP01's changes are present before they're merged to main

The correct pattern:

```bash
# Check WP01 is done
agileplus wp status user-auth WP01
# If: state=done → proceed

# Dispatch WP02 (will succeed because WP01 is done)
agileplus implement user-auth --wp WP02
```

## Evidence Obligations

Agents must produce evidence for every functional requirement their WP addresses. Inadequate evidence causes the governance check to fail:

```bash
# Good: Agent writes test results as evidence
agileplus artifact write \
  --feature user-authentication \
  --path WP01/evidence/test-results.json \
  --content '{"passed": 87, "failed": 0, "coverage": "94%", "fr_ids": ["FR-001", "FR-002"]}'

# Good: Agent registers evidence
agileplus governance enforce implementing->validated \
  --evidence-type TestResult \
  --fr-id FR-001 \
  --artifact-path WP01/evidence/test-results.json
```

Evidence requirements per governance contract:

```json
{
  "transition": "implementing->validated",
  "required_evidence": [
    {"fr_id": "FR-001", "evidence_type": "TestResult"},
    {"fr_id": "FR-002", "evidence_type": "TestResult"},
    {"fr_id": "FR-003", "evidence_type": "SecurityScan"},
    {"evidence_type": "LintResult"}
  ]
}
```

If the agent does not produce `SecurityScan` evidence, the transition check fails:

```
✗ GovernanceViolation: SecurityScan evidence missing for FR-003
  Required by: GovernanceContract v1, rule "implementing->validated"
  Remediation: Run cargo audit and record results as SecurityScan evidence
```

## Summary Reference Card

```
AGENTS MUST:
  ✓ Work in assigned worktree only
  ✓ Commit with WP ID prefix: "WP01: ..."
  ✓ Stay within file_scope
  ✓ Produce evidence for FRs
  ✓ Exit with 0 on success, non-zero on failure
  ✓ Output JSON result to stdout on completion
  ✓ Stop and report when blocked by governance conflict

AGENTS MUST NOT:
  ✗ Commit to main, develop, or any long-lived branch
  ✗ Use git push --force or git reset --hard
  ✗ Touch files outside file_scope
  ✗ Bypass hooks with --no-verify
  ✗ Commit secrets, credentials, or keys
  ✗ Modify audit trail or governance database
  ✗ Trigger human-only transitions (cancel, ship, retrospect)
  ✗ Skip pre-commit governance hooks
  ✗ Start a WP before its dependencies are Done
```

## Next Steps

- [Prompt Format](prompt-format.md) — What agents receive in their prompt
- [Harness Integration](harness-integration.md) — Implementing agent adapters
- [Governance & Audit](../concepts/governance.md) — State machine and audit chain
- [Agent Dispatch](../concepts/agent-dispatch.md) — How dispatch works end-to-end
- [Agent Integration Example](../examples/agent-integration.md) — End-to-end walkthrough
