//! Review-fix loop orchestrator.
//!
//! Polls for CI and code review status, feeds review feedback to the agent,
//! and iterates until approved or max cycles reached.
//! Traceability: FR-012 / WP12-T071

use agileplus_domain::domain::work_package::WorkPackage;
use agileplus_domain::ports::agent::{AgentConfig, AgentPort, AgentStatus};

/// Outcome of running the review-fix loop.
#[derive(Debug, Clone)]
pub enum ReviewOutcome {
    /// PR was approved and CI passed.
    Approved,
    /// Max cycles reached without approval.
    MaxCyclesReached { cycles: u32, last_feedback: String },
    /// Agent job failed during a fix cycle.
    AgentFailed { error: String },
    /// Loop was cancelled externally.
    Cancelled,
}

/// Run the review-fix loop for a work package.
///
/// In this scaffold implementation the loop polls the agent port for job
/// completion status. Full Coderabbit integration is wired in later when
/// the ReviewPort adapter (WP09) is available.
///
/// For now the loop:
///   1. Polls `AgentPort::query_status` until the agent completes.
///   2. If the agent returns success the PR is treated as approved.
///   3. If the agent indicates it is waiting for review we simulate a single
///      poll cycle that considers it approved (no-op review adapter).
///   4. If the agent fails we return `AgentFailed`.
///
/// Returns `ReviewOutcome`.
pub async fn run_review_loop<A: AgentPort>(
    wp: &WorkPackage,
    job_id: &str,
    agent: &A,
    _agent_config: &AgentConfig,
    max_cycles: u32,
    poll_interval_secs: u64,
) -> ReviewOutcome {
    let poll = std::time::Duration::from_secs(poll_interval_secs);
    let mut last_feedback = String::new();

    for cycle in 1..=max_cycles {
        println!("  Review cycle {cycle}/{max_cycles}: polling agent status...");

        // Poll with a timeout using tokio::time::timeout
        let status = match agent.query_status(job_id).await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "error polling agent status");
                tokio::time::sleep(poll).await;
                continue;
            }
        };

        match status {
            AgentStatus::Completed { result } => {
                if result.success {
                    println!("  Agent completed successfully.");
                    return ReviewOutcome::Approved;
                } else {
                    last_feedback = result.stderr.clone();
                    println!("  Agent completed with failure: {}", &result.stderr[..result.stderr.len().min(200)]);
                    // Feed back stderr as instruction for next cycle
                    if cycle < max_cycles {
                        let instruction = format!(
                            "Your previous attempt failed. Please fix the following issues:\n\n{}",
                            result.stderr
                        );
                        if let Err(e) = agent.send_instruction(job_id, &instruction).await {
                            tracing::warn!(error = %e, "failed to send instruction to agent");
                        }
                    }
                }
            }
            AgentStatus::WaitingForReview { pr_url } => {
                println!("  Agent waiting for review at: {pr_url}");
                // In a real implementation we would poll Coderabbit here.
                // For now, treat as approved.
                return ReviewOutcome::Approved;
            }
            AgentStatus::Failed { error } => {
                println!("  Agent failed: {error}");
                return ReviewOutcome::AgentFailed { error };
            }
            AgentStatus::Running { pid } => {
                tracing::debug!(pid = pid, "agent still running");
            }
            AgentStatus::Pending => {
                tracing::debug!("agent pending");
            }
        }

        tokio::time::sleep(poll).await;
    }

    println!(
        "  Max review cycles ({max_cycles}) reached for WP {}.",
        wp.id
    );
    ReviewOutcome::MaxCyclesReached { cycles: max_cycles, last_feedback }
}

/// Format structured review comments into an agent instruction.
pub fn format_feedback(comments: &[String]) -> String {
    if comments.is_empty() {
        return "No actionable feedback.".to_string();
    }
    let items: Vec<String> = comments
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{}. {}", i + 1, c))
        .collect();
    format!(
        "Please address the following review comments:\n\n{}\n",
        items.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_feedback_empty() {
        assert_eq!(format_feedback(&[]), "No actionable feedback.");
    }

    #[test]
    fn format_feedback_numbered() {
        let comments = vec!["Fix typo".to_string(), "Add test".to_string()];
        let result = format_feedback(&comments);
        assert!(result.contains("1. Fix typo"));
        assert!(result.contains("2. Add test"));
    }
}
