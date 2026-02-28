//! Git worktree management for AgilePlus.
//!
//! Worktree convention: `.worktrees/<feature-slug>-<wp-id>/`
//! Traceability: WP07-T039

use std::path::{Path, PathBuf};

use agileplus_domain::{error::DomainError, ports::WorktreeInfo};
use git2::{WorktreeAddOptions};

use crate::{git_err, GitVcsAdapter};

/// Worktree directory name relative to repo root.
fn worktree_name(feature_slug: &str, wp_id: &str) -> String {
    format!("{feature_slug}-{wp_id}")
}

/// Absolute path to the worktree directory.
fn worktree_path(repo_root: &Path, feature_slug: &str, wp_id: &str) -> PathBuf {
    repo_root
        .join(".worktrees")
        .join(worktree_name(feature_slug, wp_id))
}

/// Create a git worktree for a feature/WP pair.
pub(crate) fn create_worktree(
    adapter: &GitVcsAdapter,
    feature_slug: &str,
    wp_id: &str,
) -> Result<PathBuf, DomainError> {
    let repo = adapter.open_repo()?;
    let name = worktree_name(feature_slug, wp_id);
    let path = worktree_path(adapter.repo_path(), feature_slug, wp_id);

    // Create parent .worktrees/ dir if needed.
    let worktrees_dir = adapter.repo_path().join(".worktrees");
    std::fs::create_dir_all(&worktrees_dir)
        .map_err(|e| DomainError::Vcs(format!("create .worktrees dir: {e}")))?;

    // Create a branch for this worktree from HEAD.
    let head = repo.head().map_err(git_err)?;
    let head_commit = head.peel_to_commit().map_err(git_err)?;

    // Create branch (ignore if already exists).
    let branch = match repo.branch(&name, &head_commit, false) {
        Ok(b) => b,
        Err(e) if e.code() == git2::ErrorCode::Exists => {
            repo.find_branch(&name, git2::BranchType::Local).map_err(git_err)?
        }
        Err(e) => return Err(git_err(e)),
    };

    let branch_ref = branch.get();
    let _refname = branch_ref
        .name()
        .ok_or_else(|| DomainError::Vcs("invalid branch ref name".into()))?;

    let mut opts = WorktreeAddOptions::new();
    opts.reference(Some(branch_ref));

    repo.worktree(&name, &path, Some(&opts))
        .map_err(|e| DomainError::Vcs(format!("create worktree '{name}': {e}")))?;

    let canonical = std::fs::canonicalize(&path)
        .unwrap_or(path);

    Ok(canonical)
}

/// List all active git worktrees.
pub(crate) fn list_worktrees(adapter: &GitVcsAdapter) -> Result<Vec<WorktreeInfo>, DomainError> {
    let repo = adapter.open_repo()?;
    let names = repo.worktrees().map_err(git_err)?;

    let mut result = Vec::new();
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

        // Parse feature_slug and wp_id from worktree name (split on last '-').
        let (feature_slug, wp_id) = if let Some(pos) = name.rfind('-') {
            (name[..pos].to_string(), name[pos + 1..].to_string())
        } else {
            (name.to_string(), String::new())
        };

        // Get branch name from the worktree's HEAD.
        let branch = wt_branch_name(name, &path);

        result.push(WorktreeInfo {
            path,
            branch,
            feature_slug,
            wp_id,
        });
    }

    Ok(result)
}

/// Try to determine the branch name for a worktree.
fn wt_branch_name(wt_name: &str, wt_path: &Path) -> String {
    // Open the worktree repo to read its HEAD.
    if let Ok(wt_repo) = git2::Repository::open(wt_path) {
        if let Ok(head) = wt_repo.head() {
            if let Some(shorthand) = head.shorthand() {
                return shorthand.to_string();
            }
        }
    }
    wt_name.to_string()
}

/// Remove a worktree at the given path.
pub(crate) fn cleanup_worktree(
    adapter: &GitVcsAdapter,
    worktree_path: &Path,
) -> Result<(), DomainError> {
    // Safety check: path must be under .worktrees/.
    let worktrees_dir = adapter.repo_path().join(".worktrees");
    let canonical_path = std::fs::canonicalize(worktree_path)
        .unwrap_or_else(|_| worktree_path.to_path_buf());
    let canonical_wt_dir = std::fs::canonicalize(&worktrees_dir)
        .unwrap_or(worktrees_dir);

    if !canonical_path.starts_with(&canonical_wt_dir) {
        return Err(DomainError::Vcs(format!(
            "worktree path '{}' is not under .worktrees/",
            worktree_path.display()
        )));
    }

    let repo = adapter.open_repo()?;

    // Find the worktree by matching its path.
    let names = repo.worktrees().map_err(git_err)?;
    let mut found_name: Option<String> = None;
    for name_bytes in names.iter() {
        let name = match name_bytes {
            Some(n) => n,
            None => continue,
        };
        if let Ok(wt) = repo.find_worktree(name) {
            let wt_canonical = std::fs::canonicalize(wt.path())
                .unwrap_or_else(|_| PathBuf::from(wt.path()));
            if wt_canonical == canonical_path {
                found_name = Some(name.to_string());
                break;
            }
        }
    }

    if let Some(name) = found_name {
        let wt = repo.find_worktree(&name).map_err(git_err)?;
        let mut prune_opts = git2::WorktreePruneOptions::new();
        prune_opts.valid(true); // prune even if still valid
        wt.prune(Some(&mut prune_opts)).map_err(git_err)?;
    }

    // Remove the directory from filesystem.
    if canonical_path.exists() {
        std::fs::remove_dir_all(&canonical_path)
            .map_err(|e| DomainError::Vcs(format!("remove worktree dir: {e}")))?;
    }

    Ok(())
}
