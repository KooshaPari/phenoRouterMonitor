//! Stub AgentPort implementation for use when no real agent adapter is configured.
//!
//! Prints instructions for manual execution and returns a no-op job ID.
//! Replace with agileplus-agents adapter (WP08) when available.

use agileplus_domain::error::DomainError;
use agileplus_domain::ports::agent::{AgentConfig, AgentPort, AgentResult, AgentStatus, AgentTask};

/// A no-op agent adapter that prints instructions and simulates completion.
pub struct StubAgentAdapter;

impl AgentPort for StubAgentAdapter {
    async fn dispatch(&self, task: AgentTask, _config: &AgentConfig) -> Result<AgentResult, DomainError> {
        println!(
            "  [stub-agent] Would dispatch agent for WP '{}' in worktree '{}'.",
            task.wp_id,
            task.worktree_path.display()
        );
        println!("  [stub-agent] Prompt: {}", task.prompt_path.display());
        Ok(AgentResult {
            success: true,
            pr_url: None,
            commits: vec![],
            stdout: "stub: no agent configured".to_string(),
            stderr: String::new(),
            exit_code: 0,
        })
    }

    async fn dispatch_async(&self, task: AgentTask, _config: &AgentConfig) -> Result<String, DomainError> {
        println!(
            "  [stub-agent] Would dispatch agent for WP '{}' in worktree '{}'.",
            task.wp_id,
            task.worktree_path.display()
        );
        println!(
            "  To implement manually, open the worktree and run the agent with the prompt at:\n  {}",
            task.prompt_path.display()
        );
        // Return a synthetic job ID
        Ok(format!("stub-{}", task.wp_id))
    }

    async fn query_status(&self, job_id: &str) -> Result<AgentStatus, DomainError> {
        // Stub: immediately report success so the review loop exits
        Ok(AgentStatus::Completed {
            result: AgentResult {
                success: true,
                pr_url: None,
                commits: vec![],
                stdout: format!("stub: {job_id} completed"),
                stderr: String::new(),
                exit_code: 0,
            },
        })
    }

    async fn cancel(&self, _job_id: &str) -> Result<(), DomainError> {
        Ok(())
    }

    async fn send_instruction(&self, _job_id: &str, _instruction: &str) -> Result<(), DomainError> {
        Ok(())
    }
}
