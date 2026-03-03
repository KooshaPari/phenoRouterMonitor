---
audience: [agents]
---

# Agent Prompt Format & Specification

AgilePlus delivers structured prompts to AI agents via harness subprocesses. This document specifies the exact format agents receive, expected response patterns, and integration points.

## Prompt Delivery Mechanism

The agent dispatch layer (agileplus-agent-dispatch) sends prompts via process stdin/stdout:

```
┌─────────────────────────┐
│  AgilePlus Orchestrator │
│  (agileplus-cli)        │
└────────────┬────────────┘
             │ spawn subprocess
             │
┌────────────▼────────────┐
│  Agent Harness          │
│  (Claude Code, Cursor)  │
└────────────┬────────────┘
             │ stdin: JSON task envelope
             │
┌────────────▼────────────┐
│  AI Agent Process       │
│  (claude, cursor, etc.) │
└────────────┬────────────┘
             │ stdout: structured output
             ▼
```

## Full Prompt Format

Agents receive a multi-section markdown prompt document via the `prompt_path` field.

### Task Envelope (JSON)

```json
{
  "job_id": "3a6b8c9d-1e2f-4a5b-8c9d-1e2f4a5b8c9d",
  "feature_slug": "001-user-login",
  "wp_sequence": 1,
  "wp_id": "WP01",
  "prompt_path": "/path/to/.worktrees/001-login-WP01/WP01.md",
  "context_paths": [
    "/path/to/kitty-specs/001-user-login/spec.md",
    "/path/to/kitty-specs/001-user-login/plan.md",
    "/path/to/ARCHITECTURE.md",
    "/path/to/GOVERNANCE.md"
  ],
  "worktree_path": "/path/to/.worktrees/001-login-WP01",
  "config": {
    "kind": "claude_code",
    "timeout_secs": 1800,
    "pr_target_branch": "feat/001-login-WP01",
    "num_agents": 1,
    "max_review_cycles": 5
  }
}
```

### Markdown Prompt Document (WP01.md)

The prompt file is structured, human-readable markdown with clear sections:

```markdown
# Task: WP01 — Implement Login Form Component

## Mission

Build a React login form component that supports email/password authentication with validation and error display. This is the user-facing entry point for the authentication system.

## Feature Context

- **Feature:** 001-user-login (User Login System)
- **Feature State:** IMPLEMENT
- **Target Branch:** feat/001-login-WP01
- **Created:** 2025-01-15T10:30:00Z

## Deliverables

### Code
- [ ] `src/components/LoginForm.tsx` — React component with form validation
- [ ] `src/hooks/useLogin.ts` — Custom hook for login mutation
- [ ] `src/styles/LoginForm.module.css` — Component styles

### Tests
- [ ] `src/components/__tests__/LoginForm.test.tsx` — Unit tests (>80% coverage)
- [ ] `src/hooks/__tests__/useLogin.test.ts` — Hook tests

### Documentation
- [ ] `src/components/LoginForm.md` — Component storybook

## Work Scope (File Boundaries)

You are authorized to create or modify **only** these files:

```
src/components/LoginForm.tsx
src/components/__tests__/LoginForm.test.tsx
src/components/LoginForm.md
src/hooks/useLogin.ts
src/hooks/__tests__/useLogin.test.ts
src/styles/LoginForm.module.css
```

Any changes to other files (except lock files) require governance exception.

## Constraints & Governance

### Functional Requirements (FR)

- **FR-AUTH-001:** Form must validate email format (RFC 5322)
- **FR-AUTH-002:** Password must be at least 8 characters
- **FR-AUTH-003:** Error messages must display inline under invalid fields
- **FR-AUTH-004:** Form must disable submit button while request is in-flight

### Architecture Rules

- [ ] Use existing `src/services/auth.ts` service (do NOT rewrite)
- [ ] Use Tailwind CSS for styling (do NOT add CSS-in-JS)
- [ ] Use React hooks, not class components
- [ ] All async operations must use proper error boundaries
- [ ] No unhandled promise rejections

### Code Quality Gates

- [ ] ESLint must pass: `npm run lint`
- [ ] TypeScript must pass: `npm run typecheck`
- [ ] Tests must pass: `npm run test` (>80% coverage required)
- [ ] No console errors/warnings in test runs

### Dependencies

- **May add:** react, react-dom, zod (validation)
- **May NOT add:** external auth libs (use auth.ts service)
- **May NOT change:** tsconfig.json, package.json root entries

## Context Documents

Your context includes:

1. **Specification** (`spec.md`):
   - Full feature mission and user stories
   - Acceptance criteria for all work packages
   - Links to designs and prototypes

2. **Implementation Plan** (`plan.md`):
   - Work breakdown: WP01 through WP03
   - Dependencies and parallelization strategy
   - Risk mitigation plans

3. **Existing Code Structure** (`ARCHITECTURE.md`):
   - Component hierarchy
   - Service layer organization
   - Authentication flow diagrams

4. **Governance & Audit** (`GOVERNANCE.md`):
   - State transition rules
   - Review gate requirements
   - Non-repudiation audit log format

## Workflow & State Transitions

### Current State
- Feature: `IMPLEMENT` (all specs approved, plan complete)
- Work Package: `PLANNED` (ready for your work)
- Worktree: `.worktrees/001-login-WP01` (isolated environment)
- Branch: `feat/001-login-WP01` (created from main)

### Your Actions

1. **Initialize** (first run):
   - Read `spec.md` and `plan.md`
   - Review existing `src/services/auth.ts`
   - Set up test environment

2. **Develop**:
   - Create/edit files listed in deliverables
   - Commit frequently with message format: `WP01: [description]`
   - Run tests and linting: `npm run test && npm run lint`

3. **Finalize**:
   - Ensure all deliverables exist and tests pass
   - Make final commit: `WP01: Complete implementation`
   - All changes must be committed (no staged/unstaged)

4. **Submit for Review**:
   - When ready, exit with code 0
   - Output your final status to stdout (see Response Format below)
   - Do NOT manually transition state (harness does this)

### State Diagram

```
PLANNED ──[your work]──> DOING ──[tests pass]──> FOR_REVIEW ──[approved]──> DONE
         (git commits)         (PR created)                   (merged)
```

## Available Tools & Commands

### Git Operations (via subcommands)

```bash
# View current state
git status
git log --oneline -10

# Stage and commit
git add src/components/LoginForm.tsx
git commit -m "WP01: Implement form component"

# Create branches (already done, but shown for reference)
git branch --list
git checkout -b feat/001-login-WP02 feat/001-login-WP01
```

### Build & Test

```bash
# Install dependencies (if needed)
npm install

# Type check
npm run typecheck

# Lint
npm run lint

# Run tests
npm run test -- --coverage

# Build
npm run build
```

### Harness Commands

The harness provides these commands for state management:

```bash
# Check current state
agileplus status WP01

# List ready work packages (dependencies satisfied)
agileplus worktree list

# Create artifact (for evidence)
agileplus artifact write \
  --feature-slug 001-login \
  --relative-path WP01/test-report.txt \
  --content "Tests: 42 passed, 0 failed"
```

## Response Format (Exit & Output)

When you finish (success or failure), output JSON to stdout:

### Success Response

```json
{
  "job_id": "3a6b8c9d-1e2f-4a5b-8c9d-1e2f4a5b8c9d",
  "wp_id": "WP01",
  "success": true,
  "status": "completed",
  "summary": "Implemented LoginForm component with full test coverage",
  "commits": [
    {
      "sha": "abc123def456...",
      "message": "WP01: Implement login form with validation"
    },
    {
      "sha": "def456ghi789...",
      "message": "WP01: Add tests for LoginForm component"
    }
  ],
  "artifacts": {
    "files_created": [
      "src/components/LoginForm.tsx",
      "src/components/__tests__/LoginForm.test.tsx",
      "src/hooks/useLogin.ts",
      "src/styles/LoginForm.module.css"
    ],
    "test_coverage": "87.5%",
    "lint_status": "passed"
  },
  "pr_url": "https://github.com/org/repo/pull/42",
  "duration_seconds": 1847,
  "next_steps": [
    "Await code review from @reviewer",
    "WP02 can start in parallel (no dependencies)"
  ]
}
```

### Failure Response

```json
{
  "job_id": "3a6b8c9d-1e2f-4a5b-8c9d-1e2f4a5b8c9d",
  "wp_id": "WP01",
  "success": false,
  "status": "blocked",
  "error": "Cannot modify src/services/auth.ts (outside file scope)",
  "commits": [
    {
      "sha": "abc123def456...",
      "message": "WP01: Implement component (incomplete)"
    }
  ],
  "blocked_reason": "File scope violation",
  "remediation": "Contact human: Need to refactor auth.ts in separate WP",
  "duration_seconds": 342,
  "cleanup_required": true
}
```

## Frontmatter Tags

All context documents use YAML frontmatter. Agent-relevant pages have:

```yaml
---
audience: [agents, developers]
---
```

Pages with `agents` in audience contain agent-specific instructions.

## Error Handling & Escalation

### Recoverable Errors (Retry)

If you hit these, attempt recovery:
- Network timeouts → retry the operation
- Transient test failures → run again
- Temporary lock files → wait and retry

### Unrecoverable Errors (Escalate)

Do NOT retry; instead document and exit with status "blocked":

```json
{
  "success": false,
  "status": "blocked",
  "error": "FR-AUTH-001 cannot be satisfied: email regex library required",
  "blocked_reason": "Dependency not permitted (governance violation)",
  "remediation": "Human must approve zod library addition"
}
```

### Blocker Logging

Log blockers in the harness output, not just stderr:

```bash
echo "BLOCKER: Cannot import @babel/preset-react (not in approved list)" >&2
# Exit with non-zero code
exit 1
```

## Timeout Behavior

The harness has a timeout (default 1800s = 30 min):

- **At 90%:** Harness sends SIGTERM; you have 30s to clean up
- **At 100%:** Harness sends SIGKILL; process terminates immediately

Always check timeouts in long operations:

```typescript
// Check remaining time
const elapsed = Date.now() - startTime;
if (elapsed > timeoutMs * 0.9) {
  console.log("Approaching timeout, finalizing...");
  process.exit(0); // Clean exit
}
```

## Manifest Validation

Before exiting, the harness validates your deliverables:

```bash
# Missing required file → REJECTED
[ ! -f "src/components/LoginForm.tsx" ] && exit 1

# Tests don't pass → REJECTED
npm run test --bail 2>&1 | grep -q "FAIL" && exit 1

# Linting fails → REJECTED (unless warning-only)
npm run lint --strict 2>&1 | grep -E "error:" && exit 1
```

All deliverables must exist and tests must pass for the harness to accept completion.

## Rust Project Prompt Specifics

For Rust codebase targets (the primary AgilePlus use case), the prompt includes additional sections:

### Rust-Specific Context

```markdown
## Rust Context

### Workspace Layout
```
crates/
├── agileplus-domain/   ← Core entities, no external deps
├── agileplus-cli/      ← Binary crate, uses engine
├── agileplus-engine/   ← Orchestration, uses adapters
├── agileplus-sqlite/   ← StoragePort impl
└── agileplus-git/      ← VcsPort impl
```

### Dependency Constraints
- `serde` v1, `tokio` v1, `sqlx` v0.7 — workspace-level, do NOT change versions
- New dependencies require governance exception if not in `Cargo.toml` already
- Use `workspace.dependencies` for shared crates

### Compilation & Test Commands
```bash
cargo check                           # Type check only (fast)
cargo build -p agileplus-domain       # Build a specific crate
cargo test -p agileplus-domain        # Test a specific crate
cargo test --all                      # Test entire workspace
cargo clippy -- -D warnings           # Lint (warnings as errors)
cargo fmt --check                     # Formatting check
```

### Code Style
- No `unwrap()` in library code — use `?` or explicit `match`
- `thiserror` for error types, `anyhow` only in binary crates
- `async/await` for all I/O operations
- `tracing::instrument` on public async functions
- Test module `#[cfg(test)] mod tests { }` inline with code
```

### spec-kitty Integration

AgilePlus is integrated with `spec-kitty` — a specification formatting assistant. When the spec document has been through spec-kitty, it will contain additional structured annotations:

```markdown
<!-- spec-kitty: validated -->
<!-- spec-kitty: fr-count: 5 -->
<!-- spec-kitty: fr-coverage: 100% -->

# Feature: User Authentication

## Functional Requirements

<!-- sk:fr id="FR-AUTH-001" priority="high" -->
- Users can register with email/password
<!-- /sk:fr -->

<!-- sk:fr id="FR-AUTH-002" priority="high" -->
- Users can log in and receive a JWT token
<!-- /sk:fr -->
```

Agents should treat `<!-- sk:fr id="FR-AUTH-001" -->` markers as authoritative requirement identifiers when referencing requirements in commit messages and acceptance criteria:

```
feat(WP01): implement User struct [FR-AUTH-001, FR-AUTH-002]
```

## Multi-Agent Coordination

When `config.num_agents > 1`, multiple instances of the same agent run in parallel on different sub-tasks. Each agent is assigned a subset of deliverables:

```json
{
  "job_id": "3a6b8c9d-...",
  "wp_id": "WP01",
  "agent_index": 0,
  "agent_count": 2,
  "assigned_deliverables": [
    "src/auth/models.rs",
    "src/auth/mod.rs"
  ]
}
```

Agent 0 (index=0) handles the first half of deliverables. Agent 1 (index=1) handles the second half. The harness merges their outputs before running validation.

Coordination rules for multi-agent:
- Each agent owns its assigned files exclusively
- No agent may touch another agent's assigned files
- Both agents commit to the same branch with atomic locking
- If either agent fails, the entire WP is marked `Blocked`

## Audit Trail for Agent Actions

Every meaningful action an agent takes is appended to the JSONL audit trail. This trail is separate from the domain audit chain — it captures the fine-grained mechanics of agent execution:

```jsonl
{"ts":"2026-03-01T10:15:34Z","actor":"agent:claude-code","job":"3a6b8c9d","action":"read_spec","path":"spec.md","hash":"0x1a2b..."}
{"ts":"2026-03-01T10:16:12Z","actor":"agent:claude-code","job":"3a6b8c9d","action":"write_file","path":"src/auth/models.rs","lines":87}
{"ts":"2026-03-01T10:17:45Z","actor":"agent:claude-code","job":"3a6b8c9d","action":"run_tests","command":"cargo test","exit_code":0,"duration_ms":4230}
{"ts":"2026-03-01T10:18:02Z","actor":"agent:claude-code","job":"3a6b8c9d","action":"commit","sha":"abc123","message":"WP01: Implement User and Session models"}
{"ts":"2026-03-01T10:18:15Z","actor":"agent:claude-code","job":"3a6b8c9d","action":"exit","code":0,"duration_total_s":162}
```

This trail is stored in `agileplus-artifacts` MinIO bucket under `audit/{feature_slug}/{job_id}.jsonl`.

## Next Steps

- [Governance Constraints](governance-constraints.md) — What agents can and cannot do
- [Harness Integration](harness-integration.md) — How to add a new agent adapter
- [Agent Dispatch](../concepts/agent-dispatch.md) — Dispatch architecture
- [MCP Tools](../sdk/mcp-tools.md) — Programmatic agent interaction
- [Environment Variables](../reference/env-vars.md) — Agent timeout and retry configuration
