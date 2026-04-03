//! `agileplus-agent-dispatch` — AgentPort adapter.
//!
//! Implements the agent dispatch logic for AgilePlus:
//! - Spawns Claude Code and Codex subprocesses (T045, T046).
//! - Orchestrates worktree creation, context injection, and multi-agent
//!   parallelism (T047).
//! - Creates GitHub PRs via `gh pr create` with structured descriptions
//!   (T048 / FR-011).
//! - Runs the review-fix loop against Coderabbit and GitHub CI (T049 / FR-012).
//! - Exposes `AgentDispatchAdapter` which implements the `AgentPort` trait
//!   (T044b).

pub mod adapter;
pub mod claude_code;
pub mod codex;
pub mod dispatch;
pub mod ports;
pub mod pr_loop;
pub mod types;

// Re-export the most commonly used items at the crate root.
pub use adapter::{AgentDispatchAdapter, AgentPort};
pub use ports::{ReviewPort, VcsPort};
pub use pr_loop::PrDescription;
pub use types::{
    AgentConfig, AgentJob, AgentKind, AgentResult, AgentTask, CiStatus, CommentSeverity,
    DomainError, JobState, ReviewComment, ReviewOutcome,
};
