//! Point-in-time snapshot of git repository state using git2.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSnapshot {
    pub head_commit: String,
    pub branch: Option<String>,
    pub is_detached: bool,
    pub dirty_files: Vec<DirtyFile>,
    pub worktrees: Vec<WorktreeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirtyFile {
    pub path: String,
    pub status: FileStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub head_commit: String,
    pub is_main: bool,
}

impl GitSnapshot {
    /// Take a snapshot using git2. Falls back gracefully if repo is not available.
    pub fn capture(
        repo_root: &std::path::Path,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let repo = git2::Repository::open(repo_root)?;

        // Get HEAD
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?.id().to_string();
        let is_detached = repo.head_detached()?;
        let branch = if is_detached {
            None
        } else {
            head.shorthand().map(|s| s.to_string())
        };

        // Get dirty files
        let mut dirty_files = Vec::new();
        let statuses = repo.statuses(Some(
            git2::StatusOptions::new()
                .include_untracked(true)
                .recurse_untracked_dirs(false),
        ))?;

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = entry.status();
            let file_status = if status.contains(git2::Status::WT_NEW)
                || status.contains(git2::Status::INDEX_NEW)
            {
                FileStatus::Added
            } else if status.contains(git2::Status::WT_DELETED)
                || status.contains(git2::Status::INDEX_DELETED)
            {
                FileStatus::Deleted
            } else if status.contains(git2::Status::WT_MODIFIED)
                || status.contains(git2::Status::INDEX_MODIFIED)
            {
                FileStatus::Modified
            } else {
                FileStatus::Untracked
            };
            dirty_files.push(DirtyFile {
                path,
                status: file_status,
            });
        }

        // Build worktrees list
        let mut worktrees = Vec::new();

        // Main worktree
        worktrees.push(WorktreeInfo {
            path: repo_root.to_path_buf(),
            branch: branch.clone(),
            head_commit: head_commit.clone(),
            is_main: true,
        });

        // Linked worktrees
        if let Ok(wt_names) = repo.worktrees() {
            for name in wt_names.iter().flatten() {
                if let Ok(wt) = repo.find_worktree(name) {
                    let wt_path = wt.path().to_path_buf();
                    if let Ok(wt_repo) = git2::Repository::open(&wt_path) {
                        let wt_head = wt_repo.head().ok();
                        let wt_commit = wt_head
                            .as_ref()
                            .and_then(|h| h.peel_to_commit().ok())
                            .map(|c| c.id().to_string())
                            .unwrap_or_default();
                        let wt_branch = wt_head
                            .as_ref()
                            .and_then(|h| h.shorthand().map(|s| s.to_string()));
                        worktrees.push(WorktreeInfo {
                            path: wt_path,
                            branch: wt_branch,
                            head_commit: wt_commit,
                            is_main: false,
                        });
                    }
                }
            }
        }

        Ok(Self {
            head_commit,
            branch,
            is_detached,
            dirty_files,
            worktrees,
        })
    }
}
