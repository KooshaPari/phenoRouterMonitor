//! `AgentDispatchAdapter` — implements `AgentPort` (T044b).
//!
//! Manages in-flight agent jobs via a concurrent `DashMap`, supports
//! synchronous and async dispatch, status queries, cancellation, and
//! instruction delivery.

use crate::dispatch::dispatch_wp;
use crate::ports::VcsPort;
use crate::types::{AgentConfig, AgentJob, AgentResult, AgentTask, DomainError, JobState};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{info, warn};
use uuid::Uuid;

// ─── AgentPort trait ──────────────────────────────────────────────────────────

/// Core operations the orchestrator (or gRPC service) calls to manage agents.
#[async_trait::async_trait]
pub trait AgentPort: Send + Sync {
    /// Dispatch an agent synchronously — blocks until it finishes.
    async fn dispatch(
        &self,
        task: AgentTask,
        config: AgentConfig,
    ) -> Result<AgentResult, DomainError>;

    /// Dispatch an agent in the background; return a unique job ID immediately.
    async fn dispatch_async(
        &self,
        task: AgentTask,
        config: AgentConfig,
    ) -> Result<String, DomainError>;

    /// Return the current [`JobState`] for `job_id`.
    async fn query_status(&self, job_id: &str) -> Result<JobState, DomainError>;

    /// Kill the agent process and mark the job as failed.
    async fn cancel(&self, job_id: &str, reason: &str) -> Result<(), DomainError>;

    /// Send an ad-hoc instruction to a running agent by writing a file into
    /// its worktree.  Returns an error if the job is not running.
    async fn send_instruction(&self, job_id: &str, instruction: &str)
        -> Result<(), DomainError>;
}

// ─── Adapter implementation ───────────────────────────────────────────────────

/// Concurrent job registry + VCS adapter.
///
/// Wrapping in `Arc<AgentDispatchAdapter>` makes it `Send + Sync` so it can be
/// shared across tokio tasks and the gRPC service layer.
pub struct AgentDispatchAdapter {
    vcs: Arc<dyn VcsPort + Send + Sync>,
    jobs: Arc<DashMap<String, AgentJob>>,
    handles: Arc<DashMap<String, JoinHandle<()>>>,
}

impl AgentDispatchAdapter {
    /// Construct a new adapter backed by `vcs`.
    pub fn new(vcs: Arc<dyn VcsPort + Send + Sync>) -> Self {
        Self {
            vcs,
            jobs: Arc::new(DashMap::new()),
            handles: Arc::new(DashMap::new()),
        }
    }

    /// Generate a globally-unique job ID.
    fn new_job_id() -> String {
        Uuid::new_v4().to_string()
    }
}

#[async_trait::async_trait]
impl AgentPort for AgentDispatchAdapter {
    async fn dispatch(
        &self,
        mut task: AgentTask,
        config: AgentConfig,
    ) -> Result<AgentResult, DomainError> {
        let job_id = Self::new_job_id();
        task.job_id = job_id.clone();

        // Register job as running.
        self.jobs.insert(
            job_id.clone(),
            AgentJob {
                task: task.clone(),
                config: config.clone(),
                state: JobState::Running,
                result: None,
            },
        );

        let result = dispatch_wp(self.vcs.as_ref(), task, &config).await;

        // Update final state.
        if let Some(mut job) = self.jobs.get_mut(&job_id) {
            match &result {
                Ok(r) => {
                    job.state = JobState::Completed;
                    job.result = Some(r.clone());
                }
                Err(_) => {
                    job.state = JobState::Failed;
                }
            }
        }

        result
    }

    async fn dispatch_async(
        &self,
        mut task: AgentTask,
        config: AgentConfig,
    ) -> Result<String, DomainError> {
        let job_id = Self::new_job_id();
        task.job_id = job_id.clone();

        // Register as pending before spawning.
        self.jobs.insert(
            job_id.clone(),
            AgentJob {
                task: task.clone(),
                config: config.clone(),
                state: JobState::Pending,
                result: None,
            },
        );

        let vcs = Arc::clone(&self.vcs);
        let jobs = Arc::clone(&self.jobs);
        let handles = Arc::clone(&self.handles);
        let id = job_id.clone();

        let handle = tokio::spawn(async move {
            // Transition to running.
            if let Some(mut j) = jobs.get_mut(&id) {
                j.state = JobState::Running;
            }

            let result = dispatch_wp(vcs.as_ref(), task, &config).await;

            // Transition to terminal state.
            if let Some(mut j) = jobs.get_mut(&id) {
                match result {
                    Ok(r) => {
                        j.state = JobState::Completed;
                        j.result = Some(r);
                    }
                    Err(e) => {
                        warn!(job_id = %id, error = %e, "agent job failed");
                        j.state = JobState::Failed;
                    }
                }
            }

            handles.remove(&id);
        });

        self.handles.insert(job_id.clone(), handle);
        info!(%job_id, "agent dispatched asynchronously");
        Ok(job_id)
    }

    async fn query_status(&self, job_id: &str) -> Result<JobState, DomainError> {
        self.jobs
            .get(job_id)
            .map(|j| j.state.clone())
            .ok_or_else(|| DomainError::JobNotFound(job_id.to_owned()))
    }

    async fn cancel(&self, job_id: &str, reason: &str) -> Result<(), DomainError> {
        // Abort the background task if still running.
        if let Some((_, handle)) = self.handles.remove(job_id) {
            handle.abort();
        }

        // Mark as failed in the registry.
        match self.jobs.get_mut(job_id) {
            Some(mut job) => {
                job.state = JobState::Cancelled;
                info!(%job_id, %reason, "agent job cancelled");
                Ok(())
            }
            None => Err(DomainError::JobNotFound(job_id.to_owned())),
        }
    }

    async fn send_instruction(
        &self,
        job_id: &str,
        instruction: &str,
    ) -> Result<(), DomainError> {
        let worktree_path = self
            .jobs
            .get(job_id)
            .map(|j| j.task.worktree_path.clone())
            .ok_or_else(|| DomainError::JobNotFound(job_id.to_owned()))?;

        if worktree_path.as_os_str().is_empty() {
            return Err(DomainError::Other(
                "worktree path not yet set for this job".to_owned(),
            ));
        }

        let instruction_file = worktree_path.join(".agileplus-instruction.md");
        tokio::fs::write(&instruction_file, instruction)
            .await
            .map_err(|e| {
                DomainError::Other(format!("failed to write instruction file: {e}"))
            })?;

        info!(%job_id, path = %instruction_file.display(), "instruction written");
        Ok(())
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
    use tempfile::tempdir;

    struct FakeVcs(PathBuf);

    #[async_trait]
    impl VcsPort for FakeVcs {
        async fn create_worktree(&self, _f: &str, _w: &str) -> Result<PathBuf, DomainError> {
            Ok(self.0.clone())
        }
        async fn remove_worktree(&self, _p: &PathBuf) -> Result<(), DomainError> {
            Ok(())
        }
        async fn new_commits_since(
            &self,
            _p: &PathBuf,
            _s: &str,
        ) -> Result<Vec<String>, DomainError> {
            Ok(vec![])
        }
    }

    fn make_task(worktree: PathBuf) -> AgentTask {
        let prompt = worktree.join("prompt.md");
        std::fs::write(&prompt, "# Prompt").unwrap();
        AgentTask {
            job_id: String::new(),
            feature_slug: "feat".to_owned(),
            wp_sequence: 8,
            wp_id: "WP08".to_owned(),
            prompt_path: prompt,
            context_paths: vec![],
            worktree_path: worktree,
        }
    }

    #[tokio::test]
    async fn dispatch_async_returns_unique_job_ids() {
        let tmp1 = tempdir().unwrap();
        let tmp2 = tempdir().unwrap();

        let adapter = Arc::new(AgentDispatchAdapter::new(Arc::new(FakeVcs(
            tmp1.path().to_path_buf(),
        ))));

        let t1 = make_task(tmp1.path().to_path_buf());
        let t2 = make_task(tmp2.path().to_path_buf());

        let id1 = adapter
            .dispatch_async(t1, AgentConfig { kind: AgentKind::ClaudeCode, timeout_secs: 1, ..Default::default() })
            .await
            .unwrap();
        let id2 = adapter
            .dispatch_async(t2, AgentConfig { kind: AgentKind::ClaudeCode, timeout_secs: 1, ..Default::default() })
            .await
            .unwrap();

        assert_ne!(id1, id2, "job IDs must be unique");
    }

    #[tokio::test]
    async fn query_status_returns_not_found_for_unknown_job() {
        let tmp = tempdir().unwrap();
        let adapter = AgentDispatchAdapter::new(Arc::new(FakeVcs(tmp.path().to_path_buf())));
        let err = adapter.query_status("nonexistent").await.unwrap_err();
        matches!(err, DomainError::JobNotFound(_));
    }

    #[tokio::test]
    async fn cancel_marks_job_as_cancelled() {
        let tmp = tempdir().unwrap();
        let adapter = Arc::new(AgentDispatchAdapter::new(Arc::new(FakeVcs(
            tmp.path().to_path_buf(),
        ))));

        let t = make_task(tmp.path().to_path_buf());
        let id = adapter
            .dispatch_async(t, AgentConfig { kind: AgentKind::ClaudeCode, timeout_secs: 60, ..Default::default() })
            .await
            .unwrap();

        // Give the task a moment to register itself.
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        adapter.cancel(&id, "test cancel").await.unwrap();
        let state = adapter.query_status(&id).await.unwrap();
        assert_eq!(state, JobState::Cancelled);
    }

    #[tokio::test]
    async fn send_instruction_writes_file() {
        let tmp = tempdir().unwrap();
        let adapter = Arc::new(AgentDispatchAdapter::new(Arc::new(FakeVcs(
            tmp.path().to_path_buf(),
        ))));

        let mut t = make_task(tmp.path().to_path_buf());
        t.job_id = "test-send".to_owned();
        // Pre-insert with worktree set.
        adapter.jobs.insert(
            "test-send".to_owned(),
            AgentJob {
                task: t.clone(),
                config: AgentConfig::default(),
                state: JobState::Running,
                result: None,
            },
        );

        adapter
            .send_instruction("test-send", "## Fix\n\nPlease fix it.")
            .await
            .unwrap();

        let written = tokio::fs::read_to_string(
            tmp.path().join(".agileplus-instruction.md"),
        )
        .await
        .unwrap();
        assert!(written.contains("Please fix it."));
    }
}
