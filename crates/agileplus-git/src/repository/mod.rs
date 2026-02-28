//! Git branch and merge operations for AgilePlus.
//!
//! Traceability: WP07-T040

use agileplus_domain::{
    error::DomainError,
    ports::{ConflictInfo, MergeResult},
};
use git2::{BranchType, MergeAnalysis, Repository, Signature};
use git2::build::CheckoutBuilder;

use crate::{git_err, GitVcsAdapter};

/// Create a new branch from a base ref (branch name, tag, or commit SHA).
pub(crate) fn create_branch(
    adapter: &GitVcsAdapter,
    branch_name: &str,
    base: &str,
) -> Result<(), DomainError> {
    let repo = adapter.open_repo()?;
    let obj = repo.revparse_single(base).map_err(git_err)?;
    let commit = obj.peel_to_commit().map_err(git_err)?;
    repo.branch(branch_name, &commit, false).map_err(git_err)?;
    Ok(())
}

/// Check out an existing local branch, updating HEAD and working directory.
pub(crate) fn checkout_branch(
    adapter: &GitVcsAdapter,
    branch_name: &str,
) -> Result<(), DomainError> {
    let repo = adapter.open_repo()?;
    let branch = repo
        .find_branch(branch_name, BranchType::Local)
        .map_err(git_err)?;
    let refname = branch
        .get()
        .name()
        .ok_or_else(|| DomainError::Vcs("invalid branch ref name".into()))?
        .to_string();
    repo.set_head(&refname).map_err(git_err)?;
    repo.checkout_head(Some(CheckoutBuilder::new().force()))
        .map_err(git_err)?;
    Ok(())
}

/// Merge source branch into target. Returns MergeResult.
pub(crate) fn merge_to_target(
    adapter: &GitVcsAdapter,
    source: &str,
    target: &str,
) -> Result<MergeResult, DomainError> {
    // Checkout target first.
    checkout_branch(adapter, target)?;

    let repo = adapter.open_repo()?;

    // Resolve source branch to annotated commit.
    let source_ref = repo
        .find_branch(source, BranchType::Local)
        .map_err(git_err)?;
    let source_oid = source_ref
        .get()
        .peel_to_commit()
        .map_err(git_err)?
        .id();
    let annotated = repo
        .find_annotated_commit(source_oid)
        .map_err(git_err)?;

    let (analysis, _) = repo.merge_analysis(&[&annotated]).map_err(git_err)?;

    if analysis.contains(MergeAnalysis::ANALYSIS_UP_TO_DATE) {
        // Nothing to do.
        let head_oid = repo
            .head()
            .map_err(git_err)?
            .peel_to_commit()
            .map_err(git_err)?
            .id();
        return Ok(MergeResult {
            success: true,
            conflicts: vec![],
            merged_commit: Some(head_oid.to_string()),
        });
    }

    if analysis.contains(MergeAnalysis::ANALYSIS_FASTFORWARD) {
        // Fast-forward: just move the branch ref.
        let mut target_ref = repo
            .find_branch(target, BranchType::Local)
            .map_err(git_err)?
            .into_reference();
        target_ref
            .set_target(source_oid, "fast-forward merge")
            .map_err(git_err)?;
        repo.set_head(target_ref.name().unwrap()).map_err(git_err)?;
        repo.checkout_head(Some(CheckoutBuilder::new().force()))
            .map_err(git_err)?;
        return Ok(MergeResult {
            success: true,
            conflicts: vec![],
            merged_commit: Some(source_oid.to_string()),
        });
    }

    // Normal (three-way) merge.
    repo.merge(&[&annotated], None, None).map_err(git_err)?;
    let mut index = repo.index().map_err(git_err)?;

    if index.has_conflicts() {
        let conflicts = collect_conflicts(&index)?;
        // Clean up merge state so the repo isn't left dirty.
        repo.cleanup_state().map_err(git_err)?;
        // Reset index back.
        let head_commit = repo
            .head()
            .map_err(git_err)?
            .peel_to_commit()
            .map_err(git_err)?;
        repo.reset(head_commit.as_object(), git2::ResetType::Hard, None)
            .map_err(git_err)?;
        return Ok(MergeResult {
            success: false,
            conflicts,
            merged_commit: None,
        });
    }

    // Clean merge: create a merge commit.
    index.write().map_err(git_err)?;
    let tree_oid = index.write_tree().map_err(git_err)?;
    let tree = repo.find_tree(tree_oid).map_err(git_err)?;

    let sig = signature(&repo)?;
    let head_commit = repo
        .head()
        .map_err(git_err)?
        .peel_to_commit()
        .map_err(git_err)?;
    let source_commit = repo.find_commit(source_oid).map_err(git_err)?;

    let message = format!("Merge branch '{source}' into '{target}'");
    let merge_oid = repo
        .commit(
            Some("HEAD"),
            &sig,
            &sig,
            &message,
            &tree,
            &[&head_commit, &source_commit],
        )
        .map_err(git_err)?;

    repo.cleanup_state().map_err(git_err)?;

    Ok(MergeResult {
        success: true,
        conflicts: vec![],
        merged_commit: Some(merge_oid.to_string()),
    })
}

/// Detect conflicts between source and target without mutating repo state.
pub(crate) fn detect_conflicts(
    adapter: &GitVcsAdapter,
    source: &str,
    target: &str,
) -> Result<Vec<ConflictInfo>, DomainError> {
    let repo = adapter.open_repo()?;

    let source_commit = repo
        .find_branch(source, BranchType::Local)
        .map_err(git_err)?
        .get()
        .peel_to_commit()
        .map_err(git_err)?;

    let target_commit = repo
        .find_branch(target, BranchType::Local)
        .map_err(git_err)?
        .get()
        .peel_to_commit()
        .map_err(git_err)?;

    // Find merge base.
    let base_oid = repo
        .merge_base(source_commit.id(), target_commit.id())
        .map_err(git_err)?;
    let base_commit = repo.find_commit(base_oid).map_err(git_err)?;

    let source_tree = source_commit.tree().map_err(git_err)?;
    let target_tree = target_commit.tree().map_err(git_err)?;
    let base_tree = base_commit.tree().map_err(git_err)?;

    // Dry-run merge into an in-memory index.
    let index = repo
        .merge_trees(&base_tree, &target_tree, &source_tree, None)
        .map_err(git_err)?;

    if !index.has_conflicts() {
        return Ok(vec![]);
    }

    collect_conflicts(&index)
}

/// Extract ConflictInfo from an index that has conflicts.
fn collect_conflicts(index: &git2::Index) -> Result<Vec<ConflictInfo>, DomainError> {
    let mut conflicts = Vec::new();
    let iter = index.conflicts().map_err(git_err)?;
    for conflict in iter {
        let c = conflict.map_err(git_err)?;
        let path = c
            .our
            .as_ref()
            .or(c.their.as_ref())
            .or(c.ancestor.as_ref())
            .and_then(|e| std::str::from_utf8(&e.path).ok().map(|s| s.to_string()))
            .unwrap_or_default();
        conflicts.push(ConflictInfo {
            path,
            ours: c.our.map(|e| format!("{:?}", e.id)),
            theirs: c.their.map(|e| format!("{:?}", e.id)),
        });
    }
    Ok(conflicts)
}

/// Build a git Signature using repo config or fallback defaults.
fn signature(repo: &Repository) -> Result<Signature<'static>, DomainError> {
    repo.signature()
        .or_else(|_| Signature::now("AgilePlus", "agileplus@localhost"))
        .map_err(git_err)
}
