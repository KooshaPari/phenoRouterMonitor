---
audience: [agents, developers]
---

# Agent Integration Example

End-to-end example of dispatching an AI agent to implement a work package.

## Scenario

You have a feature `003-auth-system` with work package `WP02: Implement login endpoint`.

## Step 1: Check Status

```bash
$ agileplus status 003

Feature: 003-auth-system (Implement)

  WP01  Models & migrations     ████████████ done
  WP02  Login endpoint          ░░░░░░░░░░░░ planned
  WP03  Session middleware      ░░░░░░░░░░░░ planned (blocked by WP02)
```

## Step 2: Dispatch Agent

```bash
$ agileplus implement WP02 --agent claude-code

Creating worktree: .worktrees/003-auth-system-WP02
Branch: feat/003-auth-system-WP02
Dispatching to Claude Code...

Session started: sess_abc123
```

## Step 3: Agent Works

The agent receives a structured prompt with:
- Feature spec context
- Plan for WP02
- Deliverable file list
- Governance constraints

The agent creates files, writes tests, and commits:

```
feat(WP02): implement login endpoint

- POST /api/auth/login with email/password
- JWT token generation and validation
- Rate limiting on login attempts
- Unit tests for auth service
```

## Step 4: Move to Review

```bash
$ agileplus move WP02 --to for_review --note "Login endpoint complete with tests"

WP02 moved to for_review
  Commits: 3 (ahead of main)
  Files changed: 8
  Tests: 12 passing
```

## Step 5: Review

```bash
$ agileplus review WP02

Reviewing WP02: Login endpoint
  ✓ All deliverables present
  ✓ Tests pass (12/12)
  ✓ No files outside WP scope modified
  ✓ Commit messages reference WP02

Result: APPROVED
WP02 moved to done
WP03 is now unblocked
```

## Step 6: Continue Pipeline

```bash
$ agileplus implement WP03 --agent claude-code
# WP03 is now unblocked and can proceed
```
