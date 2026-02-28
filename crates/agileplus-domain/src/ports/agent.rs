//! Agent port — AI agent dispatch and communication abstraction.
//!
//! Traceability: FR-004, FR-010, FR-011, FR-012, FR-013 / WP05-T027

use std::future::Future;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

/// Supported agent backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentKind {
    ClaudeCode,
    Codex,
}

/// Configuration for an agent invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub kind: AgentKind,
    pub max_review_cycles: u32,
    pub timeout_secs: u64,
    pub extra_args: Vec<String>,
}

/// A task to be dispatched to an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub wp_id: String,
    pub feature_slug: String,
    pub prompt_path: PathBuf,
    pub worktree_path: PathBuf,
    pub context_files: Vec<PathBuf>,
}

/// The result returned by an agent after completing (or failing) a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub success: bool,
    pub pr_url: Option<String>,
    pub commits: Vec<String>,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Current status of a dispatched agent job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Pending,
    Running { pid: u32 },
    WaitingForReview { pr_url: String },
    Completed { result: AgentResult },
    Failed { error: String },
}

/// Port for AI agent dispatch and communication.
///
/// Abstracts agent backends so different providers (Claude Code, Codex)
/// are interchangeable. The Agent Dispatch adapter (WP08) implements this.
pub trait AgentPort: Send + Sync {
    /// Spawn an agent and wait for completion.
    fn dispatch(
        &self,
        task: AgentTask,
        config: &AgentConfig,
    ) -> impl Future<Output = Result<AgentResult, DomainError>> + Send;

    /// Spawn an agent without blocking; returns a job ID for later polling.
    fn dispatch_async(
        &self,
        task: AgentTask,
        config: &AgentConfig,
    ) -> impl Future<Output = Result<String, DomainError>> + Send;

    /// Query the current status of a previously dispatched job.
    fn query_status(
        &self,
        job_id: &str,
    ) -> impl Future<Output = Result<AgentStatus, DomainError>> + Send;

    /// Cancel a running or pending job.
    fn cancel(&self, job_id: &str) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Send a follow-up instruction to a running agent.
    fn send_instruction(
        &self,
        job_id: &str,
        instruction: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;
}
