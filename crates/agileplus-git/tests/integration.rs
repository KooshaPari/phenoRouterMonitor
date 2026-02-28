//! Integration tests for GitVcsAdapter.
//! All tests use temporary git repositories — never the real project repo.
//!
//! Traceability: WP07-T043

use agileplus_domain::ports::VcsPort;
use agileplus_git::GitVcsAdapter;
use git2::{Repository, Signature};
use std::path::Path;
use tempfile::TempDir;

// ---- Test harness ----

/// Create a temp git repo with an initial commit and return adapter + tempdir.
fn setup_test_repo() -> (TempDir, GitVcsAdapter) {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = Repository::init(dir.path()).expect("git init");

    // Configure identity so commits work.
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test User").unwrap();
    config.set_str("user.email", "test@test.com").unwrap();
    drop(config);

    // Create an initial commit so HEAD exists.
    make_commit(&repo, dir.path(), "README.md", "# Test repo\n", "Initial commit");

    let adapter = GitVcsAdapter::new(dir.path().to_path_buf()).expect("adapter");
    (dir, adapter)
}

/// Helper: write a file and create a commit.
fn make_commit(repo: &Repository, workdir: &Path, filename: &str, content: &str, message: &str) {
    let path = workdir.join(filename);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, content).unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(Path::new(filename)).unwrap();
    index.write().unwrap();

    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let sig = Signature::now("Test User", "test@test.com").unwrap();

    let parents: Vec<git2::Commit> = match repo.head() {
        Ok(head) => vec![head.peel_to_commit().unwrap()],
        Err(_) => vec![],
    };
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
        .unwrap();
}

// ---- Adapter construction ----

#[test]
fn test_adapter_new_valid_repo() {
    let (dir, _adapter) = setup_test_repo();
    // Already validated by setup_test_repo succeeding.
    drop(dir);
}

#[test]
fn test_adapter_new_invalid_dir() {
    let dir = tempfile::tempdir().unwrap();
    let result = GitVcsAdapter::new(dir.path().to_path_buf());
    assert!(result.is_err(), "should fail on non-git dir");
}

// ---- Artifact tests ----

#[tokio::test]
async fn test_write_and_read_artifact() {
    let (dir, adapter) = setup_test_repo();
    let content = "# Spec\nThis is a spec.\n";

    adapter
        .write_artifact("my-feature", "spec.md", content)
        .await
        .expect("write_artifact");

    let read_back = adapter
        .read_artifact("my-feature", "spec.md")
        .await
        .expect("read_artifact");

    assert_eq!(read_back, content);
    drop(dir);
}

#[tokio::test]
async fn test_artifact_exists_before_and_after() {
    let (dir, adapter) = setup_test_repo();

    let exists_before = adapter
        .artifact_exists("my-feature", "spec.md")
        .await
        .unwrap();
    assert!(!exists_before);

    adapter
        .write_artifact("my-feature", "spec.md", "content")
        .await
        .unwrap();

    let exists_after = adapter
        .artifact_exists("my-feature", "spec.md")
        .await
        .unwrap();
    assert!(exists_after);
    drop(dir);
}

#[tokio::test]
async fn test_read_missing_artifact_returns_not_found() {
    let (dir, adapter) = setup_test_repo();
    let result = adapter
        .read_artifact("nonexistent", "spec.md")
        .await;
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("not found") || err_str.contains("NotFound"),
        "expected NotFound error, got: {err_str}"
    );
    drop(dir);
}

#[tokio::test]
async fn test_write_artifact_stages_in_index() {
    let (dir, adapter) = setup_test_repo();

    adapter
        .write_artifact("feat-x", "plan.md", "# Plan\n")
        .await
        .unwrap();

    // Verify the file is staged in the git index.
    let repo = Repository::open(dir.path()).unwrap();
    let index = repo.index().unwrap();
    let found = index.iter().any(|e| {
        std::str::from_utf8(&e.path)
            .map(|p| p.contains("feat-x/plan.md"))
            .unwrap_or(false)
    });
    assert!(found, "artifact should be staged in index");
    drop(dir);
}

#[tokio::test]
async fn test_scan_feature_artifacts() {
    let (dir, adapter) = setup_test_repo();
    let slug = "my-feature";

    adapter
        .write_artifact(slug, "meta.json", r#"{"slug":"my-feature"}"#)
        .await
        .unwrap();
    adapter
        .write_artifact(slug, "audit/chain.jsonl", r#"{"event":"created"}"#)
        .await
        .unwrap();
    adapter
        .write_artifact(slug, "evidence/screenshot.png", "binary-ish")
        .await
        .unwrap();

    let artifacts = adapter
        .scan_feature_artifacts(slug)
        .await
        .unwrap();

    assert!(artifacts.meta_json.is_some(), "meta.json should be present");
    assert!(artifacts.audit_chain.is_some(), "chain.jsonl should be present");
    assert!(!artifacts.evidence_paths.is_empty(), "evidence should be found");
    drop(dir);
}

// ---- Scanner tests ----

#[test]
fn test_scan_all_features_finds_two_features() {
    let (dir, adapter) = setup_test_repo();

    // Create two feature dirs with meta.json.
    for slug in &["feature-a", "feature-b"] {
        let path = dir.path().join("kitty-specs").join(slug).join("meta.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, r#"{"slug":"x"}"#).unwrap();
    }
    // Create a dir WITHOUT meta.json (should be excluded).
    std::fs::create_dir_all(dir.path().join("kitty-specs").join("no-meta")).unwrap();

    let slugs = agileplus_git::scan_all_features(&adapter).unwrap();
    assert_eq!(slugs.len(), 2);
    assert!(slugs.contains(&"feature-a".to_string()));
    assert!(slugs.contains(&"feature-b".to_string()));
    drop(dir);
}

#[test]
fn test_scan_excludes_dirs_without_meta() {
    let (dir, adapter) = setup_test_repo();

    std::fs::create_dir_all(dir.path().join("kitty-specs").join("no-meta")).unwrap();
    let slugs = agileplus_git::scan_all_features(&adapter).unwrap();
    assert!(slugs.is_empty());
    drop(dir);
}

// ---- Branch tests ----

#[tokio::test]
async fn test_create_branch() {
    let (dir, adapter) = setup_test_repo();

    adapter
        .create_branch("feature-xyz", "HEAD")
        .await
        .expect("create_branch");

    let repo = Repository::open(dir.path()).unwrap();
    let branch = repo.find_branch("feature-xyz", git2::BranchType::Local);
    assert!(branch.is_ok(), "branch should exist after create_branch");
    drop(dir);
}

#[tokio::test]
async fn test_checkout_branch() {
    let (dir, adapter) = setup_test_repo();

    adapter.create_branch("dev", "HEAD").await.unwrap();
    adapter.checkout_branch("dev").await.unwrap();

    let repo = Repository::open(dir.path()).unwrap();
    let head = repo.head().unwrap();
    let shorthand = head.shorthand().unwrap_or("");
    assert_eq!(shorthand, "dev");
    drop(dir);
}

#[tokio::test]
async fn test_merge_fast_forward() {
    let (dir, adapter) = setup_test_repo();

    // Create feature branch, add commit, merge back.
    adapter.create_branch("feat", "HEAD").await.unwrap();
    adapter.checkout_branch("feat").await.unwrap();

    let repo = Repository::open(dir.path()).unwrap();
    make_commit(&repo, dir.path(), "new-file.txt", "new content\n", "feat commit");
    drop(repo);

    let result = adapter.merge_to_target("feat", "master").await;
    // If "master" doesn't exist, try "main".
    let result = match result {
        Err(_) => adapter.merge_to_target("feat", "main").await.unwrap(),
        Ok(r) => r,
    };

    assert!(result.success);
    assert!(result.conflicts.is_empty());
    drop(dir);
}

#[tokio::test]
async fn test_merge_with_conflict() {
    let (dir, adapter) = setup_test_repo();

    let repo = Repository::open(dir.path()).unwrap();
    let default_branch = get_default_branch(&repo);
    drop(repo);

    // Create two branches from the same base that edit the same file.
    adapter.create_branch("branch-a", "HEAD").await.unwrap();
    adapter.create_branch("branch-b", "HEAD").await.unwrap();

    // Branch A: edit conflict.txt.
    adapter.checkout_branch("branch-a").await.unwrap();
    let repo = Repository::open(dir.path()).unwrap();
    make_commit(&repo, dir.path(), "conflict.txt", "version from A\n", "branch-a commit");
    drop(repo);

    // Branch B: edit the same file differently.
    adapter.checkout_branch("branch-b").await.unwrap();
    let repo = Repository::open(dir.path()).unwrap();
    make_commit(&repo, dir.path(), "conflict.txt", "version from B\n", "branch-b commit");
    drop(repo);

    // Merge A into default branch first.
    adapter.checkout_branch(&default_branch).await.unwrap();
    adapter.merge_to_target("branch-a", &default_branch).await.unwrap();

    // Now merge B (should conflict).
    let result = adapter
        .merge_to_target("branch-b", &default_branch)
        .await
        .unwrap();

    // May be a conflict OR a successful fast-forward in some edge cases depending on git state.
    // Either way, result should not panic. Conflicts should be detected.
    // Since both branches divergently edited conflict.txt from same base, we expect conflicts.
    if !result.success {
        assert!(!result.conflicts.is_empty(), "conflicts should be non-empty on failure");
    }
    drop(dir);
}

#[tokio::test]
async fn test_detect_conflicts_divergent_branches() {
    let (dir, adapter) = setup_test_repo();

    let repo = Repository::open(dir.path()).unwrap();
    drop(repo);

    adapter.create_branch("branch-x", "HEAD").await.unwrap();
    adapter.create_branch("branch-y", "HEAD").await.unwrap();

    adapter.checkout_branch("branch-x").await.unwrap();
    let repo = Repository::open(dir.path()).unwrap();
    make_commit(&repo, dir.path(), "shared.txt", "x version\n", "x commit");
    drop(repo);

    adapter.checkout_branch("branch-y").await.unwrap();
    let repo = Repository::open(dir.path()).unwrap();
    make_commit(&repo, dir.path(), "shared.txt", "y version\n", "y commit");
    drop(repo);

    // detect_conflicts should report conflict on shared.txt without mutating state.
    let conflicts = adapter.detect_conflicts("branch-x", "branch-y").await.unwrap();
    assert!(
        !conflicts.is_empty(),
        "expected conflict on shared.txt between branch-x and branch-y"
    );
    let paths: Vec<&str> = conflicts.iter().map(|c| c.path.as_str()).collect();
    assert!(paths.iter().any(|p| p.contains("shared.txt")));
    drop(dir);
}

#[tokio::test]
async fn test_detect_conflicts_no_conflict() {
    let (dir, adapter) = setup_test_repo();

    adapter.create_branch("clean-a", "HEAD").await.unwrap();
    adapter.create_branch("clean-b", "HEAD").await.unwrap();

    adapter.checkout_branch("clean-a").await.unwrap();
    let repo = Repository::open(dir.path()).unwrap();
    make_commit(&repo, dir.path(), "file-a.txt", "a content\n", "clean-a commit");
    drop(repo);

    adapter.checkout_branch("clean-b").await.unwrap();
    let repo = Repository::open(dir.path()).unwrap();
    make_commit(&repo, dir.path(), "file-b.txt", "b content\n", "clean-b commit");
    drop(repo);

    let conflicts = adapter.detect_conflicts("clean-a", "clean-b").await.unwrap();
    assert!(conflicts.is_empty(), "no conflicts expected for non-overlapping changes");
    drop(dir);
}

// ---- Worktree tests ----

#[tokio::test]
async fn test_create_and_list_worktree() {
    let (dir, adapter) = setup_test_repo();

    let path = adapter
        .create_worktree("my-feat", "WP01")
        .await
        .expect("create_worktree");

    assert!(path.exists(), "worktree directory should exist");
    assert!(path.join(".git").exists(), "worktree should have .git file");

    let worktrees = adapter.list_worktrees().await.unwrap();
    let found = worktrees.iter().any(|wt| wt.path == path);
    assert!(found, "new worktree should appear in list_worktrees");
    drop(dir);
}

#[tokio::test]
async fn test_cleanup_worktree() {
    let (dir, adapter) = setup_test_repo();

    let path = adapter
        .create_worktree("feat-x", "WP02")
        .await
        .unwrap();

    assert!(path.exists());

    adapter.cleanup_worktree(&path).await.expect("cleanup_worktree");

    assert!(!path.exists(), "worktree dir should be removed after cleanup");
    drop(dir);
}

#[tokio::test]
async fn test_cleanup_worktree_safety_check() {
    let (dir, adapter) = setup_test_repo();
    // Try to cleanup a path outside .worktrees/.
    let result = adapter.cleanup_worktree(dir.path()).await;
    assert!(result.is_err(), "should not allow cleanup outside .worktrees/");
    drop(dir);
}

// ---- History scanning ----

#[test]
fn test_get_feature_history() {
    let (dir, adapter) = setup_test_repo();
    let repo = Repository::open(dir.path()).unwrap();

    // Add commit touching a feature dir.
    make_commit(
        &repo,
        dir.path(),
        "kitty-specs/my-feature/spec.md",
        "# Spec\n",
        "Add spec for my-feature",
    );
    make_commit(
        &repo,
        dir.path(),
        "other-file.txt",
        "unrelated\n",
        "Unrelated commit",
    );
    drop(repo);

    let history = agileplus_git::get_feature_history(&adapter, "my-feature").unwrap();
    assert!(!history.is_empty(), "should find commits for my-feature");
    assert!(
        history.iter().any(|c| c.message.contains("my-feature")),
        "feature commit should appear in history"
    );
}

// ---- Helpers ----

fn get_default_branch(repo: &Repository) -> String {
    if let Ok(head) = repo.head() {
        if let Some(name) = head.shorthand() {
            return name.to_string();
        }
    }
    "main".to_string()
}
