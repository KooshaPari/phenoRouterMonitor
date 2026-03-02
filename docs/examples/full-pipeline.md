---
audience: [developers, agents, pms]
---

# Full Pipeline Example

Walk through the complete AgilePlus pipeline — from specification to shipped feature. This example shows all CLI commands, expected outputs, and decision points.

## Scenario

We're adding OAuth2 authentication to a web application. The team wants Google and GitHub login, session management, and UI components. The feature will take 3-4 weeks with a small team.

## Phase 1: Specification & Planning

### 1. Create the Spec

First, create a feature specification that captures requirements without implementation details:

```bash
agileplus specify \
  --title "OAuth2 Authentication" \
  --description "Add Google and GitHub OAuth2 login flows with persistent session management"
```

This creates `kitty-specs/001-oauth2-auth/spec.md`. The spec includes:
- **Functional Requirements** — what the feature does
  - Users can sign up via Google OAuth
  - Users can sign up via GitHub OAuth
  - Sessions persist across browser restarts
  - Logout clears all sessions
- **User Scenarios** — real-world workflows
  - User visits login page, clicks "Sign in with Google", is redirected to Google
  - Returns to app authenticated with user profile
  - Navigates away and returns; session is still valid
- **Acceptance Criteria** — measurable success signals
  - Both OAuth flows complete in <2 seconds
  - Session valid for 30 days
  - Logout is instant (no database lag)

### 2. Research the Codebase

Scan existing code for patterns and dependencies:

```bash
agileplus research 001
```

```
Analyzing codebase...

Existing auth infrastructure:
  - Session middleware: src/middleware/session.rs
  - JWT utilities: src/auth/jwt.rs
  - User model: src/models/user.rs

Framework: Actix-web 4.4 with Tokio runtime

Recommended dependencies:
  - actix-web-httpauth (0.8.1)  — OAuth2 support
  - openidconnect (0.11)         — OpenID Connect library
  - serde_json (1.0)              — JSON handling

Existing patterns detected:
  ✓ Middleware tower pattern
  ✓ Error handling with Result/Error enum
  ✓ Config via environment variables
```

### 3. Plan Work Packages

Decompose the spec into ordered work packages:

```bash
agileplus plan 001
```

```
Planning feature 001...

Generated 4 work packages:

WP01: Provider Configuration & Models
  - Register OAuth apps with Google and GitHub
  - Add OAuth2 config to app settings
  - Create OAuthProvider enum and OAuthToken model
  - Deliverables: providers.rs, oauth_config.rs (with tests)
  - Est. effort: 2 days

WP02: Google Login Flow
  - Implement GET /auth/google/redirect endpoint
  - Implement GET /auth/google/callback endpoint
  - Exchange auth code for token
  - Fetch user profile from Google API
  - Create user record if new
  - Set session cookie
  - Deliverables: google_handler.rs, tests (with E2E test harness)
  - Est. effort: 3 days

WP03: GitHub Login Flow
  - Implement GET /auth/github/redirect endpoint
  - Implement GET /auth/github/callback endpoint
  - Exchange auth code for token
  - Fetch user profile from GitHub API
  - Create user record if new
  - Set session cookie
  - Deliverables: github_handler.rs, tests
  - Est. effort: 3 days

WP04: Session & Logout
  - Extend session middleware to support OAuth sessions
  - Implement GET /auth/logout endpoint
  - Add session expiration logic
  - Add session invalidation on logout
  - Deliverables: session_ext.rs, logout_handler.rs, tests
  - Est. effort: 2 days

Dependency graph:
  WP01 (no dependencies)
  WP02 ← WP01
  WP03 ← WP01
  WP04 ← WP02, WP03

Critical path: WP01 → WP02 → WP04 (8 days)
Parallel work: WP02 and WP03 can run simultaneously
```

## Phase 2: Implementation

### 4. Implement Work Package 1

Start with provider configuration (no dependencies):

```bash
agileplus implement 001 --wp WP01
```

```
Creating worktree for WP01...
  Worktree path: .worktrees/001-oauth2-auth-WP01
  Branch: feat/001-oauth2-auth-WP01
  Base: main (at commit abc1234)

Dispatching to Claude Code agent...
  Session: sess_w7k2j3l
  Prompt: [500 lines of structured context with spec, plan, and deliverables]

Agent is working. You can track progress with:
  agileplus status 001 --watch
```

Check progress:

```bash
agileplus status 001
```

```
Feature 001: OAuth2 Authentication

WP01  Provider Configuration  ████████░░░░ doing
  Commits: 2
  Files changed: 3
  Latest commit: "feat(WP01): add OAuthProvider enum and config"

WP02  Google Login Flow       ░░░░░░░░░░░░ planned (blocked by WP01)
WP03  GitHub Login Flow       ░░░░░░░░░░░░ planned (blocked by WP01)
WP04  Session & Logout        ░░░░░░░░░░░░ planned (blocked by WP02, WP03)
```

The agent completes WP01, opens files in the worktree, writes code, and commits with proper messages. Review the changes:

```bash
cd .worktrees/001-oauth2-auth-WP01
git log --oneline -5
```

```
e7f3b2a feat(WP01): add OAuthProvider enum and config
d4c1a9f feat(WP01): add OAuth configuration validation
c2b8e7f feat(WP01): add OAuthToken and OAuthSession models
```

Move to review:

```bash
agileplus move 001 WP01 --to for_review
```

```
WP01 moved to for_review
  Commits: 3 (ahead of main)
  Files changed: 5
  Lines added: 247
  Tests: 8 passing
```

### 5. Review Work Package 1

Run automated checks:

```bash
agileplus review 001 WP01
```

```
Reviewing WP01 against spec and plan...

Specification Check:
  ✓ All deliverables present (providers.rs, oauth_config.rs)
  ✓ Covers "Register OAuth apps" requirement
  ✓ Covers "Add OAuth2 config" requirement

Implementation Check:
  ✓ All tests pass locally (8/8)
  ✓ No files modified outside WP scope
  ✓ Commit messages reference WP01
  ✓ Code follows project conventions (clippy clean)

Governance Check:
  ✓ Audit trail intact
  ✓ Hash chain valid

Result: APPROVED ✓
WP01 moved to done
```

Unblock dependent work:

```bash
agileplus status 001
```

```
WP01  Provider Configuration  ████████████ done
WP02  Google Login Flow       ░░░░░░░░░░░░ planned (ready to start)
WP03  GitHub Login Flow       ░░░░░░░░░░░░ planned (ready to start)
WP04  Session & Logout        ░░░░░░░░░░░░ planned (still blocked)
```

### 6. Parallel Implementation: WP02 & WP03

Start both Google and GitHub flows in parallel:

```bash
# Terminal 1: Dispatch WP02
agileplus implement 001 --wp WP02 --agent claude-code

# Terminal 2: Dispatch WP03
agileplus implement 001 --wp WP03 --agent codex

# Terminal 3: Monitor progress
agileplus status 001 --watch
```

```
Feature 001: OAuth2 Authentication

WP01  Provider Configuration  ████████████ done
WP02  Google Login Flow       ████████░░░░ doing (by claude-code)
WP03  GitHub Login Flow       ██████░░░░░░ doing (by codex)
WP04  Session & Logout        ░░░░░░░░░░░░ planned (blocked)
```

After both complete:

```bash
agileplus move 001 WP02 --to for_review
agileplus move 001 WP03 --to for_review
agileplus review 001 WP02 && agileplus review 001 WP03
```

Both pass review and move to `done`.

### 7. Implement WP04 (Session Management)

Now the final piece can start (unblocked):

```bash
agileplus implement 001 --wp WP04 --agent claude-code
```

Agent completes session and logout handlers. Review and approve:

```bash
agileplus move 001 WP04 --to for_review
agileplus review 001 WP04  # → APPROVED
```

## Phase 3: Validation & Shipping

### 8. Validate Feature Completeness

Before shipping, ensure the entire feature is ready:

```bash
agileplus validate 001
```

```
Validating feature 001...

Artifacts
  ✓ spec.md present and valid
  ✓ plan.md present and valid
  ✓ All WPs have task.md

Completeness
  ✓ All 4 WPs are in 'done' state
  ✓ All functional requirements mapped to WPs
  ✓ All acceptance criteria have test coverage
  ✓ No open clarification markers (?)

Code Quality
  ✓ All tests pass (36/36)
  ✓ Code coverage: 87% (target: 80%)
  ✓ No clippy warnings
  ✓ No merge conflicts detected

Governance
  ✓ Audit trail complete and signed
  ✓ All WP branches valid
  ✓ No unauthorized modifications

Result: READY TO SHIP ✓
```

### 9. Ship the Feature

Merge all work packages and cut a release:

```bash
agileplus ship 001
```

```
Merging work packages in dependency order...

Merging WP01 → main
  Branch: feat/001-oauth2-auth-WP01
  Commits: 3
  Files: 5
  ✓ Merged

Merging WP02 → main
  Branch: feat/001-oauth2-auth-WP02
  Commits: 4
  Files: 6
  ✓ Merged

Merging WP03 → main
  Branch: feat/001-oauth2-auth-WP03
  Commits: 4
  Files: 5
  ✓ Merged

Merging WP04 → main
  Branch: feat/001-oauth2-auth-WP04
  Commits: 2
  Files: 3
  ✓ Merged

Creating release tag: v0.2.0-oauth2-auth
Updating CHANGELOG.md
Recording completion timestamp

Feature 001 shipped successfully!
  Total commits: 13
  Total files changed: 19
  Total lines added: 587
  Timeline: 8 working days (Sept 1–10)
```

### 10. Generate Retrospective

After shipping, capture learnings:

```bash
agileplus retrospective 001
```

```
Retrospective for feature 001: OAuth2 Authentication

Timeline
  Specification → Done: 1 day
  Planning → Done: 0.5 days
  Implementation → Done: 8 working days
  Total: 9.5 days (vs. estimated 9 days)

Work Package Stats
  WP01  Provider Config    — 3 commits, 2 days (agent)
  WP02  Google Login       — 4 commits, 3 days (agent)
  WP03  GitHub Login       — 4 commits, 3 days (agent, parallel)
  WP04  Session & Logout   — 2 commits, 2 days (agent)

Review Cycles
  WP01  1 review round (approved first time)
  WP02  1 review round (approved first time)
  WP03  1 review round (approved first time)
  WP04  1 review round (approved first time)

Insights
  ✓ Spec-first approach prevented scope creep
  ✓ Parallel WP execution saved 3 days
  ✓ Clear deliverables reduced rework
  ✗ Missing error handling in callback handlers (caught in review)
  → Action: Add error scenario testing to constitution

Agent Performance
  claude-code: 3 WPs, 100% pass rate, avg 2.67 days/WP
  codex: 1 WP, 100% pass rate, 3 days

Dependency Management
  Critical path was WP01 → WP02 → WP04 (8 days)
  Parallelization of WP02/WP03 saved 3 days vs. sequential
```

## Key Takeaways

1. **Specification first** — Clear requirements before planning avoids rework
2. **Dependency awareness** — Parallelization accelerates delivery
3. **Delivery artifacts** — Each WP has a clear list of files to deliver
4. **Governance at every step** — Automated validation catches issues early
5. **Agent dispatch** — Agents work within guardrails defined by spec and plan
6. **Retrospectives** — Learnings feed back into the constitution for next time
