---
work_package_id: WP08
title: Agent Dispatch Adapter
lane: planned
dependencies:
- WP05
subtasks:
- T044
- T045
- T046
- T047
- T048
- T049
phase: Phase 2 - Adapters
assignee: ''
agent: ''
shell_pid: ''
review_status: ''
reviewed_by: ''
history:
- timestamp: '2026-02-27T00:00:00Z'
  lane: planned
  agent: system
  shell_pid: ''
  action: Prompt generated via /spec-kitty.tasks
---

# WP08: Agent Dispatch Adapter

## Implementation Command

```bash
spec-kitty implement WP08 --base WP07
```

## Objectives

Implement the agent dispatch adapter in `crates/agileplus-agents/` that fulfills the `AgentPort` trait from WP05. This adapter spawns Claude Code and Codex as subprocesses, passes WP prompts with full context, creates PRs via `gh` CLI, and orchestrates the review-fix loop (Coderabbit comments -> agent fix -> re-push -> re-poll).

### Success Criteria

1. `AgentDispatchAdapter` implements every method of `AgentPort`.
2. Claude Code harness spawns `claude` with `--print` mode, passes WP prompt, captures output.
3. Codex harness spawns `codex` in batch mode, passes WP prompt, captures output.
4. Dispatch logic selects agent from config, creates worktree (via VcsPort), injects prompt, spawns 1-3 subagents.
5. PR creation via `gh pr create` with structured body containing WP goal, FR references, acceptance criteria (FR-011).
6. Review-fix loop polls for Coderabbit comments, feeds actionable comments to agent, re-pushes, re-polls (FR-012).
7. Mock dispatch test passes: agent spawned, PR created, review loop simulated.

## Context & Constraints

- **Agent invocation**: Via `tokio::process::Command`. Capture stdout/stderr. See `research.md` R5.
- **Agent CLI modes**: Claude Code supports `--print` for non-interactive use. Codex supports batch execution.
- **PR creation**: Via `gh pr create` CLI (not GitHub API directly). PR body follows FR-011 structure.
- **Review polling**: Poll GitHub API every 30s for review status and Coderabbit comments. Max 5 review cycles (configurable). See `plan.md` section 3.
- **Worktree dependency**: This adapter calls `VcsPort.create_worktree()` from WP07 to set up the agent's working directory.
- **Agent context**: Each agent receives the WP prompt file + spec.md + plan.md + data-model.md as context files.
- **Concurrency**: Multiple agents can run in parallel (1-3 per WP, multiple WPs in parallel). Use `tokio::spawn` for async dispatch.

## Subtask Guidance

---

### T044: Implement `AgentDispatchAdapter` struct implementing `AgentPort`

**Purpose**: Create the adapter struct that manages agent processes and implements the AgentPort trait.

**Steps**:
1. Create `crates/agileplus-agents/src/lib.rs` with the adapter struct.
2. Define `AgentDispatchAdapter`:
   ```rust
   pub struct AgentDispatchAdapter {
       vcs: Arc<dyn VcsPort + Send + Sync>,
       jobs: Arc<DashMap<String, AgentJob>>,
   }
   ```
   Where `AgentJob` tracks running agent state:
   ```rust
   struct AgentJob {
       task: AgentTask,
       config: AgentConfig,
       status: AgentStatus,
       handle: Option<JoinHandle<AgentResult>>,
   }
   ```

3. Implement constructor:
   - `pub fn new(vcs: Arc<dyn VcsPort + Send + Sync>) -> Self`

4. Implement `AgentPort` for `AgentDispatchAdapter`:
   - `dispatch`: Synchronous dispatch -- spawn agent, wait for completion, return result.
   - `dispatch_async`: Spawn agent in background via `tokio::spawn`, store job, return job ID (UUID).
   - `query_status`: Look up job in `jobs` map, return current status.
   - `cancel`: Kill the child process via `child.kill()`, mark job as failed.
   - `send_instruction`: Write instruction to agent's stdin (if supported) or create instruction file in worktree.

5. Use `DashMap` (from `dashmap` crate) for concurrent job tracking.

**Files**: `crates/agileplus-agents/src/lib.rs`

**Validation**:
- `AgentDispatchAdapter` is `Send + Sync`.
- Job IDs are unique UUIDs.
- `query_status` returns correct status for pending, running, completed jobs.

---

### T045: Implement `claude_code.rs`: Claude Code agent harness

**Purpose**: Spawn Claude Code in `--print` mode with the WP prompt and context, capture output.

**Steps**:
1. Create `crates/agileplus-agents/src/claude_code.rs`.

2. Define the spawn function:
   ```rust
   pub async fn spawn_claude_code(
       task: &AgentTask,
       config: &AgentConfig,
   ) -> Result<AgentResult, DomainError>
   ```

3. Build the command:
   ```rust
   let mut cmd = tokio::process::Command::new("claude");
   cmd.arg("--print")
      .arg("--dangerously-skip-permissions")  // for non-interactive mode
      .current_dir(&task.worktree_path);
   ```

4. Construct the prompt payload:
   - Read WP prompt file from `task.prompt_path`.
   - Append context: "Reference files: spec.md, plan.md, data-model.md".
   - Pass via stdin or `--prompt` flag depending on Claude Code's interface.

5. Execute and capture:
   ```rust
   let output = cmd.output().await?;
   let result = AgentResult {
       success: output.status.success(),
       stdout: String::from_utf8_lossy(&output.stdout).to_string(),
       stderr: String::from_utf8_lossy(&output.stderr).to_string(),
       exit_code: output.status.code().unwrap_or(-1),
       pr_url: extract_pr_url(&output.stdout),  // parse from agent output
       commits: extract_commits(&output.stdout),
   };
   ```

6. Implement `extract_pr_url()`: parse agent stdout for GitHub PR URLs (regex: `https://github.com/.+/pull/\d+`).
7. Implement `extract_commits()`: parse git log in worktree for new commits since dispatch.

8. Handle timeout: Use `tokio::time::timeout(Duration::from_secs(config.timeout_secs), ...)`.

**Files**: `crates/agileplus-agents/src/claude_code.rs`

**Validation**:
- Command is constructed with correct flags.
- Prompt payload includes WP context.
- Timeout kills the process after configured duration.
- Output parsing extracts PR URL and commit SHAs.
- Test with mock command (replace `claude` binary path in tests).

---

### T046: Implement `codex.rs`: Codex agent harness

**Purpose**: Spawn Codex in batch mode, analogous to Claude Code harness but with Codex-specific CLI flags.

**Steps**:
1. Create `crates/agileplus-agents/src/codex.rs`.

2. Define the spawn function:
   ```rust
   pub async fn spawn_codex(
       task: &AgentTask,
       config: &AgentConfig,
   ) -> Result<AgentResult, DomainError>
   ```

3. Build the command:
   ```rust
   let mut cmd = tokio::process::Command::new("codex");
   cmd.arg("--quiet")
      .arg("--approval-mode=full-auto")
      .current_dir(&task.worktree_path);
   ```

4. Prompt injection: Codex reads from stdin or accepts a prompt file path.
   - Write combined prompt to a temp file in the worktree.
   - Pass via `--prompt-file` or pipe to stdin.

5. Execute, capture, and parse output (same pattern as T045).
6. Implement timeout handling.

7. Note: The Codex CLI interface may differ from Claude Code. Design the harness to be easily updated. Key abstraction: both harnesses return `AgentResult` with the same fields.

**Files**: `crates/agileplus-agents/src/codex.rs`

**Validation**:
- Command is constructed with Codex-specific flags.
- Same `AgentResult` structure as Claude Code harness.
- Timeout handling works.
- Harness is swappable (changing from Codex to Claude Code is a config change, not a code change).

---

### T047: Implement `dispatch.rs`: agent selection, worktree setup, multi-agent spawn

**Purpose**: Orchestrate the full dispatch flow: select agent from config, create worktree, inject prompt and context files, spawn 1-3 subagents.

**Steps**:
1. Create `crates/agileplus-agents/src/dispatch.rs`.

2. Implement the core dispatch orchestration:
   ```rust
   pub async fn dispatch_wp(
       vcs: &dyn VcsPort,
       task: AgentTask,
       config: &AgentConfig,
   ) -> Result<AgentResult, DomainError>
   ```

3. Dispatch flow:
   a. **Create worktree**: `vcs.create_worktree(task.feature_slug, task.wp_id)`.
   b. **Copy context files** into worktree: spec.md, plan.md, data-model.md from the feature's kitty-specs directory.
   c. **Write WP prompt** to worktree: copy `task.prompt_path` into the worktree root.
   d. **Select agent harness** based on `config.kind`:
      ```rust
      match config.kind {
          AgentKind::ClaudeCode => claude_code::spawn_claude_code(&task, config).await,
          AgentKind::Codex => codex::spawn_codex(&task, config).await,
      }
      ```
   e. **Collect result**: Parse agent output for PR URL, commits.

4. **Multi-agent dispatch** (1-3 subagents per WP):
   ```rust
   pub async fn dispatch_wp_parallel(
       vcs: &dyn VcsPort,
       task: AgentTask,
       config: &AgentConfig,
       num_agents: usize,
   ) -> Result<Vec<AgentResult>, DomainError>
   ```
   - Spawn `num_agents` instances in parallel via `tokio::join!` or `futures::future::join_all`.
   - Each agent works in the same worktree (they coordinate via git commits).
   - Typically `num_agents = 1` for most WPs; increase for large WPs.

5. **Error handling**: If agent fails, capture stderr, return `AgentResult { success: false, ... }`. Do not panic.

**Files**: `crates/agileplus-agents/src/dispatch.rs`

**Validation**:
- Dispatch creates worktree before spawning agent.
- Context files are present in worktree when agent starts.
- Agent selection works for both ClaudeCode and Codex.
- Multi-agent dispatch spawns correct number of agents.
- Failed agent returns error result, does not crash the orchestrator.

---

### T048: Implement `pr_loop.rs`: PR creation and description building (FR-011)

**Purpose**: Create a GitHub PR after agent work completes, with a structured description containing the WP goal, FR references, and acceptance criteria.

**Steps**:
1. Create `crates/agileplus-agents/src/pr_loop.rs`.

2. **PR creation**:
   ```rust
   pub async fn create_pr(
       worktree_path: &Path,
       wp_title: &str,
       wp_id: &str,
       description: &PrDescription,
       target_branch: &str,
   ) -> Result<String, DomainError>  // returns PR URL
   ```

3. Build PR via `gh` CLI:
   ```rust
   let mut cmd = tokio::process::Command::new("gh");
   cmd.arg("pr").arg("create")
      .arg("--title").arg(format!("{}: {}", wp_id, wp_title))
      .arg("--body").arg(description.to_markdown())
      .arg("--base").arg(target_branch)
      .current_dir(worktree_path);
   ```

4. Define `PrDescription` struct:
   ```rust
   pub struct PrDescription {
       pub wp_id: String,
       pub wp_title: String,
       pub goal: String,              // From WP prompt
       pub fr_references: Vec<String>, // e.g., ["FR-001", "FR-010"]
       pub acceptance_criteria: String, // Markdown checklist
       pub context_summary: String,    // Brief spec/plan summary
   }
   ```

5. Implement `PrDescription::to_markdown()`:
   ```markdown
   ## WP Goal
   {goal}

   ## Functional Requirements
   - {fr_references joined}

   ## Acceptance Criteria
   {acceptance_criteria}

   ## Context
   {context_summary}

   ---
   *Generated by AgilePlus spec-driven development engine*
   ```

6. Parse PR URL from `gh pr create` stdout.

7. **PR status update**: Method to update WP's `pr_url` and `pr_state` after creation.

**Files**: `crates/agileplus-agents/src/pr_loop.rs`

**Validation**:
- PR title follows format: `WP0x: <title>`.
- PR body contains all sections: goal, FR references, acceptance criteria, context.
- `gh pr create` is called with correct arguments.
- PR URL is parsed from output.

---

### T049: Implement review-fix loop (FR-012)

**Purpose**: After PR creation, poll for Coderabbit review comments, feed actionable comments to the agent for fixing, re-push, and re-poll until approval or max cycles reached.

**Steps**:
1. Add to `crates/agileplus-agents/src/pr_loop.rs`.

2. **Review-fix loop**:
   ```rust
   pub async fn run_review_fix_loop(
       pr_url: &str,
       review: &dyn ReviewPort,
       agent_config: &AgentConfig,
       task: &AgentTask,
       max_cycles: u32,
   ) -> Result<ReviewLoopResult, DomainError>
   ```

3. Define `ReviewLoopResult`:
   ```rust
   pub struct ReviewLoopResult {
       pub approved: bool,
       pub cycles_used: u32,
       pub final_review_status: ReviewStatus,
       pub final_ci_status: CiStatus,
       pub comment_history: Vec<Vec<ReviewComment>>,
   }
   ```

4. Loop logic:
   ```
   for cycle in 0..max_cycles:
       1. Wait for review: review.await_review(pr_url, timeout).await
       2. If Approved: return success
       3. If ChangesRequested:
           a. Get actionable comments: review.get_actionable_comments(pr_url).await
           b. Format comments as agent instruction
           c. Spawn agent with fix instruction in worktree
           d. Agent commits and pushes fixes
           e. Continue to next cycle (await new review)
       4. Check CI: review.await_ci(pr_url, timeout).await
           - If Failed: feed CI logs to agent, re-push
       5. If timeout: log warning, continue
   return ReviewLoopResult with cycles_used == max_cycles, approved == false
   ```

5. **Comment formatting for agent**:
   ```rust
   fn format_review_comments_as_instruction(comments: &[ReviewComment]) -> String
   ```
   Produce a markdown instruction like:
   ```markdown
   ## Review Feedback - Fix Required

   ### File: src/lib.rs, Line 42 (Critical)
   > Comment body here

   ### File: src/main.rs, Line 10 (Major)
   > Comment body here

   Please fix all critical and major issues, then commit and push.
   ```

6. **Exponential backoff** for polling: Start at 30s, increase to 60s, 120s, cap at 300s between polls.

7. **Governance exception**: If max cycles exceeded, log a governance exception in the audit trail and return failure. The caller (CLI implement command) decides whether to block or allow manual override.

**Files**: `crates/agileplus-agents/src/pr_loop.rs`

**Validation**:
- Loop terminates after max_cycles.
- Approved review exits loop early with success.
- Actionable comments are formatted and passed to agent.
- Exponential backoff increases poll interval.
- Governance exception logged when max cycles exceeded.
- Test with mock ReviewPort: simulate approve after 2 cycles, verify cycle count.
- Test with mock ReviewPort: simulate max cycles exceeded, verify failure result.

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Agent CLI interface changes | High -- spawn commands break | Abstract behind AgentPort trait. Each harness is a separate module, easily updated. Pin agent CLI versions in docs. |
| Coderabbit review latency | Medium -- slow polling wastes time | Configurable poll interval with exponential backoff. Timeout per cycle. |
| Agent produces no PR | Medium -- dispatch succeeds but no PR created | Check for PR URL in agent output. If missing, attempt `gh pr create` manually from worktree. |
| Multiple agents in same worktree | High -- git conflicts between subagents | Default to 1 agent per worktree. Multi-agent is opt-in and experimental. |
| `gh` CLI not installed | Medium -- PR creation fails | Check for `gh` in PATH at adapter construction. Return clear error message. |
| Agent timeout vs. long-running tasks | Medium -- premature kill | Configurable timeout per WP complexity. Default 30min for small WPs, 2h for large. |
| Review comment parsing | Low -- Coderabbit format changes | Parse conservatively. Treat unparseable comments as informational. Log parsing failures. |

## Review Guidance

1. **Agent isolation**: Verify agents run in worktrees, never in the main repo checkout.
2. **No hardcoded paths**: All paths computed from task/config, not hardcoded.
3. **Error recovery**: Agent failure should not crash the orchestrator. Errors captured and returned.
4. **PR body quality**: PR description must contain WP goal, FR references, and acceptance criteria per FR-011.
5. **Review loop termination**: Verify the loop always terminates (max_cycles or approval).
6. **Timeout handling**: Both agent execution and review polling have configurable timeouts.
7. **Mock testability**: Verify all external calls (agent CLI, gh CLI, ReviewPort) can be mocked in tests.
8. **Secret safety**: No credentials in agent prompts or PR descriptions. Agent CLI handles its own auth.

## Activity Log

| Timestamp | Action | Agent | Details |
|-----------|--------|-------|---------|
| 2026-02-27T00:00:00Z | Prompt generated | system | WP08 prompt created via /spec-kitty.tasks |
