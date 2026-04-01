//! Worktree management using gix.
//!
//! Provides create, list, and prune operations for git worktrees.

use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{GitError, Result};

/// Information about a git worktree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub head: String,
    pub is_detached: bool,
    pub created_at: Option<DateTime<Utc>>,
}

/// High-level worktree manager.
pub struct WorktreeManager;

impl WorktreeManager {
    pub fn create(
        repo_path: impl AsRef<Path>,
        branch: &str,
        worktree_path: impl AsRef<Path>,
    ) -> Result<WorktreeInfo> {
        create_worktree(repo_path, branch, worktree_path)
    }

    pub fn list(repo_path: impl AsRef<Path>) -> Result<Vec<WorktreeInfo>> {
        list_worktrees(repo_path)
    }

    pub fn prune(repo_path: impl AsRef<Path>, max_age_days: u64) -> Result<usize> {
        prune_stale(repo_path, max_age_days)
    }
}

/// Create a new worktree at `worktree_path` from `branch`.
pub fn create_worktree(
    repo_path: impl AsRef<Path>,
    branch: &str,
    worktree_path: impl AsRef<Path>,
) -> Result<WorktreeInfo> {
    let repo_path = repo_path.as_ref();
    let worktree_path = worktree_path.as_ref();

    if !repo_path.join(".git").exists() && !repo_path.join("HEAD").exists() {
        return Err(GitError::RepoNotFound(repo_path.display().to_string()));
    }

    let output = Command::new("git")
        .args([
            "-C",
            repo_path.to_str().unwrap_or("."),
            "worktree",
            "add",
            worktree_path.to_str().unwrap_or("."),
            branch,
        ])
        .output()
        .map_err(|e| GitError::Operation(format!("failed to spawn git: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::Worktree(format!(
            "git worktree add failed: {stderr}"
        )));
    }

    info_for_path(worktree_path, repo_path)
}

/// List all worktrees for the repository at `repo_path`.
pub fn list_worktrees(repo_path: impl AsRef<Path>) -> Result<Vec<WorktreeInfo>> {
    let repo_path = repo_path.as_ref();
    let repo = gix::open(repo_path).map_err(|e| GitError::Gix(e.to_string()))?;

    let proxies = repo
        .worktrees()
        .map_err(|e| GitError::Worktree(e.to_string()))?;

    let mut infos = Vec::new();
    for proxy in proxies {
        let info = proxy_to_info(&repo, &proxy)?;
        infos.push(info);
    }

    Ok(infos)
}

/// Prune stale worktrees older than `max_age_days`.
pub fn prune_stale(repo_path: impl AsRef<Path>, max_age_days: u64) -> Result<usize> {
    let repo_path = repo_path.as_ref();
    let worktrees = list_worktrees(repo_path)?;

    let mut removed = 0;
    let cutoff = Utc::now()
        .checked_sub_signed(chrono::Duration::days(max_age_days as i64))
        .ok_or_else(|| GitError::Operation("date overflow".into()))?;

    for wt in &worktrees {
        let age_ok = wt.created_at.map_or(false, |t| t < cutoff);
        if !age_ok {
            continue;
        }

        let output = Command::new("git")
            .args(["-C", repo_path.to_str().unwrap_or("."), "worktree", "prune"])
            .output()
            .map_err(|e| GitError::Operation(format!("failed to spawn git: {e}")))?;

        if output.status.success() {
            removed += 1;
        }
    }

    Ok(removed)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn proxy_to_info(repo: &gix::Repository, proxy: &gix::worktree::Proxy<'_>) -> Result<WorktreeInfo> {
    let path = proxy.base().to_path_buf();

    let head_commit = repo
        .head_commit()
        .map_err(|e| GitError::Gix(e.to_string()))?;
    let head = head_commit.id.to_string();

    let head_ref = repo.head().map_err(|e| GitError::Gix(e.to_string()))?;
    let is_detached = head_ref.is_detached();
    let branch = if is_detached {
        None
    } else {
        head_ref.referent_name().map(|n| n.shorten().to_string())
    };

    let created_at = detect_created_at(&path);

    Ok(WorktreeInfo {
        path,
        branch,
        head,
        is_detached,
        created_at,
    })
}

fn info_for_path(worktree_path: &Path, repo_path: &Path) -> Result<WorktreeInfo> {
    let repo = gix::open(repo_path).map_err(|e| GitError::Gix(e.to_string()))?;

    let head_commit = repo
        .head_commit()
        .map_err(|e| GitError::Gix(e.to_string()))?;
    let head = head_commit.id.to_string();

    let head_ref = repo.head().map_err(|e| GitError::Gix(e.to_string()))?;
    let is_detached = head_ref.is_detached();
    let branch = if is_detached {
        None
    } else {
        head_ref.referent_name().map(|n| n.shorten().to_string())
    };

    let created_at = detect_created_at(worktree_path);

    Ok(WorktreeInfo {
        path: worktree_path.to_path_buf(),
        branch,
        head,
        is_detached,
        created_at,
    })
}

fn detect_created_at(path: &Path) -> Option<DateTime<Utc>> {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.created().ok())
        .map(|t| DateTime::<Utc>::from(t))
        .or_else(|| {
            std::fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| DateTime::<Utc>::from(t))
        })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-GIT-001
    #[test]
    fn worktree_info_serialization() {
        let info = WorktreeInfo {
            path: PathBuf::from("/tmp/wt"),
            branch: Some("feature/x".into()),
            head: "abc123".into(),
            is_detached: false,
            created_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("feature/x"));
        assert!(json.contains("abc123"));

        let roundtrip: WorktreeInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.branch, Some("feature/x".into()));
        assert_eq!(roundtrip.head, "abc123");
        assert!(!roundtrip.is_detached);
    }

    // Traces to: FR-GIT-002
    #[test]
    fn worktree_info_detached() {
        let info = WorktreeInfo {
            path: PathBuf::from("/tmp/wt"),
            branch: None,
            head: "def456".into(),
            is_detached: true,
            created_at: None,
        };
        assert!(info.is_detached);
        assert!(info.branch.is_none());
        assert_eq!(info.head, "def456");
    }

    // Traces to: FR-GIT-003
    #[test]
    fn create_worktree_nonexistent_repo() {
        let result = create_worktree("/nonexistent/path", "main", "/tmp/wt");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitError::RepoNotFound(_)));
    }

    // Traces to: FR-GIT-004
    #[test]
    fn list_worktrees_nonexistent_repo() {
        let result = list_worktrees("/nonexistent/path");
        assert!(result.is_err());
    }

    // Traces to: FR-GIT-005
    #[test]
    fn worktree_manager_api() {
        let manager = WorktreeManager;
        let result = manager.list("/nonexistent");
        assert!(result.is_err());
    }

    // Traces to: FR-GIT-006
    #[test]
    fn detect_created_at_existing_path() {
        let dir = tempfile::tempdir().unwrap();
        let created = detect_created_at(dir.path());
        assert!(created.is_some());
    }

    // Traces to: FR-GIT-007
    #[test]
    fn detect_created_at_nonexistent_path() {
        let created = detect_created_at(Path::new("/nonexistent/path"));
        assert!(created.is_none());
    }

    // Traces to: FR-GIT-008
    #[test]
    fn git_error_display() {
        let err = GitError::RepoNotFound("/tmp/repo".into());
        assert!(err.to_string().contains("/tmp/repo"));

        let err = GitError::Worktree("add failed".into());
        assert!(err.to_string().contains("add failed"));
    }

    // Traces to: FR-GIT-009
    #[test]
    fn list_worktrees_empty_repo() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path();

        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let result = list_worktrees(repo_path);
        assert!(result.is_ok());
        let worktrees = result.unwrap();
        assert!(!worktrees.is_empty());
    }

    // Traces to: FR-GIT-010
    #[test]
    fn create_and_list_worktree() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path();

        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["commit", "--allow-empty", "-m", "init"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let wt_path = dir.path().join("wt1");
        let result = create_worktree(repo_path, "main", &wt_path);
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.path, wt_path);
        assert_eq!(info.branch, Some("main".into()));

        let all = list_worktrees(repo_path).unwrap();
        assert!(all.len() >= 2);
    }
}
