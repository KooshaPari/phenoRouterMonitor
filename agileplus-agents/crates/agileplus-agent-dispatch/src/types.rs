//! Shared domain types for agent dispatch.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

// ─── Agent Kind ────────────────────────────────────────────────────────────────

/// Which AI agent harness to use when spawning a subprocess.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentKind {
    ClaudeCode,
    Codex,
}

// ─── Agent Task ────────────────────────────────────────────────────────────────

/// Describes the work a single agent must perform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    /// Unique job identifier (UUID v4).
    pub job_id: String,
    /// AgilePlus feature slug (e.g. `001-spec-driven-development-engine`).
    pub feature_slug: String,
    /// Work-package sequence number inside the feature (e.g. `8` for WP08).
    pub wp_sequence: u32,
    /// Human-readable work-package identifier (e.g. `WP08`).
    pub wp_id: String,
    /// Path to the WP prompt markdown file on disk.
    pub prompt_path: PathBuf,
    /// Additional context files (spec.md, plan.md, data-model.md …).
    pub context_paths: Vec<PathBuf>,
    /// Absolute path to the git worktree the agent should operate in.
    pub worktree_path: PathBuf,
}

// ─── Agent Config ──────────────────────────────────────────────────────────────

/// Runtime configuration passed to an agent harness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub kind: AgentKind,
    /// Maximum wall-clock seconds before the agent process is killed.
    pub timeout_secs: u64,
    /// `gh pr create` target branch (usually the WP base branch).
    pub pr_target_branch: String,
    /// How many parallel subagent instances to spawn (1–3).
    pub num_agents: usize,
    /// Maximum review-fix cycles before governance exception.
    pub max_review_cycles: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            kind: AgentKind::ClaudeCode,
            timeout_secs: 1800, // 30 min default
            pr_target_branch: "main".to_owned(),
            num_agents: 1,
            max_review_cycles: 5,
        }
    }
}

// ─── Agent Result ──────────────────────────────────────────────────────────────

/// Output produced by a completed (or failed) agent run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub job_id: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    /// GitHub PR URL if the agent (or the harness fallback) created one.
    pub pr_url: Option<String>,
    /// New commit SHAs observed in the worktree after the agent finished.
    pub commits: Vec<String>,
}

// ─── Agent Job (internal state) ────────────────────────────────────────────────

/// Lifecycle state of an in-flight agent job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Full runtime record stored in the jobs map.
#[derive(Debug)]
pub struct AgentJob {
    pub task: AgentTask,
    pub config: AgentConfig,
    pub state: JobState,
    /// Populated once the job finishes (success or failure).
    pub result: Option<AgentResult>,
}

// ─── Review types ──────────────────────────────────────────────────────────────

/// Severity of a single review comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommentSeverity {
    Critical,
    Major,
    Minor,
    Info,
}

/// A single actionable comment returned by the review adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub file_path: String,
    pub line: Option<u32>,
    pub severity: CommentSeverity,
    pub body: String,
}

/// CI pipeline status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CiStatus {
    Pending,
    Passing,
    Failing,
    Unknown,
}

/// Overall review outcome after one poll.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewOutcome {
    Approved,
    ChangesRequested,
    Pending,
    Dismissed,
}

// ─── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("agent process error: {0}")]
    ProcessError(String),

    #[error("agent timed out after {0}s")]
    Timeout(u64),

    #[error("VCS operation failed: {0}")]
    VcsError(String),

    #[error("PR creation failed: {0}")]
    PrCreationError(String),

    #[error("review loop exhausted: {cycles} cycles with no approval")]
    ReviewLoopExhausted { cycles: u32 },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("job not found: {0}")]
    JobNotFound(String),

    #[error("{0}")]
    Other(String),
}
