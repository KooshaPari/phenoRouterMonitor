//! Git adapter plugin for AgilePlus.
//!
//! Implements [`agileplus_plugin_core::traits::VcsPlugin`] using git2.
//!
//! ## Architecture
//!
//! This crate follows the Hexagonal Architecture pattern:
//! - **Port**: `VcsPlugin` trait from `agileplus-plugin-core`
//! - **Adapter**: `GitAdapter` struct implementing the port

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use git2::{Repository, BranchType};

use agileplus_plugin_core::{
    error::{PluginError, PluginResult},
    traits::{
        AdapterPlugin, ConflictInfo, FeatureArtifacts, MergeResult, PluginConfig, VcsPlugin,
        WorktreeInfo,
    },
};

/// Map a git2 error to a PluginError.
fn git_err(e: git2::Error) -> PluginError {
    match e.code() {
        git2::ErrorCode::NotFound => PluginError::NotFound(e.message().to_string()),
        git2::ErrorCode::Exists => PluginError::AlreadyExists(e.message().to_string()),
        _ => PluginError::Operation(e.message().to_string()),
    }
}

/// Git adapter implementing [`VcsPlugin`] using git2.
pub struct GitAdapter {
    repo_path: PathBuf,
}

impl GitAdapter {
    /// Create a new Git adapter pointing to an existing git repository.
    pub fn new(repo_path: impl Into<PathBuf>) -> PluginResult<Self> {
        let repo_path = repo_path.into();
        Repository::open(&repo_path).map_err(git_err)?;
        Ok(Self { repo_path })
    }

    /// Create adapter from current working directory.
    pub fn from_cwd() -> PluginResult<Self> {
        let cwd = std::env::current_dir().map_err(|e| {
            PluginError::Initialization(format!("Failed to get current directory: {}", e))
        })?;
        Self::new(cwd)
    }

    /// Get the repository path.
    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }

    /// Open a fresh repository handle.
    fn open_repo(&self) -> Result<Repository, PluginError> {
        Repository::open(&self.repo_path).map_err(git_err)
    }

    /// Get the main branch name (usually "main" or "master").
    fn main_branch_name(&self) -> PluginResult<String> {
        let repo = self.open_repo()?;

        // Try "main" first, then "master"
        for name in ["main", "master"] {
            if repo.find_branch(name, BranchType::Local).is_ok() {
                return Ok(name.to_string());
            }
        }

        // Fall back to HEAD
        let head = repo.head().map_err(|e| {
            PluginError::NotFound(format!("Could not determine main branch: {}", e))
        })?;

        let name = if head.is_branch() {
            head.name()
                .map(|n| n.strip_prefix("refs/heads/").unwrap_or(n).to_string())
                .unwrap_or_else(|| "main".to_string())
        } else {
            "main".to_string()
        };

        Ok(name)
    }
}

// Safety: GitAdapter only stores PathBuf which is Send+Sync.
unsafe impl Send for GitAdapter {}
unsafe impl Sync for GitAdapter {}

#[async_trait]
impl AdapterPlugin for GitAdapter {
    fn name(&self) -> &str {
        "git"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn initialize(&self, _config: PluginConfig) -> PluginResult<()> {
        Ok(())
    }
}

#[async_trait]
impl VcsPlugin for GitAdapter {
    async fn create_worktree(
        &self,
        feature_slug: &str,
        wp_id: &str,
    ) -> PluginResult<PathBuf> {
        let repo = self.open_repo()?;
        let base_branch = self.main_branch_name()?;

        let branch_name = format!("feature/{}", feature_slug.replace('_', "-"));

        if repo.find_branch(&branch_name, BranchType::Local).is_ok() {
            return Err(PluginError::AlreadyExists(format!(
                "Branch '{}' already exists",
                branch_name
            )));
        }

        let worktree_path = self.repo_path.join(".worktrees").join(wp_id);
        std::fs::create_dir_all(&worktree_path)?;

        let base_ref = repo.find_branch(&base_branch, BranchType::Local)
            .map_err(|e| PluginError::NotFound(format!("Base branch not found: {}", e)))?;
        let base_commit = base_ref.get().peel_to_commit().map_err(git_err)?;

        let wt_repo = Repository::init(&worktree_path).map_err(|e| {
            PluginError::Operation(format!("Failed to init worktree: {}", e))
        })?;

        wt_repo.branch(&branch_name, &base_commit, false).map_err(|e| {
            PluginError::Operation(format!("Failed to create branch: {}", e))
        })?;

        wt_repo.set_head(&format!("refs/heads/{}", branch_name)).map_err(|e| {
            PluginError::Operation(format!("Failed to checkout branch: {}", e))
        })?;

        Ok(worktree_path)
    }

    async fn list_worktrees(&self) -> PluginResult<Vec<WorktreeInfo>> {
        let repo = self.open_repo()?;
        let names = repo.worktrees().map_err(|e| {
            PluginError::Operation(format!("Failed to list worktrees: {}", e))
        })?;

        let mut worktrees = Vec::new();

        for name_bytes in names.iter() {
            let name = match name_bytes {
                Some(n) => n,
                None => continue,
            };

            let wt = match repo.find_worktree(name) {
                Ok(w) => w,
                Err(_) => continue,
            };

            let path = PathBuf::from(wt.path());
            let name_str = name.to_string();

            // Parse feature_slug and wp_id from worktree name
            let (feature_slug, wp_id) = if let Some(pos) = name_str.rfind('-') {
                let potential_wp = &name_str[pos + 1..];
                if potential_wp.starts_with("WP")
                    && potential_wp.len() > 2
                    && potential_wp[2..].chars().all(|c| c.is_ascii_digit())
                {
                    (name_str[..pos].to_string(), potential_wp.to_string())
                } else {
                    (name_str.clone(), String::new())
                }
            } else {
                (name_str.clone(), String::new())
            };

            // Get branch name from worktree HEAD
            let branch = if let Ok(wt_repo) = Repository::open(&path) {
                wt_repo.head().ok().and_then(|h| h.shorthand().map(String::from)).unwrap_or_else(|| name_str.clone())
            } else {
                name_str.clone()
            };

            worktrees.push(WorktreeInfo {
                path,
                branch,
                feature_slug,
                wp_id,
            });
        }

        Ok(worktrees)
    }

    async fn cleanup_worktree(&self, worktree_path: &Path) -> PluginResult<()> {
        let repo = self.open_repo()?;

        // Find the worktree by matching its path
        let names = repo.worktrees().map_err(|e| {
            PluginError::Operation(format!("Failed to list worktrees: {}", e))
        })?;

        let mut found_name: Option<String> = None;
        for name_bytes in names.iter() {
            let name = match name_bytes {
                Some(n) => n,
                None => continue,
            };
            if let Ok(wt) = repo.find_worktree(name) {
                let wt_path = PathBuf::from(wt.path());
                let canonical_path = std::fs::canonicalize(worktree_path).unwrap_or_else(|_| worktree_path.to_path_buf());
                let canonical_wt = std::fs::canonicalize(&wt_path).unwrap_or(wt_path);
                if canonical_path == canonical_wt {
                    found_name = Some(name.to_string());
                    break;
                }
            }
        }

        if let Some(name) = found_name {
            let wt = repo.find_worktree(&name).map_err(git_err)?;
            let mut prune_opts = git2::WorktreePruneOptions::new();
            prune_opts.valid(true);
            wt.prune(Some(&mut prune_opts)).map_err(git_err)?;
        }

        // Remove the directory
        if worktree_path.exists() {
            std::fs::remove_dir_all(worktree_path)?;
        }

        Ok(())
    }

    async fn create_branch(&self, branch_name: &str, base: &str) -> PluginResult<()> {
        let repo = self.open_repo()?;

        let base_branch = repo.find_branch(base, BranchType::Local)
            .map_err(|e| PluginError::NotFound(format!("Base branch '{}' not found: {}", base, e)))?;
        let base_commit = base_branch.get().peel_to_commit().map_err(git_err)?;

        repo.branch(branch_name, &base_commit, false).map_err(|e| {
            PluginError::Operation(format!("Failed to create branch: {}", e))
        })?;

        Ok(())
    }

    async fn checkout_branch(&self, branch_name: &str) -> PluginResult<()> {
        let repo = self.open_repo()?;

        let branch = repo.find_branch(branch_name, BranchType::Local)
            .map_err(|e| PluginError::NotFound(format!("Branch '{}' not found: {}", branch_name, e)))?;

        branch.get().peel_to_commit().map_err(git_err)?;

        let mut checkout_opts = git2::build::CheckoutBuilder::new();
        checkout_opts.force();

        repo.checkout_head(Some(&mut checkout_opts)).map_err(|e| {
            PluginError::Operation(format!("Failed to checkout branch: {}", e))
        })?;

        Ok(())
    }

    async fn merge_to_target(
        &self,
        source: &str,
        target: &str,
    ) -> PluginResult<MergeResult> {
        let repo = self.open_repo()?;

        self.checkout_branch(target).await?;

        let source_branch = repo.find_branch(source, BranchType::Local)
            .map_err(|e| PluginError::NotFound(format!("Source branch '{}' not found: {}", source, e)))?;
        let target_branch = repo.find_branch(target, BranchType::Local)
            .map_err(|e| PluginError::NotFound(format!("Target branch '{}' not found: {}", target, e)))?;

        let source_commit = source_branch.get().peel_to_commit().map_err(git_err)?;
        let target_commit = target_branch.get().peel_to_commit().map_err(git_err)?;

        let mut merge_opts = git2::MergeOptions::new();
        let mut checkout_opts = git2::build::CheckoutBuilder::new();
        checkout_opts.force();

        // Find annotated commit for merge
        let source_oid = source_commit.id();
        let annotated = repo.find_annotated_commit(source_oid)
            .map_err(|e| PluginError::Operation(format!("Failed to find annotated commit: {}", e)))?;

        repo.merge(&[&annotated], Some(&mut merge_opts), Some(&mut checkout_opts))
            .map_err(|e| PluginError::Operation(format!("Failed to perform merge: {}", e)))?;

        let mut index = repo.index().map_err(|e| {
            PluginError::Operation(format!("Failed to get index: {}", e))
        })?;

        let conflicts: Vec<ConflictInfo> = if index.has_conflicts() {
            vec![ConflictInfo {
                path: "conflicts detected".to_string(),
                ours: None,
                theirs: None,
            }]
        } else {
            vec![]
        };

        if !conflicts.is_empty() {
            return Ok(MergeResult {
                success: false,
                conflicts,
                merged_commit: None,
            });
        }

        let signature = repo.signature().map_err(|e| {
            PluginError::Operation(format!("Failed to get signature: {}", e))
        })?;

        let tree = index.write_tree().map_err(|e| {
            PluginError::Operation(format!("Failed to write tree: {}", e))
        })?;

        let tree = repo.find_tree(tree).map_err(git_err)?;

        let commit_id = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("Merge branch '{}' into '{}'", source, target),
            &tree,
            &[&target_commit, &source_commit],
        ).map_err(|e| {
            PluginError::Operation(format!("Failed to create merge commit: {}", e))
        })?;

        let _ = repo.cleanup_state();

        Ok(MergeResult {
            success: true,
            conflicts: vec![],
            merged_commit: Some(commit_id.to_string()),
        })
    }

    async fn detect_conflicts(
        &self,
        source: &str,
        target: &str,
    ) -> PluginResult<Vec<ConflictInfo>> {
        let repo = self.open_repo()?;

        let source_branch = repo.find_branch(source, BranchType::Local)
            .map_err(|e| PluginError::NotFound(format!("Source branch '{}' not found: {}", source, e)))?;
        let target_branch = repo.find_branch(target, BranchType::Local)
            .map_err(|e| PluginError::NotFound(format!("Target branch '{}' not found: {}", target, e)))?;

        let source_commit = source_branch.get().peel_to_commit().map_err(git_err)?;
        let target_commit = target_branch.get().peel_to_commit().map_err(git_err)?;

        let diff = repo.diff_tree_to_tree(
            Some(&target_commit.tree().map_err(git_err)?),
            Some(&source_commit.tree().map_err(git_err)?),
            None,
        ).map_err(|e| {
            PluginError::Operation(format!("Failed to diff trees: {}", e))
        })?;

        let mut conflicts = Vec::new();

        diff.foreach(
            &mut |delta, _| {
                let path = delta.new_file().path()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                if !path.is_empty() {
                    conflicts.push(ConflictInfo {
                        path,
                        ours: None,
                        theirs: None,
                    });
                }
                true
            },
            None,
            None,
            None,
        ).map_err(|e| {
            PluginError::Operation(format!("Failed to iterate diff: {}", e))
        })?;

        Ok(conflicts)
    }

    async fn read_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> PluginResult<String> {
        let artifact_path = self
            .repo_path
            .join("kitty-specs")
            .join(feature_slug)
            .join(relative_path);

        std::fs::read_to_string(&artifact_path).map_err(|e| {
            PluginError::NotFound(format!("Artifact not found at {:?}: {}", artifact_path, e))
        })
    }

    async fn write_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
        content: &str,
    ) -> PluginResult<()> {
        let artifact_path = self
            .repo_path
            .join("kitty-specs")
            .join(feature_slug)
            .join(relative_path);

        if let Some(parent) = artifact_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&artifact_path, content).map_err(|e| {
            PluginError::Io(e)
        })
    }

    async fn artifact_exists(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> PluginResult<bool> {
        let artifact_path = self
            .repo_path
            .join("kitty-specs")
            .join(feature_slug)
            .join(relative_path);

        Ok(artifact_path.exists())
    }

    async fn scan_feature_artifacts(
        &self,
        feature_slug: &str,
    ) -> PluginResult<FeatureArtifacts> {
        let feature_path = self.repo_path.join("kitty-specs").join(feature_slug);

        if !feature_path.exists() {
            return Ok(FeatureArtifacts {
                meta_json: None,
                audit_chain: None,
                evidence_paths: vec![],
            });
        }

        let mut artifacts = FeatureArtifacts {
            meta_json: None,
            audit_chain: None,
            evidence_paths: vec![],
        };

        let meta_path = feature_path.join("meta.json");
        if meta_path.exists() {
            artifacts.meta_json = Some(meta_path.to_string_lossy().to_string());
        }

        let audit_path = feature_path.join("audit");
        if audit_path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(audit_path) {
                for entry in entries.flatten() {
                    if entry.path().is_file() {
                        artifacts.evidence_paths.push(entry.path().to_string_lossy().to_string());
                    }
                }
            }
        }

        Ok(artifacts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> PluginResult<(TempDir, GitAdapter)> {
        let temp_dir = TempDir::new().map_err(|e| PluginError::Io(e))?;
        let repo_path = temp_dir.path();
        Repository::init(repo_path).map_err(|e| PluginError::Initialization(format!("Failed to init repo: {}", e)))?;
        let adapter = GitAdapter::new(repo_path)?;
        Ok((temp_dir, adapter))
    }

    #[tokio::test]
    async fn test_adapter_name_and_version() -> PluginResult<()> {
        let (_dir, adapter) = create_test_repo()?;
        assert_eq!(adapter.name(), "git");
        assert_eq!(adapter.version(), env!("CARGO_PKG_VERSION"));
        Ok(())
    }

    #[tokio::test]
    async fn test_artifact_operations() -> PluginResult<()> {
        let (_dir, adapter) = create_test_repo()?;

        adapter.write_artifact("test-feature", "test.txt", "Hello, World!").await?;

        assert!(adapter.artifact_exists("test-feature", "test.txt").await?);

        let content = adapter.read_artifact("test-feature", "test.txt").await?;
        assert_eq!(content, "Hello, World!");

        Ok(())
    }
}
