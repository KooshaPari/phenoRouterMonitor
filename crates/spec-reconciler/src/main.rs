use anyhow::{anyhow, Result};
use git2::{Repository, MergeOptions, IndexAddOption, Oid};
use std::path::Path;
use tracing::{info, warn, error};

#[derive(Debug, Clone)]
struct ReconciliationResult {
    merged: Vec<String>,
    conflicted: Vec<ConflictedBranch>,
    skipped: Vec<String>,
    errors: Vec<String>,
}

#[derive(Debug, Clone)]
struct ConflictedBranch {
    branch_name: String,
    conflicted_files: Vec<String>,
    error_message: String,
}

impl ReconciliationResult {
    fn new() -> Self {
        Self {
            merged: Vec::new(),
            conflicted: Vec::new(),
            skipped: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn print_summary(&self) {
        info!("=== Spec Reconciliation Summary ===");
        info!("✅ Merged: {}", self.merged.len());
        for branch in &self.merged {
            info!("   - {}", branch);
        }

        if !self.conflicted.is_empty() {
            info!("⚠️  Conflicted: {}", self.conflicted.len());
            for conflict in &self.conflicted {
                info!("   - {} (files: {})", conflict.branch_name, conflict.conflicted_files.join(", "));
            }
        }

        info!("⊘ Skipped: {}", self.skipped.len());
        for branch in &self.skipped {
            info!("   - {}", branch);
        }

        if !self.errors.is_empty() {
            error!("❌ Errors: {}", self.errors.len());
            for err in &self.errors {
                error!("   - {}", err);
            }
        }

        info!("===================================");
    }
}

/// Reconciles spec branches (specs/agent-*) into specs/main
/// Handles auto-merges and flags conflicts for manual review
async fn reconcile_specs(repo_path: &Path) -> Result<ReconciliationResult> {
    let repo = Repository::open(repo_path)
        .map_err(|e| anyhow!("Failed to open repository: {}", e))?;

    info!("Opened repository at {}", repo_path.display());

    // Ensure specs/main branch exists; if not, create it from main
    ensure_specs_main_branch(&repo)?;

    let mut result = ReconciliationResult::new();

    // List all branches matching specs/agent-*
    let agent_branches = list_agent_branches(&repo)?;
    info!("Found {} agent spec branches to reconcile", agent_branches.len());

    if agent_branches.is_empty() {
        info!("No agent branches found matching 'specs/agent-*'");
        result.print_summary();
        return Ok(result);
    }

    // For each agent branch, attempt merge into specs/main
    for branch_name in agent_branches {
        match merge_branch_to_main(&repo, &branch_name) {
            Ok(_) => {
                info!("✅ Successfully merged {} to specs/main", branch_name);
                result.merged.push(branch_name);
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("conflict") {
                    // Extract conflicted files from index
                    let conflicted_files = extract_conflicted_files(&repo).unwrap_or_default();
                    info!("⚠️  Conflict in {}: {} file(s)", branch_name, conflicted_files.len());

                    // Abort the merge to leave repo in clean state
                    abort_merge(&repo).ok();

                    result.conflicted.push(ConflictedBranch {
                        branch_name,
                        conflicted_files,
                        error_message: error_msg,
                    });
                } else {
                    error!("❌ Error merging {}: {}", branch_name, e);
                    result.errors.push(format!("{}: {}", branch_name, e));
                }
            }
        }
    }

    result.print_summary();
    Ok(result)
}

/// Ensures specs/main branch exists; creates from main if needed
fn ensure_specs_main_branch(repo: &Repository) -> Result<()> {
    // Try to get specs/main
    match repo.find_branch("specs/main", git2::BranchType::Local) {
        Ok(_) => {
            info!("specs/main branch exists");
            Ok(())
        }
        Err(_) => {
            info!("specs/main does not exist; creating from main");

            // Get the main branch HEAD
            let main_ref = repo.find_reference("refs/heads/main")
                .or_else(|_| repo.find_reference("refs/remotes/origin/main"))
                .map_err(|e| anyhow!("Could not find main or origin/main: {}", e))?;

            let main_oid = main_ref.target()
                .ok_or_else(|| anyhow!("main reference has no target"))?;

            let main_commit = repo.find_commit(main_oid)
                .map_err(|e| anyhow!("Could not find main commit: {}", e))?;

            // Create specs/main from main
            repo.branch("specs/main", &main_commit, false)
                .map_err(|e| anyhow!("Failed to create specs/main branch: {}", e))?;

            info!("Created specs/main branch");
            Ok(())
        }
    }
}

/// Lists all branches matching specs/agent-* pattern
fn list_agent_branches(repo: &Repository) -> Result<Vec<String>> {
    let mut branches = Vec::new();

    let branch_iter = repo.branches(Some(git2::BranchType::Local))
        .map_err(|e| anyhow!("Failed to list branches: {}", e))?;

    for branch_result in branch_iter {
        let (branch, _) = branch_result
            .map_err(|e| anyhow!("Error iterating branches: {}", e))?;

        if let Ok(name) = branch.name() {
            if let Some(name) = name {
                if name.starts_with("specs/agent-") {
                    branches.push(name.to_string());
                }
            }
        }
    }

    Ok(branches)
}

/// Merges a branch into specs/main
fn merge_branch_to_main(repo: &Repository, branch_name: &str) -> Result<()> {
    // Get reference to specs/main
    let main_ref = repo.find_reference("refs/heads/specs/main")
        .map_err(|e| anyhow!("Could not find specs/main: {}", e))?;

    // Checkout specs/main
    let main_commit = repo.reference_to_annotated_commit(&main_ref)
        .map_err(|e| anyhow!("Could not get specs/main commit: {}", e))?;

    repo.set_head("refs/heads/specs/main")
        .map_err(|e| anyhow!("Failed to set HEAD to specs/main: {}", e))?;

    repo.checkout_tree(main_commit.get_tree()?, None)
        .map_err(|e| anyhow!("Failed to checkout specs/main: {}", e))?;

    // Get the agent branch
    let agent_ref = repo.find_reference(&format!("refs/heads/{}", branch_name))
        .map_err(|e| anyhow!("Could not find branch {}: {}", branch_name, e))?;

    let agent_commit = repo.reference_to_annotated_commit(&agent_ref)
        .map_err(|e| anyhow!("Could not get commit for {}: {}", branch_name, e))?;

    // Perform merge
    let mut merge_options = MergeOptions::new();
    merge_options.fail_on_conflict(true);

    repo.merge(&[&agent_commit], Some(&mut merge_options), None)
        .map_err(|e| anyhow!("Merge operation failed: {}", e))?;

    // Check for conflicts
    let index = repo.index()
        .map_err(|e| anyhow!("Failed to get index: {}", e))?;

    if index.has_conflicts() {
        return Err(anyhow!("conflict: Merge resulted in conflicts"));
    }

    // Stage all changes
    let mut index = repo.index()
        .map_err(|e| anyhow!("Failed to get index: {}", e))?;

    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .map_err(|e| anyhow!("Failed to stage changes: {}", e))?;

    index.write()
        .map_err(|e| anyhow!("Failed to write index: {}", e))?;

    // Create commit
    let tree = index.write_tree()
        .map_err(|e| anyhow!("Failed to write tree: {}", e))?;

    let tree_obj = repo.find_tree(tree)
        .map_err(|e| anyhow!("Failed to find tree: {}", e))?;

    let sig = repo.signature()
        .map_err(|e| anyhow!("Failed to get signature: {}", e))?;

    let main_commit = repo.find_commit(main_commit.id())
        .map_err(|e| anyhow!("Failed to find main commit: {}", e))?;

    let agent_commit_obj = repo.find_commit(agent_commit.id())
        .map_err(|e| anyhow!("Failed to find agent commit: {}", e))?;

    let message = format!("Merge {} into specs/main", branch_name);

    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &message,
        &tree_obj,
        &[&main_commit, &agent_commit_obj],
    )
    .map_err(|e| anyhow!("Failed to create commit: {}", e))?;

    info!("✅ Merged {} to specs/main", branch_name);
    Ok(())
}

/// Extracts list of conflicted files from the index
fn extract_conflicted_files(repo: &Repository) -> Result<Vec<String>> {
    let index = repo.index()
        .map_err(|e| anyhow!("Failed to get index: {}", e))?;

    let mut conflicted = Vec::new();

    // Iterate through conflicts in the index
    let mut conflict_iter = index.conflicts()
        .map_err(|e| anyhow!("Failed to get conflicts: {}", e))?;

    while let Some(conflict) = conflict_iter.next() {
        let conflict = conflict
            .map_err(|e| anyhow!("Error reading conflict: {}", e))?;

        if let Some(ours) = conflict.0 {
            conflicted.push(String::from_utf8_lossy(ours.path).to_string());
        } else if let Some(theirs) = conflict.2 {
            conflicted.push(String::from_utf8_lossy(theirs.path).to_string());
        }
    }

    Ok(conflicted)
}

/// Aborts a merge in progress
fn abort_merge(repo: &Repository) -> Result<()> {
    repo.merge_cleanup()
        .map_err(|e| anyhow!("Failed to abort merge: {}", e))?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse()?),
        )
        .init();

    info!("Starting spec reconciliation service");

    let repo_path = std::env::current_dir()?;
    let result = reconcile_specs(&repo_path).await?;

    // Exit with error code if there were errors
    if !result.errors.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_ensure_specs_main_created() {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        // Create initial commit on main
        let sig = repo.signature().unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            "HEAD",
            &sig,
            &sig,
            "Initial commit",
            &tree,
            &[],
        ).unwrap();

        // Rename current branch to main
        let head = repo.head().unwrap();
        let target = head.target().unwrap();
        repo.branch("main", &repo.find_commit(target).unwrap(), true).unwrap();
        repo.set_head("refs/heads/main").unwrap();

        // Test: ensure_specs_main_branch should create it
        assert!(ensure_specs_main_branch(&repo).is_ok());

        // Verify specs/main exists
        assert!(repo.find_branch("specs/main", git2::BranchType::Local).is_ok());
    }

    #[test]
    fn test_list_agent_branches() {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        let sig = repo.signature().unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        let commit_oid = repo.commit(
            "HEAD",
            &sig,
            &sig,
            "Initial commit",
            &tree,
            &[],
        ).unwrap();
        let commit = repo.find_commit(commit_oid).unwrap();

        // Create main branch
        repo.branch("main", &commit, true).unwrap();

        // Create agent branches
        repo.branch("specs/agent-001", &commit, false).unwrap();
        repo.branch("specs/agent-002", &commit, false).unwrap();
        repo.branch("other-branch", &commit, false).unwrap();

        // Test: list_agent_branches should only return specs/agent-*
        let branches = list_agent_branches(&repo).unwrap();
        assert_eq!(branches.len(), 2);
        assert!(branches.contains(&"specs/agent-001".to_string()));
        assert!(branches.contains(&"specs/agent-002".to_string()));
        assert!(!branches.contains(&"other-branch".to_string()));
    }
}
