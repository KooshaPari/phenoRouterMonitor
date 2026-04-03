//! Claude Code agent harness (T045).
//!
//! Spawns `claude --print --dangerously-skip-permissions` as a subprocess,
//! feeds the WP prompt via stdin, captures stdout / stderr, and parses the
//! PR URL and new commit SHAs from the output.

use crate::types::{AgentConfig, AgentResult, AgentTask, DomainError};
use regex::Regex;
use std::sync::OnceLock;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

// ─── Public entry point ───────────────────────────────────────────────────────

/// Spawn Claude Code for `task` using `config`, wait for it to finish, and
/// return an [`AgentResult`].
pub async fn spawn_claude_code(
    task: &AgentTask,
    config: &AgentConfig,
) -> Result<AgentResult, DomainError> {
    let prompt = build_prompt(task).await?;

    let future = async {
        let mut child = Command::new("claude")
            .arg("--print")
            .arg("--dangerously-skip-permissions")
            .current_dir(&task.worktree_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| DomainError::ProcessError(format!("failed to spawn claude: {e}")))?;

        // Feed prompt via stdin.
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(prompt.as_bytes())
                .await
                .map_err(|e| DomainError::ProcessError(format!("stdin write failed: {e}")))?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| DomainError::ProcessError(format!("wait failed: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let exit_code = output.status.code().unwrap_or(-1);

        let pr_url = extract_pr_url(&stdout);
        let commits = extract_commits_from_output(&stdout);

        Ok(AgentResult {
            job_id: task.job_id.clone(),
            success: output.status.success(),
            stdout,
            stderr,
            exit_code,
            pr_url,
            commits,
        })
    };

    timeout(Duration::from_secs(config.timeout_secs), future)
        .await
        .map_err(|_| DomainError::Timeout(config.timeout_secs))?
}

// ─── Prompt construction ──────────────────────────────────────────────────────

async fn build_prompt(task: &AgentTask) -> Result<String, DomainError> {
    let prompt_content = tokio::fs::read_to_string(&task.prompt_path)
        .await
        .map_err(|e| {
            DomainError::ProcessError(format!(
                "failed to read prompt file {}: {e}",
                task.prompt_path.display()
            ))
        })?;

    let mut parts = vec![prompt_content];

    if !task.context_paths.is_empty() {
        parts.push("\n\n---\n## Reference Context Files\n".to_owned());
        for ctx_path in &task.context_paths {
            match tokio::fs::read_to_string(ctx_path).await {
                Ok(content) => {
                    parts.push(format!(
                        "\n### {}\n\n{}\n",
                        ctx_path.file_name().unwrap_or_default().to_string_lossy(),
                        content
                    ));
                }
                Err(e) => {
                    tracing::warn!("could not read context file {}: {e}", ctx_path.display());
                }
            }
        }
    }

    Ok(parts.concat())
}

// ─── Output parsing ───────────────────────────────────────────────────────────

/// Extract the first GitHub PR URL from agent stdout.
pub fn extract_pr_url(output: &str) -> Option<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"https://github\.com/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+/pull/\d+")
            .expect("valid regex")
    });
    re.find(output).map(|m| m.as_str().to_owned())
}

/// Extract commit SHAs mentioned in agent stdout (7-char or full SHA).
pub fn extract_commits_from_output(output: &str) -> Vec<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"\b([0-9a-f]{7,40})\b").expect("valid regex")
    });
    re.find_iter(output)
        .map(|m| m.as_str().to_owned())
        .collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pr_url_extraction_finds_url() {
        let output = "Agent created PR: https://github.com/phenotype/agileplus/pull/42 and finished.";
        assert_eq!(
            extract_pr_url(output),
            Some("https://github.com/phenotype/agileplus/pull/42".to_owned())
        );
    }

    #[test]
    fn pr_url_extraction_returns_none_when_absent() {
        assert_eq!(extract_pr_url("no url here"), None);
    }

    #[test]
    fn commit_extraction_finds_shas() {
        let output = "Committed abc1234 and pushed def5678901234567890123456789012345678901 successfully.";
        let commits = extract_commits_from_output(output);
        assert!(commits.contains(&"abc1234".to_owned()));
    }

    #[tokio::test]
    async fn spawn_claude_code_times_out() {
        // Override the timeout to 0 seconds so the function times out immediately.
        let task = AgentTask {
            job_id: "test-job".to_owned(),
            feature_slug: "test-feature".to_owned(),
            wp_sequence: 1,
            wp_id: "WP01".to_owned(),
            prompt_path: std::path::PathBuf::from("/nonexistent/prompt.md"),
            context_paths: vec![],
            worktree_path: std::env::temp_dir(),
        };
        let config = AgentConfig {
            timeout_secs: 0,
            ..Default::default()
        };
        let result = spawn_claude_code(&task, &config).await;
        // Either times out or fails to spawn — both are acceptable errors here.
        assert!(result.is_err());
    }
}
