//! AgilePlus git adapter — worktree, repository, and artifact operations.
//!
//! Implements `VcsPort` using git2 (libgit2 bindings). No CLI shelling.
//! Traceability: WP07

pub mod artifact;
pub mod repository;
pub mod worktree;

use std::path::{Path, PathBuf};

use agileplus_domain::{
    error::DomainError,
    ports::{
        ConflictInfo, FeatureArtifacts, MergeResult, VcsPort, WorktreeInfo,
    },
};
use git2::Repository;

/// Map a git2 error to a DomainError.
pub(crate) fn git_err(e: git2::Error) -> DomainError {
    match e.code() {
        git2::ErrorCode::NotFound => DomainError::NotFound(e.message().to_string()),
        git2::ErrorCode::Exists => DomainError::Conflict(e.message().to_string()),
        _ => DomainError::Vcs(e.message().to_string()),
    }
}

/// Git VCS adapter. Implements `VcsPort` using git2.
///
/// Repository is NOT stored — opened fresh per operation (git2::Repository is not Send).
pub struct GitVcsAdapter {
    repo_path: PathBuf,
}

impl GitVcsAdapter {
    /// Create adapter pointing at an existing git repository.
    pub fn new(repo_path: PathBuf) -> Result<Self, DomainError> {
        // Validate it's a real git repo.
        Repository::open(&repo_path).map_err(git_err)?;
        Ok(Self { repo_path })
    }

    /// Discover repository from the current working directory.
    pub fn from_current_dir() -> Result<Self, DomainError> {
        let repo = Repository::discover(".").map_err(git_err)?;
        let path = repo
            .workdir()
            .ok_or_else(|| DomainError::Vcs("bare repository not supported".into()))?
            .to_path_buf();
        Ok(Self { repo_path: path })
    }

    /// Open a fresh Repository handle (git2::Repository is not Send).
    pub(crate) fn open_repo(&self) -> Result<Repository, DomainError> {
        Repository::open(&self.repo_path).map_err(git_err)
    }

    /// Return the repo root path.
    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }
}

// Safety: We never store Repository. Only PathBuf which is Send+Sync.
unsafe impl Send for GitVcsAdapter {}
unsafe impl Sync for GitVcsAdapter {}

impl VcsPort for GitVcsAdapter {
    // ---- Worktree operations ----

    fn create_worktree(
        &self,
        feature_slug: &str,
        wp_id: &str,
    ) -> impl std::future::Future<Output = Result<PathBuf, DomainError>> + Send {
        let result = worktree::create_worktree(self, feature_slug, wp_id);
        async move { result }
    }

    fn list_worktrees(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<WorktreeInfo>, DomainError>> + Send {
        let result = worktree::list_worktrees(self);
        async move { result }
    }

    fn cleanup_worktree(
        &self,
        worktree_path: &Path,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        let result = worktree::cleanup_worktree(self, worktree_path);
        async move { result }
    }

    // ---- Branch operations ----

    fn create_branch(
        &self,
        branch_name: &str,
        base: &str,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        let result = repository::create_branch(self, branch_name, base);
        async move { result }
    }

    fn checkout_branch(
        &self,
        branch_name: &str,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        let result = repository::checkout_branch(self, branch_name);
        async move { result }
    }

    fn merge_to_target(
        &self,
        source: &str,
        target: &str,
    ) -> impl std::future::Future<Output = Result<MergeResult, DomainError>> + Send {
        let result = repository::merge_to_target(self, source, target);
        async move { result }
    }

    fn detect_conflicts(
        &self,
        source: &str,
        target: &str,
    ) -> impl std::future::Future<Output = Result<Vec<ConflictInfo>, DomainError>> + Send {
        let result = repository::detect_conflicts(self, source, target);
        async move { result }
    }

    // ---- Artifact operations ----

    fn read_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> impl std::future::Future<Output = Result<String, DomainError>> + Send {
        let result = artifact::read_artifact(self, feature_slug, relative_path);
        async move { result }
    }

    fn write_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
        content: &str,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send {
        let result = artifact::write_artifact(self, feature_slug, relative_path, content);
        async move { result }
    }

    fn artifact_exists(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> impl std::future::Future<Output = Result<bool, DomainError>> + Send {
        let result = artifact::artifact_exists(self, feature_slug, relative_path);
        async move { result }
    }

    fn scan_feature_artifacts(
        &self,
        feature_slug: &str,
    ) -> impl std::future::Future<Output = Result<FeatureArtifacts, DomainError>> + Send {
        let result = artifact::scan_feature_artifacts(self, feature_slug);
        async move { result }
    }
}

/// Scan all feature slugs found under kitty-specs/ that contain meta.json.
pub fn scan_all_features(adapter: &GitVcsAdapter) -> Result<Vec<String>, DomainError> {
    let base = adapter.repo_path().join("kitty-specs");
    if !base.exists() {
        return Ok(vec![]);
    }
    let mut slugs = Vec::new();
    for entry in std::fs::read_dir(&base).map_err(|e| DomainError::Vcs(e.to_string()))? {
        let entry = entry.map_err(|e| DomainError::Vcs(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() && path.join("meta.json").exists() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                slugs.push(name.to_string());
            }
        }
    }
    Ok(slugs)
}

/// Commit info for history scanning.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub oid: String,
    pub message: String,
    pub author: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Get commit history touching kitty-specs/<feature_slug>/.
pub fn get_feature_history(
    adapter: &GitVcsAdapter,
    feature_slug: &str,
) -> Result<Vec<CommitInfo>, DomainError> {
    let repo = adapter.open_repo()?;
    let mut revwalk = repo.revwalk().map_err(git_err)?;
    revwalk.push_head().map_err(git_err)?;
    revwalk.set_sorting(git2::Sort::TIME).map_err(git_err)?;

    let filter_prefix = format!("kitty-specs/{feature_slug}/");
    let mut results = Vec::new();

    for oid in revwalk {
        let oid = oid.map_err(git_err)?;
        let commit = repo.find_commit(oid).map_err(git_err)?;

        // Check if this commit touches the feature path.
        let touches = if commit.parent_count() == 0 {
            // Root commit — check if tree has the path.
            let tree = commit.tree().map_err(git_err)?;
            tree.get_path(Path::new(&filter_prefix)).is_ok()
        } else {
            let parent = commit.parent(0).map_err(git_err)?;
            let diff = repo
                .diff_tree_to_tree(
                    Some(&parent.tree().map_err(git_err)?),
                    Some(&commit.tree().map_err(git_err)?),
                    None,
                )
                .map_err(git_err)?;
            let mut found = false;
            diff.foreach(
                &mut |delta, _| {
                    let path = delta
                        .new_file()
                        .path()
                        .and_then(|p| p.to_str())
                        .unwrap_or("");
                    if path.starts_with(&filter_prefix) {
                        found = true;
                    }
                    true
                },
                None,
                None,
                None,
            )
            .map_err(git_err)?;
            found
        };

        if touches {
            use chrono::TimeZone;
            let ts = commit.time();
            let dt = chrono::Utc
                .timestamp_opt(ts.seconds(), 0)
                .single()
                .unwrap_or_default();
            results.push(CommitInfo {
                oid: oid.to_string(),
                message: commit.message().unwrap_or("").to_string(),
                author: commit.author().name().unwrap_or("").to_string(),
                timestamp: dt,
            });
        }
    }

    Ok(results)
}
