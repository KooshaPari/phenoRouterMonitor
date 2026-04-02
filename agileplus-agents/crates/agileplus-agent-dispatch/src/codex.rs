//! Codex agent harness (T046).
//!
//! Spawns `codex --quiet --approval-mode=full-auto` as a subprocess.
//! Writes the combined WP prompt to a temporary file inside the worktree and
//! passes it via stdin (Codex reads from stdin in non-interactive mode).

use crate::claude_code::{extract_commits_from_output, extract_pr_url};
use crate::types::{AgentConfig, AgentResult, AgentTask, DomainError};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

// ─── Public entry point ───────────────────────────────────────────────────────

/// Spawn Codex for `task` using `config`, wait for it to finish, and return an
/// [`AgentResult`].
pub async fn spawn_codex(
    task: &AgentTask,
    config: &AgentConfig,
) -> Result<AgentResult, DomainError> {
    let prompt = build_prompt(task).await?;

    let future = async {
        // Write combined prompt to a temp file inside the worktree so Codex
        // can reference it easily.  We also pass via stdin for maximum
        // compatibility with different Codex versions.
        let prompt_file = task.worktree_path.join(".agileplus-codex-prompt.md");
        tokio::fs::write(&prompt_file, &prompt)
            .await
            .map_err(|e| DomainError::ProcessError(format!("prompt file write failed: {e}")))?;

        let mut child = Command::new("codex")
            .arg("--quiet")
            .arg("--approval-mode=full-auto")
            .current_dir(&task.worktree_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| DomainError::ProcessError(format!("failed to spawn codex: {e}")))?;

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

        // Best-effort cleanup of temp prompt file.
        let _ = tokio::fs::remove_file(&prompt_file).await;

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

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn spawn_codex_returns_error_on_timeout() {
        let task = AgentTask {
            job_id: "codex-test".to_owned(),
            feature_slug: "test".to_owned(),
            wp_sequence: 1,
            wp_id: "WP01".to_owned(),
            prompt_path: PathBuf::from("/nonexistent/prompt.md"),
            context_paths: vec![],
            worktree_path: std::env::temp_dir(),
        };
        let config = AgentConfig {
            kind: crate::types::AgentKind::Codex,
            timeout_secs: 0,
            ..Default::default()
        };
        // Either timeout or spawn failure — both are errors, which is correct.
        assert!(spawn_codex(&task, &config).await.is_err());
    }
}
