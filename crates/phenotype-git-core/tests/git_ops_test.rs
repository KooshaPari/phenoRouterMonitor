//! Integration tests for GitRepository and GitCommit.
//! Traces to: FR-GIT-001

use phenotype_git_core::{GitCommit, GitRepository};
use std::path::Path;

/// Find the repo root by searching upward from CARGO_MANIFEST_DIR.
fn find_repo_root() -> std::path::PathBuf {
    let mut dir =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    loop {
        if dir.join(".git").exists() {
            return dir;
        }
        if !dir.pop() {
            panic!("could not find git repo root");
        }
    }
}

#[test]
fn test_git_repository_open() {
    let root = find_repo_root();
    let repo = GitRepository::open(&root);
    assert!(repo.is_ok(), "Should successfully open the repo");
}

#[test]
fn test_git_repository_open_invalid() {
    let result = GitRepository::open(Path::new("/nonexistent-path-12345"));
    assert!(result.is_err(), "Should fail to open non-existent path");
}

#[test]
fn test_git_repository_is_bare() {
    let root = find_repo_root();
    let repo = GitRepository::open(&root).expect("Failed to open repo");
    assert!(!repo.is_bare(), "This repo should not be bare");
}

#[test]
fn test_git_repository_head_commit() {
    let root = find_repo_root();
    let repo = GitRepository::open(&root).expect("Failed to open repo");
    let commit = repo.head_commit().expect("Failed to get head commit");

    assert!(
        commit.is_some(),
        "Repository should have at least one commit"
    );

    if let Some(commit) = commit {
        assert!(!commit.id.is_empty(), "Commit ID should not be empty");
        assert_eq!(commit.id.len(), 8, "Commit ID should be 8 characters");
    }
}

#[test]
fn test_git_commit_new() {
    let commit = GitCommit::new("abc12345", "feat: add new feature");
    assert_eq!(commit.id, "abc12345");
    assert_eq!(commit.message, "feat: add new feature");
}

#[test]
fn test_git_commit_clone() {
    let commit1 = GitCommit::new("abc12345", "fix: bug fix");
    let commit2 = commit1.clone();

    assert_eq!(commit1.id, commit2.id);
    assert_eq!(commit1.message, commit2.message);
}

#[test]
fn test_git_commit_debug() {
    let commit = GitCommit::new("abc12345", "docs: update");
    let debug_str = format!("{:?}", commit);
    assert!(debug_str.contains("abc12345"));
    assert!(debug_str.contains("docs: update"));
}
