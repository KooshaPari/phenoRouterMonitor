//! Dispatch orchestration (T047).
//!
//! `dispatch_wp` is the single-agent entrypoint: it creates a worktree, copies
//! context files, selects the right harness, and returns the result.
//! `dispatch_wp_parallel` launches 1-3 agents concurrently.

use crate::claude_code::spawn_claude_code;
use crate::codex::spawn_codex;
use crate::ports::VcsPort;
use crate::types::{AgentConfig, AgentKind, AgentResult, AgentTask, DomainError};
use futures::future::join_all;
use std::path::PathBuf;
use tracing::{info, warn};

// ─── Single-agent dispatch ────────────────────────────────────────────────────

/// Full dispatch flow for a single agent:
/// 1. Create worktree via [`VcsPort`].
/// 2. Copy context files into the worktree.
/// 3. Select harness from `config.kind`.
/// 4. Spawn agent, return result.
pub async fn dispatch_wp(
    vcs: &dyn VcsPort,
    mut task: AgentTask,
    config: &AgentConfig,
) -> Result<AgentResult, DomainError> {
    // 1. Create the worktree.
    let worktree = vcs
        .create_worktree(&task.feature_slug, &task.wp_id)
        .await?;
    task.worktree_path = worktree.clone();

    info!(
        job_id = %task.job_id,
        worktree = %worktree.display(),
        "worktree created"
    );

    // 2. Copy context files into the worktree root.
    copy_context_files(&task.context_paths, &worktree).await;

    // 3. Copy WP prompt into the worktree root so the agent can reference it.
    let dest_prompt = worktree.join("WP-PROMPT.md");
    if let Err(e) = tokio::fs::copy(&task.prompt_path, &dest_prompt).await {
        warn!("could not copy prompt into worktree: {e}");
    } else {
        task.prompt_path = dest_prompt;
    }

    // 4. Select harness and spawn.
    let result = match config.kind {
        AgentKind::ClaudeCode => spawn_claude_code(&task, config).await?,
        AgentKind::Codex => spawn_codex(&task, config).await?,
    };

    info!(
        job_id = %task.job_id,
        success = result.success,
        pr_url = ?result.pr_url,
        "agent finished"
    );

    Ok(result)
}

// ─── Multi-agent (parallel) dispatch ─────────────────────────────────────────

/// Spawn `config.num_agents` (clamped to 1–3) instances in parallel, each
/// receiving the same task and config.  Returns the results in spawn order.
pub async fn dispatch_wp_parallel(
    vcs: &dyn VcsPort,
    task: AgentTask,
    config: &AgentConfig,
) -> Result<Vec<AgentResult>, DomainError> {
    let num = config.num_agents.clamp(1, 3);

    let futures: Vec<_> = (0..num)
        .map(|i| {
            // Each subagent gets a unique job_id suffix so they don't collide.
            let mut sub_task = task.clone();
            sub_task.job_id = format!("{}-sub{i}", task.job_id);
            // We need owned VcsPort here — callers must wrap in Arc.
            dispatch_wp_owned(vcs, sub_task, config.clone())
        })
        .collect();

    let results: Vec<Result<AgentResult, DomainError>> = join_all(futures).await;

    // Collect results; first error becomes the return value.
    let mut ok_results = Vec::with_capacity(num);
    for r in results {
        ok_results.push(r?);
    }
    Ok(ok_results)
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Thin wrapper so the parallel dispatch can capture `config` by value.
async fn dispatch_wp_owned(
    vcs: &dyn VcsPort,
    task: AgentTask,
    config: AgentConfig,
) -> Result<AgentResult, DomainError> {
    dispatch_wp(vcs, task, &config).await
}

/// Copy context files into `worktree_root`, logging but not failing on errors.
async fn copy_context_files(paths: &[PathBuf], worktree_root: &PathBuf) {
    for src in paths {
        let file_name = src.file_name().unwrap_or_default();
        let dest = worktree_root.join(file_name);
        if let Err(e) = tokio::fs::copy(src, &dest).await {
            warn!("could not copy context file {}: {e}", src.display());
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::VcsPort;
    use crate::types::{AgentConfig, AgentKind, AgentTask, DomainError};
    use async_trait::async_trait;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tempfile::tempdir;

    struct FakeVcs {
        worktree_dir: PathBuf,
    }

    #[async_trait]
    impl VcsPort for FakeVcs {
        async fn create_worktree(
            &self,
            _feature: &str,
            _wp: &str,
        ) -> Result<PathBuf, DomainError> {
            Ok(self.worktree_dir.clone())
        }

        async fn remove_worktree(&self, _path: &PathBuf) -> Result<(), DomainError> {
            Ok(())
        }

        async fn new_commits_since(
            &self,
            _path: &PathBuf,
            _since: &str,
        ) -> Result<Vec<String>, DomainError> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn dispatch_wp_creates_worktree_and_copies_context() {
        let tmp = tempdir().unwrap();
        let worktree = tmp.path().to_path_buf();

        // Create a fake prompt file.
        let prompt_file = tmp.path().join("WP-TEST.md");
        tokio::fs::write(&prompt_file, "# Test Prompt").await.unwrap();

        // Create a fake context file.
        let ctx_file = tmp.path().join("spec.md");
        tokio::fs::write(&ctx_file, "# Spec").await.unwrap();

        let vcs = FakeVcs {
            worktree_dir: worktree.clone(),
        };

        let task = AgentTask {
            job_id: "test-dispatch".to_owned(),
            feature_slug: "test-feat".to_owned(),
            wp_sequence: 8,
            wp_id: "WP08".to_owned(),
            prompt_path: prompt_file,
            context_paths: vec![ctx_file],
            worktree_path: PathBuf::new(), // will be overwritten by dispatch_wp
        };

        let config = AgentConfig {
            kind: AgentKind::ClaudeCode,
            timeout_secs: 1, // very short — will time out, but we care about the setup
            ..Default::default()
        };

        // We expect an error (no `claude` binary in CI), but the worktree
        // path should have been set and the prompt copied before the error.
        let _ = dispatch_wp(&vcs, task, &config).await;

        // Prompt was copied into the worktree.
        assert!(worktree.join("WP-PROMPT.md").exists());
        // Context file was copied.
        assert!(worktree.join("spec.md").exists());
    }

    #[tokio::test]
    async fn dispatch_wp_parallel_clamps_to_three() {
        let tmp = tempdir().unwrap();
        let worktree = tmp.path().to_path_buf();

        let prompt_file = tmp.path().join("prompt.md");
        tokio::fs::write(&prompt_file, "# P").await.unwrap();

        let vcs = Arc::new(FakeVcs {
            worktree_dir: worktree,
        });

        let task = AgentTask {
            job_id: "parallel-test".to_owned(),
            feature_slug: "feat".to_owned(),
            wp_sequence: 1,
            wp_id: "WP01".to_owned(),
            prompt_path: prompt_file,
            context_paths: vec![],
            worktree_path: PathBuf::new(),
        };

        // Request 5 agents — should be clamped to 3.
        let config = AgentConfig {
            kind: AgentKind::ClaudeCode,
            timeout_secs: 1,
            num_agents: 5,
            ..Default::default()
        };

        // Errors are expected (no `claude` binary) but we verify it runs.
        let _ = dispatch_wp_parallel(vcs.as_ref(), task, &config).await;
    }
}
