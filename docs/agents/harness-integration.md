---
audience: [agents, developers]
---

# Harness Integration & Development

A harness is the adapter layer between AgilePlus and a specific AI agent (Claude Code, Cursor, Codex, etc.). This guide explains harness architecture and how to implement custom harnesses.

## Harness Architecture

```
┌─────────────────────────────────────────────┐
│  AgilePlus Orchestrator                     │
│  (agileplus-cli + agent-dispatch)           │
└────────────────┬────────────────────────────┘
                 │
        ┌────────▼────────┐
        │ Harness Factory │
        └────────┬────────┘
                 │
    ┌────────────┼────────────┐
    │            │            │
┌───▼──┐  ┌─────▼─────┐  ┌───▼──────┐
│Claude │  │  Cursor   │  │Custom    │
│Code   │  │  Harness  │  │Harness   │
│Harness│  │           │  │(you add) │
└───┬──┘  └─────┬─────┘  └───┬──────┘
    │           │            │
    └───────────┬────────────┘
                │
        ┌───────▼────────┐
        │  Agent Process │
        │  (subprocess)  │
        └────────────────┘
```

## Harness Lifecycle & Contract

Every harness must implement the `AgentPort` trait:

```rust
pub trait AgentPort: Send + Sync {
    /// Dispatch an agent synchronously — blocks until completion.
    async fn dispatch(
        &self,
        task: AgentTask,
        config: AgentConfig,
    ) -> Result<AgentResult, DomainError>;

    /// Dispatch an agent in background; returns a job ID for polling.
    async fn dispatch_async(
        &self,
        task: AgentTask,
        config: AgentConfig,
    ) -> Result<String, DomainError>;

    /// Poll the status of a previously dispatched job.
    async fn query_status(&self, job_id: &str) -> Result<JobState, DomainError>;

    /// Cancel a running or pending job.
    async fn cancel(&self, job_id: &str, reason: &str) -> Result<(), DomainError>;

    /// Send an instruction to a running agent (write to its worktree).
    async fn send_instruction(
        &self,
        job_id: &str,
        instruction: &str,
    ) -> Result<(), DomainError>;
}
```

### Task Input

The harness receives:

```rust
pub struct AgentTask {
    pub job_id: String,              // UUID v4
    pub feature_slug: String,        // "001-login"
    pub wp_sequence: u32,            // 1, 2, 3...
    pub wp_id: String,               // "WP01"
    pub prompt_path: PathBuf,        // Path to WP01.md
    pub context_paths: Vec<PathBuf>, // [spec.md, plan.md, ...]
    pub worktree_path: PathBuf,      // .worktrees/001-login-WP01
}

pub struct AgentConfig {
    pub kind: AgentKind,             // ClaudeCode, Codex
    pub timeout_secs: u64,           // 1800 (30 min default)
    pub pr_target_branch: String,    // "feat/001-login-WP01"
    pub num_agents: usize,           // 1-3 parallel agents
    pub max_review_cycles: u32,      // 5 default
}
```

### Result Output

The harness returns:

```rust
pub struct AgentResult {
    pub job_id: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub pr_url: Option<String>,      // GitHub PR URL if created
    pub commits: Vec<String>,        // Commit SHAs
}
```

## Claude Code Harness (Reference Implementation)

The Claude Code harness is built-in and serves as the reference:

```rust
pub struct ClaudeCodeHarness {
    config: ClaudeCodeConfig,
}

impl ClaudeCodeHarness {
    pub fn new(config: ClaudeCodeConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl AgentPort for ClaudeCodeHarness {
    async fn dispatch(
        &self,
        task: AgentTask,
        config: AgentConfig,
    ) -> Result<AgentResult, DomainError> {
        // 1. Set up environment
        let mut cmd = Command::new(&self.config.binary);
        cmd.current_dir(&task.worktree_path);
        cmd.env("AGILEPLUS_JOB_ID", &task.job_id);
        cmd.env("AGILEPLUS_WP_ID", &task.wp_id);
        cmd.env("AGILEPLUS_PROMPT", &task.prompt_path.display().to_string());

        // 2. Add context files
        let context_str = task.context_paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(":");
        cmd.env("AGILEPLUS_CONTEXT", context_str);

        // 3. Set timeout
        let timeout = Duration::from_secs(config.timeout_secs);

        // 4. Launch subprocess
        let mut child = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| DomainError::ProcessError(e.to_string()))?;

        // 5. Wait with timeout
        let output = tokio::time::timeout(timeout, child.wait_with_output())
            .await
            .map_err(|_| DomainError::Timeout(config.timeout_secs))?
            .map_err(|e| DomainError::ProcessError(e.to_string()))?;

        // 6. Parse result
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // 7. Extract PR URL if present
        let pr_url = extract_pr_url_from_stdout(&stdout);

        // 8. Return result
        Ok(AgentResult {
            job_id: task.job_id,
            success: output.status.success(),
            stdout,
            stderr,
            exit_code: output.status.code().unwrap_or(1),
            pr_url,
            commits: extract_commits_from_worktree(&task.worktree_path).await?,
        })
    }

    async fn dispatch_async(
        &self,
        task: AgentTask,
        config: AgentConfig,
    ) -> Result<String, DomainError> {
        let job_id = task.job_id.clone();

        // Spawn background task
        tokio::spawn(async move {
            let _ = self.dispatch(task, config).await;
        });

        Ok(job_id)
    }

    async fn query_status(&self, job_id: &str) -> Result<JobState, DomainError> {
        // Poll job map for status
        // Return Pending, Running, Completed, Failed, Cancelled
        todo!()
    }

    async fn cancel(&self, job_id: &str, reason: &str) -> Result<(), DomainError> {
        // Send SIGTERM to process, wait 30s, then SIGKILL
        todo!()
    }

    async fn send_instruction(
        &self,
        job_id: &str,
        instruction: &str,
    ) -> Result<(), DomainError> {
        // Write instruction to file in worktree
        // Agent polls this file periodically
        todo!()
    }
}
```

## Custom Harness: Cursor Example

Here's how to add a Cursor harness:

### 1. Create Harness Crate

```bash
mkdir -p agileplus-agents/crates/agileplus-agent-cursor
```

### 2. Implement AgentPort

```rust
// agileplus-agents/crates/agileplus-agent-cursor/src/lib.rs

use agileplus_domain::ports::AgentPort;
use agileplus_agent_dispatch::types::{AgentTask, AgentConfig, AgentResult, DomainError};
use std::process::{Command, Stdio};
use tokio::time::timeout;

pub struct CursorHarness {
    binary_path: String,
}

impl CursorHarness {
    pub fn new(binary_path: String) -> Self {
        Self { binary_path }
    }
}

#[async_trait::async_trait]
impl AgentPort for CursorHarness {
    async fn dispatch(
        &self,
        task: AgentTask,
        config: AgentConfig,
    ) -> Result<AgentResult, DomainError> {
        // Cursor-specific implementation
        let mut cmd = Command::new(&self.binary_binary_path);
        cmd.current_dir(&task.worktree_path);

        // Pass prompt via file (Cursor expects file-based input)
        cmd.arg("--prompt-file").arg(&task.prompt_path);

        // Pass context files
        for ctx_path in &task.context_paths {
            cmd.arg("--context").arg(ctx_path);
        }

        // Set working directory
        cmd.arg("--work-dir").arg(&task.worktree_path);

        // Timeout
        let timeout_duration = std::time::Duration::from_secs(config.timeout_secs);

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| DomainError::ProcessError(format!("Failed to spawn cursor: {}", e)))?;

        let output = timeout(timeout_duration, child.wait_with_output())
            .await
            .map_err(|_| DomainError::Timeout(config.timeout_secs))?
            .map_err(|e| DomainError::ProcessError(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(AgentResult {
            job_id: task.job_id,
            success: output.status.success(),
            stdout,
            stderr,
            exit_code: output.status.code().unwrap_or(1),
            pr_url: None,  // TODO: Extract from Cursor's output
            commits: vec![],  // TODO: Extract commits
        })
    }

    async fn dispatch_async(
        &self,
        task: AgentTask,
        config: AgentConfig,
    ) -> Result<String, DomainError> {
        let job_id = task.job_id.clone();
        // Implement background dispatch
        Ok(job_id)
    }

    async fn query_status(&self, job_id: &str) -> Result<JobState, DomainError> {
        // Poll job map
        todo!()
    }

    async fn cancel(&self, job_id: &str, reason: &str) -> Result<(), DomainError> {
        // SIGTERM then SIGKILL
        todo!()
    }

    async fn send_instruction(
        &self,
        job_id: &str,
        instruction: &str,
    ) -> Result<(), DomainError> {
        // Write instruction file
        todo!()
    }
}
```

### 3. Register in Dispatcher

```rust
// agileplus-agents/src/main.rs

use agileplus_agent_cursor::CursorHarness;

fn main() {
    let dispatcher = AgentDispatchAdapter::new(vcs);

    // Register built-in harnesses
    let claude_harness = ClaudeCodeHarness::new(config);
    dispatcher.register("claude-code", claude_harness);

    // Register custom harness
    let cursor_harness = CursorHarness::new("/path/to/cursor".to_string());
    dispatcher.register("cursor", cursor_harness);
}
```

### 4. Configure in .kittify/config.toml

```toml
[agents.cursor]
harness     = "cursor"
binary_path = "/Applications/Cursor.app/Contents/MacOS/Cursor"
timeout_secs = 1800
max_retries = 5
```

## Testing Your Harness

### Unit Tests

```rust
#[tokio::test]
async fn test_cursor_harness_dispatches() {
    let harness = CursorHarness::new("/path/to/cursor".to_string());

    let task = AgentTask {
        job_id: "test-job".to_string(),
        wp_id: "WP01".to_string(),
        // ... other fields
    };

    let config = AgentConfig::default();

    let result = harness.dispatch(task, config).await;
    assert!(result.is_ok());
}
```

### Integration Test

```bash
# Test harness via CLI
agileplus agent test cursor --prompt "Write hello world"
```

This runs a minimal test through your harness and validates:
- Harness starts without error
- Prompt is delivered correctly
- Output is collected successfully
- Agent exits cleanly

### Manual Testing

```bash
# Dispatch a real work package
agileplus implement 001-feature --agent cursor --timeout 600

# Monitor job
agileplus agent status job-uuid-here
```

## Environment Variables for Harnesses

Harnesses can read configuration via environment:

```bash
# For custom harness
export AGILEPLUS_CUSTOM_HARNESS_BINARY="/path/to/agent"
export AGILEPLUS_CUSTOM_HARNESS_TIMEOUT="3600"

agileplus implement 001-feature --agent custom
```

## Output Format Expectations

All harnesses must produce consistent output. The orchestrator expects:

### Success

```json
{
  "job_id": "uuid-here",
  "wp_id": "WP01",
  "success": true,
  "status": "completed",
  "summary": "Implemented component with tests",
  "commits": [
    { "sha": "abc123...", "message": "WP01: Implement component" }
  ],
  "pr_url": "https://github.com/org/repo/pull/42"
}
```

### Failure

```json
{
  "job_id": "uuid-here",
  "wp_id": "WP01",
  "success": false,
  "status": "failed",
  "error": "Tests failed: 3 failures",
  "exit_code": 1
}
```

## Harness Compliance Checklist

- [ ] Implements all methods in `AgentPort` trait
- [ ] Properly handles timeouts (no hanging processes)
- [ ] Extracts commits from worktree git history
- [ ] Logs all commands and output for debugging
- [ ] Handles file permissions correctly (readable prompt/context)
- [ ] Cleans up resources (kill process on cancel)
- [ ] Returns consistent result format
- [ ] Tests pass: `cargo test -p agileplus-agent-cursor`
- [ ] Integration tests pass: `agileplus agent test cursor ...`

## Common Pitfalls

1. **Forgetting to set working directory:** Agent can't find files
   ```rust
   // ✗ WRONG: Runs in current directory
   cmd.spawn()

   // ✓ RIGHT: Runs in worktree
   cmd.current_dir(&task.worktree_path);
   ```

2. **Not handling timeouts:** Process hangs indefinitely
   ```rust
   // ✗ WRONG: No timeout
   let output = child.wait_with_output().await?;

   // ✓ RIGHT: With timeout
   let output = timeout(Duration::from_secs(1800), child.wait_with_output()).await??;
   ```

3. **Not extracting commits:** Orchestrator can't track work
   ```rust
   // ✗ WRONG: Returns empty commit list
   commits: vec![],

   // ✓ RIGHT: Extract from git history
   commits: extract_commits_from_worktree(&task.worktree_path).await?,
   ```

4. **Hardcoding paths:** Breaks in CI/Docker
   ```rust
   // ✗ WRONG: Hardcoded path
   let binary = "/usr/local/bin/claude";

   // ✓ RIGHT: Configurable or from PATH
   let binary = env::var("CLAUDE_CODE_PATH").unwrap_or("claude".to_string());
   ```

## Deployment Checklist

Before releasing a harness:

1. **Documentation:** README with setup instructions
2. **Examples:** Sample config.toml and command usage
3. **Tests:** Unit + integration tests passing
4. **Performance:** Baseline latency measured
5. **Error handling:** All error paths tested
6. **Security:** No secrets logged or stored
7. **Versioning:** Semver in Cargo.toml
