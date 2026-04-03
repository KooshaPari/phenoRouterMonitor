---
audience: [agents, developers]
---

# Agent Integration Example

End-to-end example of dispatching an AI agent to implement a work package. Shows configuration, prompt format, and expected agent behavior.

## Scenario

You have a feature `003-auth-system` with three work packages:
- WP01: User & Session models (DONE)
- WP02: Login endpoint (PLANNED, ready to implement)
- WP03: Session middleware (BLOCKED until WP02 completes)

## Step 1: Configure Agent Harness

Before dispatching, configure the agent connection in `.kittify/config.yaml`:

```yaml
agent:
  harness: claude-code
  timeout_seconds: 3600
  retries: 2
  environment:
    RUST_LOG: debug
    DATABASE_URL: postgresql://localhost/testdb
  constraints:
    max_file_size: 1000000  # 1MB per file
    allowed_paths:
      - "src/"
      - "tests/"
      - "Cargo.toml"
    forbidden_paths:
      - ".kittify/"
      - "env_secrets/"
      - "package-lock.json"  # use Cargo.lock
```

Check agent availability:

```bash
agileplus agents list
```

```
Available Agent Harnesses:

claude-code (active)
  Status: connected
  Model: Claude Opus 4.6
  Last heartbeat: 1 second ago
  Sessions: 2 active

codex (available)
  Status: idle
  Model: Codex v2
  Last heartbeat: 5 minutes ago
  Sessions: 0 active

Select harness with --agent flag or set default in config.
```

## Step 2: Check WP Status

```bash
agileplus status 003
```

```
Feature: 003-auth-system

State: Implementing

WP01  User & Session Models      ████████████ done
  Commits: 4  |  Files: 3  |  Tests: 18/18 passing
  Agent: claude-code  |  Completed: 2 days ago

WP02  Login Endpoint             ░░░░░░░░░░░░ planned (ready)
  Deliverables: login_handler.rs, tests
  Blocked by: WP01 (unblocked)
  Est. effort: 3 days

WP03  Session Middleware         ░░░░░░░░░░░░ planned (blocked)
  Blocked by: WP02
  Est. effort: 2 days

Total WP progress: 1/3 (33%)
Estimated completion: 5 days
```

## Step 3: Dispatch Agent to WP02

Dispatch the agent with all context:

```bash
agileplus implement 003 --wp WP02 --agent claude-code
```

Behind the scenes, AgilePlus:
1. Creates a worktree at `.worktrees/003-auth-system-WP02`
2. Checks out a new branch `feat/003-auth-system-WP02` from main
3. Packages the prompt with spec, plan, and governance constraints
4. Sends to Claude Code harness

```
Dispatching WP02 to Claude Code...

Workspace setup:
  Worktree: .worktrees/003-auth-system-WP02
  Branch: feat/003-auth-system-WP02
  Base commit: abc1234 (main)

Prompt prepared:
  Feature spec: 003-auth-system (2.1 KB)
  Work package plan: WP02 (1.8 KB)
  Deliverables list: 3 files
  Governance constraints: 1.2 KB
  Total prompt: 6.1 KB

Dispatching to Claude Code harness...
  Session: sess_w7k2j3l9m
  Model: Claude Opus 4.6
  Started: 2024-03-01 10:15:33 UTC

Waiting for agent to initialize...
Agent ready. Working on WP02.
```

## Step 4: Agent Receives the Prompt

The agent receives a structured prompt like this:

```
# MISSION

Implement WP02: Login Endpoint for feature 003-auth-system

## CONTEXT

Feature: User Authentication System
  Overview: Add login/logout functionality with JWT tokens
  Timeline: March 1–15
  Status: In Implementation (WP01 complete)

## SPECIFICATION

### Functional Requirements (from spec.md)
FR-1: Users can log in with email and password
  - Accept POST /api/auth/login with { email, password }
  - Return JWT token if valid
  - Return 401 if invalid
  - Rate limit: 5 attempts per minute per IP

FR-2: JWT tokens are validated on subsequent requests
  - Check Authorization: Bearer <token>
  - Validate signature and expiration
  - Return 401 if invalid

FR-3: Sessions persist across requests
  - Tokens valid for 7 days
  - Can refresh token via POST /api/auth/refresh
  - Refresh tokens valid for 30 days

### Success Criteria
- Both endpoints respond in <200ms (p95)
- 100% test coverage for happy path + error cases
- Rate limiting prevents brute force attempts
- Token validation prevents unauthorized access

## WORK PACKAGE PLAN

WP02: Login Endpoint
  Depends on: WP01 (User & Session models) ✓ DONE
  Blocks: WP03 (Session middleware)

### Architecture Decision
- Use Actix-web middleware for token validation
- JWT library: jsonwebtoken 9.1
- Redis for rate limiting (already set up in WP01)

### Deliverables
1. src/handlers/login.rs
   - POST /api/auth/login endpoint
   - Password validation via bcrypt
   - JWT generation (7-day expiry)
   - Rate limiting middleware

2. src/handlers/refresh.rs
   - POST /api/auth/refresh endpoint
   - Validate refresh token (30-day expiry)
   - Generate new access token

3. tests/login_handler_test.rs
   - Happy path: valid email/password → token
   - Invalid email → 401
   - Invalid password → 401
   - Rate limiting: 6th attempt → 429
   - Token refresh: valid token → new token
   - Expired token → 401
   - Malformed token → 401

### Test Data
  Test user: test@example.com / password123 (hashed)
  Test database: PostgreSQL testdb (WP01 provides fixtures)

## GOVERNANCE CONSTRAINTS

Constitution: Must follow Rust best practices
  ✓ All code must pass: cargo clippy -- -D warnings
  ✓ Test coverage: >= 85% per file
  ✓ No unwrap() in library code (use ? or Result)
  ✓ Function length: < 50 lines preferred
  ✗ Cannot modify: .kittify/*, env_secrets/*, package-lock.json
  ✗ Cannot execute: System commands that modify production data

Agent Constraints
  ✓ Can create/modify: src/handlers/*, tests/*
  ✓ Can modify: Cargo.toml (dependencies), src/lib.rs (exports)
  ✗ Cannot modify: src/models/* (WP01's domain)
  ✗ Cannot run: cargo test (will run at review time)

Commit Convention
  Format: feat(WP02): <description>
  Examples:
    - feat(WP02): implement login endpoint with JWT
    - feat(WP02): add rate limiting for login attempts
    - feat(WP02): add login handler tests

## ENVIRONMENT

Project: AgilePlus auth demo
Framework: Actix-web 4.4 + Tokio
Database: PostgreSQL (migrations already applied via WP01)
Cache: Redis (available at localhost:6379)

Current main branch includes:
- User model with password hashing
- Session model and database schema
- Rate limiting infrastructure
- Test fixtures and factories

## YOUR TASK

Implement WP02 according to spec, plan, and acceptance criteria.

When done:
1. Create commits with the convention above
2. Leave the worktree on branch feat/003-auth-system-WP02
3. All tests must pass locally
4. No modifications outside src/handlers/, src/middleware/, tests/

You have 1 hour. Let's go!
```

## Step 5: Agent Works

The agent takes action in the worktree:

```bash
# Agent creates the login handler
$ cat > src/handlers/login.rs <<'EOF'
use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::models::{User, Claims};

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
    refresh_token: String,
}

#[post("/api/auth/login")]
pub async fn login(req: web::Json<LoginRequest>) -> HttpResult<HttpResponse> {
    // Validate email/password
    let user = User::find_by_email(&req.email)
        .await
        .ok_or(AuthError::InvalidCredentials)?;

    if !user.verify_password(&req.password) {
        return Err(AuthError::InvalidCredentials.into());
    }

    // Generate JWT
    let claims = Claims::new(user.id, 7 * 24 * 3600); // 7 days
    let token = encode(&Header::default(), &claims, &ENCODING_KEY)?;

    let refresh_claims = Claims::new(user.id, 30 * 24 * 3600); // 30 days
    let refresh_token = encode(&Header::default(), &refresh_claims, &ENCODING_KEY)?;

    Ok(HttpResponse::Ok().json(LoginResponse { token, refresh_token }))
}
EOF

# Agent commits
$ git add src/handlers/login.rs
$ git commit -m "feat(WP02): implement login endpoint with JWT"
```

Agent creates multiple commits:

```bash
$ git log --oneline -5
```

```
4e7a3c2 feat(WP02): add rate limiting for login attempts
a8f2b5e feat(WP02): add login handler tests (15 test cases)
d6c1a9f feat(WP02): implement login endpoint with JWT
c3b4a2e feat(WP02): add refresh token endpoint
```

## Step 6: Monitor Agent Progress

Monitor in real-time:

```bash
agileplus status 003 --watch
```

```
Feature: 003-auth-system

WP01  User & Session Models      ████████████ done
WP02  Login Endpoint             ██████░░░░░░ doing  (claude-code)
  Session: sess_w7k2j3l9m
  Started: 10:15 UTC
  Files changed: 4
  Commits: 2
  Current task: writing tests

WP03  Session Middleware         ░░░░░░░░░░░░ planned (blocked)
```

Check detailed logs:

```bash
agileplus logs WP02 --tail 20
```

```
[10:15:33] Agent session started
[10:16:12] Reading specification: spec.md (2.1 KB)
[10:16:15] Reading plan: plan.md (1.8 KB)
[10:17:45] Created src/handlers/login.rs (245 lines)
[10:18:02] Commit: feat(WP02): implement login endpoint with JWT
[10:18:45] Created tests/login_test.rs (312 lines)
[10:19:20] Running cargo test locally...
[10:19:55] Tests passing: 15/15
[10:20:10] Commit: feat(WP02): add login handler tests
[10:21:30] Working on rate limiting middleware...
```

## Step 7: Agent Completes & Moves to Review

Agent finishes and signals completion:

```bash
agileplus status 003
```

```
Feature: 003-auth-system

WP01  User & Session Models      ████████████ done
WP02  Login Endpoint             ██████░░░░░░ for_review
  Commits: 4 (ahead of main)
  Files: 6 changed
  Tests: 18 passing
  Agent: claude-code (completed)

WP03  Session Middleware         ░░░░░░░░░░░░ planned (blocked)
```

## Step 8: Code Review

Run automated review:

```bash
agileplus review 003 WP02
```

```
Reviewing WP02: Login Endpoint

Deliverables Check
  ✓ src/handlers/login.rs (245 lines, 18 tests)
  ✓ src/handlers/refresh.rs (187 lines, 8 tests)
  ✓ tests/login_handler_test.rs (312 lines)
  ✓ All 3 deliverables present

Test Quality
  ✓ All tests pass (26/26)
  ✓ Test coverage: 92% (target: 85%)
  ✓ Tests cover: happy path, errors, rate limiting, refresh

Code Quality
  ✓ No clippy warnings
  ✓ Formatted with rustfmt
  ✓ Function length OK (max: 48 lines, target: 50)
  ✓ No unwrap() in library code ✓

Specification Compliance
  ✓ POST /api/auth/login implemented
  ✓ JWT generation (7-day expiry) ✓
  ✓ Rate limiting (5 per minute) ✓
  ✓ Response time < 200ms (tested)
  ✓ Password validation via bcrypt ✓

Governance
  ✓ Commit messages follow convention
  ✓ Only src/handlers/, src/middleware/, tests/ modified
  ✓ No forbidden files touched
  ✓ Audit trail intact

Result: APPROVED ✓
```

## Step 9: Merge & Unblock Next WP

```bash
agileplus move 003 WP02 --to done
```

```
WP02 moved to done
  Merging branch feat/003-auth-system-WP02 → main
  ✓ Merged (4 commits, 6 files)

Dependency resolution:
  WP03 (Session Middleware) is now unblocked
```

Check status:

```bash
agileplus status 003
```

```
Feature: 003-auth-system

WP01  User & Session Models      ████████████ done
WP02  Login Endpoint             ████████████ done
  Agent: claude-code  |  Completed: 45 minutes
  Review cycles: 1 (approved on first pass)

WP03  Session Middleware         ░░░░░░░░░░░░ ready (unblocked)
```

## Step 10: Dispatch Next Agent (Parallel)

Now WP03 is unblocked. Dispatch another agent:

```bash
agileplus implement 003 --wp WP03 --agent claude-code
```

```
Dispatching WP03 to Claude Code...

Workspace setup:
  Worktree: .worktrees/003-auth-system-WP03
  Branch: feat/003-auth-system-WP03
  Base commit: 4e7a3c2 (main - includes WP02)

Prompt prepared:
  Feature spec: 003-auth-system (2.1 KB)
  Work package plan: WP03 (1.6 KB)
  Deliverables: 2 files
  Constraint notes: WP02 completion unblocks this
  Total: 5.5 KB

Dispatching to Claude Code...
  Session: sess_k9l3m5n
  Started: 2024-03-01 11:05:22 UTC

Agent ready. WP03 in progress.
```

## Key Configuration & Constraints

### Agent Environment Variables

```yaml
agent:
  environment:
    RUST_LOG: info
    DATABASE_URL: postgresql://localhost/testdb
    REDIS_URL: redis://localhost:6379
    JWT_SECRET: ${{ secrets.JWT_SECRET }}  # Injected at runtime
```

### What Agents CAN Do

- ✓ Create new files in `src/`
- ✓ Modify existing implementation files
- ✓ Write comprehensive tests
- ✓ Update `Cargo.toml` dependencies
- ✓ Add new modules/re-exports
- ✓ Run tests locally (`cargo test`)
- ✓ Commit code with proper messages

### What Agents CANNOT Do

- ✗ Modify governance files (`.kittify/`, `constitution.md`)
- ✗ Run system commands that affect production
- ✗ Execute database migrations outside the plan
- ✗ Modify work from other WPs without explicit override
- ✗ Change git history (no force push)

## Key Takeaways

1. **Clear context** — Agents receive spec, plan, and constraints
2. **Isolated workspaces** — Each WP gets its own worktree and branch
3. **Automated governance** — Constraints prevent unauthorized modifications
4. **Real-time monitoring** — Watch agent progress via CLI
5. **Quality gates** — Automated review before merge
6. **Dependency awareness** — Unblocking happens automatically when dependencies complete

## Complete Audit Trail After WP02

After WP02 completes and is merged, the audit chain for `003-auth-system` looks like:

```jsonl
{"id":1,"feature_id":3,"wp_id":null,"timestamp":"2026-03-01T08:00:00Z","actor":"human:alice","transition":"created->specified","prev_hash":"0000...","hash":"1a2b..."}
{"id":2,"feature_id":3,"wp_id":null,"timestamp":"2026-03-01T10:00:00Z","actor":"agent:claude-code","transition":"specified->researched","prev_hash":"1a2b...","hash":"3c4d..."}
{"id":3,"feature_id":3,"wp_id":null,"timestamp":"2026-03-01T12:00:00Z","actor":"agent:claude-code","transition":"researched->planned","prev_hash":"3c4d...","hash":"5e6f..."}
{"id":4,"feature_id":3,"wp_id":1,"timestamp":"2026-03-01T13:00:00Z","actor":"agent:claude-code","transition":"WP01:planned->doing","prev_hash":"5e6f...","hash":"7a8b..."}
{"id":5,"feature_id":3,"wp_id":1,"timestamp":"2026-03-01T16:00:00Z","actor":"agent:claude-code","transition":"WP01:doing->review","prev_hash":"7a8b...","hash":"9c0d..."}
{"id":6,"feature_id":3,"wp_id":1,"timestamp":"2026-03-01T17:00:00Z","actor":"ci:automation","transition":"WP01:review->done","prev_hash":"9c0d...","hash":"1e2f...","evidence_refs":[{"fr_id":"FR-001","evidence_id":1}]}
{"id":7,"feature_id":3,"wp_id":2,"timestamp":"2026-03-02T09:00:00Z","actor":"agent:claude-code","transition":"WP02:planned->doing","prev_hash":"1e2f...","hash":"3a4b..."}
{"id":8,"feature_id":3,"wp_id":2,"timestamp":"2026-03-02T11:00:00Z","actor":"agent:claude-code","transition":"WP02:doing->review","prev_hash":"3a4b...","hash":"5c6d..."}
{"id":9,"feature_id":3,"wp_id":2,"timestamp":"2026-03-02T12:00:00Z","actor":"ci:automation","transition":"WP02:review->done","prev_hash":"5c6d...","hash":"7e8f..."}
```

Verify the chain at any time:

```bash
agileplus events audit-verify --feature 003-auth-system
# ✓ Audit chain intact (9 entries verified)
# Hash coverage: created (id=1) → WP02 done (id=9)
```

## NATS Events Flow for This Example

Every transition above published a NATS message. Here is the full message flow for WP02:

```
Subject: agileplus.feature.003-auth-system.wp.WP02.state.changed
Payload: {"from":"planned","to":"doing","actor":"agent:claude-code","job_id":"sess_w7k2j3l9m"}

Subject: agileplus.agent.job.sess_w7k2j3l9m.started
Payload: {"feature_slug":"003-auth-system","wp_id":"WP02","pid":54321}

Subject: agileplus.agent.job.sess_w7k2j3l9m.commit
Payload: {"sha":"4e7a3c2","message":"feat(WP02): implement login endpoint with JWT"}

Subject: agileplus.agent.job.sess_w7k2j3l9m.commit
Payload: {"sha":"a8f2b5e","message":"feat(WP02): add login handler tests (15 test cases)"}

Subject: agileplus.agent.job.sess_w7k2j3l9m.completed
Payload: {"success":true,"commits":4,"files_changed":6,"tests_passed":26}

Subject: agileplus.feature.003-auth-system.wp.WP02.state.changed
Payload: {"from":"doing","to":"review","actor":"agent:claude-code"}

Subject: agileplus.feature.003-auth-system.wp.WP02.state.changed
Payload: {"from":"review","to":"done","actor":"ci:automation"}

Subject: agileplus.feature.003-auth-system.wp.WP03.unblocked
Payload: {"wp_id":"WP03","unblocked_by":"WP02","ready_to_start":true}
```

The dashboard receives all these via SSE and updates in real-time. No polling required.

## Error Handling: What If the Agent Fails?

If Claude Code exits non-zero or times out:

```bash
agileplus implement 003 --wp WP02 --agent claude-code
```

```
Dispatching WP02 to Claude Code...
...
Agent session timeout after 1800 seconds.
Capturing partial output...

Agent failed:
  Exit code: -1 (timeout)
  Commits made: 2 (partial work)
  Tests: not run

WP02 moved to Blocked state.
Audit entry recorded: actor=system, transition=WP02:doing->blocked
Reason: Agent timeout (SIGKILL after 30s warning period)

Next steps:
  Option 1: Review partial commits and continue manually
    cd .worktrees/003-auth-system-WP02
    git log --oneline   # see what was done
    # ... continue work ...
    agileplus wp transition WP02 --to review

  Option 2: Retry with fresh agent session
    agileplus wp transition WP02 --to planned  # reset
    agileplus implement 003 --wp WP02 --agent claude-code --timeout 3600
```

The audit entry for the timeout:

```json
{
  "id": 8,
  "transition": "WP02:doing->blocked",
  "actor": "system:timeout",
  "evidence_refs": [],
  "metadata": {
    "reason": "agent_timeout",
    "timeout_secs": 1800,
    "job_id": "sess_w7k2j3l9m",
    "partial_commits": 2
  }
}
```

## Next Steps

- [Agent Dispatch](../concepts/agent-dispatch.md) — Architecture deep dive
- [Prompt Format](../agents/prompt-format.md) — What Claude Code receives
- [Governance Constraints](../agents/governance-constraints.md) — What agents can do
- [Harness Integration](../agents/harness-integration.md) — Adding new agent adapters
- [MCP Tools](../sdk/mcp-tools.md) — Programmatic agent control
- [Feature Lifecycle](../concepts/feature-lifecycle.md) — Full lifecycle from created to shipped
