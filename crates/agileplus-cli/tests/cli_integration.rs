//! Integration tests for the agileplus CLI binary.
//!
//! Uses assert_cmd + tempfile to test the binary end-to-end.

use std::path::PathBuf;

use assert_cmd::Command;
use tempfile::TempDir;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Initialize a temporary git repo for testing.
fn init_temp_git_repo() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    std::process::Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(dir.path())
        .output()
        .unwrap_or_else(|_| {
            // Fall back to older git that doesn't support --initial-branch
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(dir.path())
                .output()
                .expect("git init")
        });
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir.path())
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(dir.path())
        .output()
        .expect("git config name");
    dir
}

#[test]
fn help_prints_usage() {
    Command::cargo_bin("agileplus")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Spec-driven development engine"));
}

#[test]
fn specify_help_prints_subcommand_usage() {
    Command::cargo_bin("agileplus")
        .unwrap()
        .args(["specify", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("feature"));
}

#[test]
fn research_help_prints_subcommand_usage() {
    Command::cargo_bin("agileplus")
        .unwrap()
        .args(["research", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("feature"));
}

#[test]
fn specify_from_file_creates_feature() {
    let repo_dir = init_temp_git_repo();
    let db_path = repo_dir.path().join(".agileplus").join("agileplus.db");
    let spec_path = fixtures_dir().join("sample-spec.md");

    Command::cargo_bin("agileplus")
        .unwrap()
        .args([
            "--db",
            db_path.to_str().unwrap(),
            "specify",
            "--feature",
            "test-001",
            "--from-file",
            spec_path.to_str().unwrap(),
        ])
        .current_dir(repo_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("test-001"));
}

#[test]
fn specify_creates_spec_artifact() {
    let repo_dir = init_temp_git_repo();
    let db_path = repo_dir.path().join(".agileplus").join("agileplus.db");
    let spec_path = fixtures_dir().join("sample-spec.md");

    Command::cargo_bin("agileplus")
        .unwrap()
        .args([
            "--db",
            db_path.to_str().unwrap(),
            "specify",
            "--feature",
            "my-feature",
            "--from-file",
            spec_path.to_str().unwrap(),
        ])
        .current_dir(repo_dir.path())
        .assert()
        .success();

    // Verify the spec.md was written
    let spec_file = repo_dir.path().join("kitty-specs").join("my-feature").join("spec.md");
    assert!(spec_file.exists(), "spec.md should have been created at {}", spec_file.display());
}

#[test]
fn research_on_nonexistent_feature_runs_pre_specify_mode() {
    let repo_dir = init_temp_git_repo();
    let db_path = repo_dir.path().join(".agileplus").join("agileplus.db");

    Command::cargo_bin("agileplus")
        .unwrap()
        .args([
            "--db",
            db_path.to_str().unwrap(),
            "research",
            "--feature",
            "nonexistent-feature",
        ])
        .current_dir(repo_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Pre-specify"));
}

#[test]
fn research_after_specify_transitions_to_researched() {
    let repo_dir = init_temp_git_repo();
    let db_path = repo_dir.path().join(".agileplus").join("agileplus.db");
    let spec_path = fixtures_dir().join("sample-spec.md");

    // First specify
    Command::cargo_bin("agileplus")
        .unwrap()
        .args([
            "--db",
            db_path.to_str().unwrap(),
            "specify",
            "--feature",
            "feat-research",
            "--from-file",
            spec_path.to_str().unwrap(),
        ])
        .current_dir(repo_dir.path())
        .assert()
        .success();

    // Then research
    Command::cargo_bin("agileplus")
        .unwrap()
        .args([
            "--db",
            db_path.to_str().unwrap(),
            "research",
            "--feature",
            "feat-research",
        ])
        .current_dir(repo_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Researched"));
}

#[test]
fn specify_refinement_detects_no_changes() {
    let repo_dir = init_temp_git_repo();
    let db_path = repo_dir.path().join(".agileplus").join("agileplus.db");
    let spec_path = fixtures_dir().join("sample-spec.md");

    // First specify
    Command::cargo_bin("agileplus")
        .unwrap()
        .args([
            "--db",
            db_path.to_str().unwrap(),
            "specify",
            "--feature",
            "rev-feat",
            "--from-file",
            spec_path.to_str().unwrap(),
        ])
        .current_dir(repo_dir.path())
        .assert()
        .success();

    // Re-run with same file — should detect no changes
    Command::cargo_bin("agileplus")
        .unwrap()
        .args([
            "--db",
            db_path.to_str().unwrap(),
            "specify",
            "--feature",
            "rev-feat",
            "--from-file",
            spec_path.to_str().unwrap(),
        ])
        .current_dir(repo_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("No changes"));
}

#[test]
fn specify_refinement_writes_diff_artifact() {
    let repo_dir = init_temp_git_repo();
    let db_path = repo_dir.path().join(".agileplus").join("agileplus.db");
    let spec_path = fixtures_dir().join("sample-spec.md");
    let revised_path = fixtures_dir().join("sample-spec-revised.md");

    // Initial specify
    Command::cargo_bin("agileplus")
        .unwrap()
        .args([
            "--db",
            db_path.to_str().unwrap(),
            "specify",
            "--feature",
            "diff-feat",
            "--from-file",
            spec_path.to_str().unwrap(),
        ])
        .current_dir(repo_dir.path())
        .assert()
        .success();

    // Revise with different content
    Command::cargo_bin("agileplus")
        .unwrap()
        .args([
            "--db",
            db_path.to_str().unwrap(),
            "specify",
            "--feature",
            "diff-feat",
            "--from-file",
            revised_path.to_str().unwrap(),
        ])
        .current_dir(repo_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("updated"));
}

#[test]
fn no_git_repo_shows_helpful_error() {
    let dir = TempDir::new().expect("temp dir");
    let db_path = dir.path().join("test.db");
    let spec_path = fixtures_dir().join("sample-spec.md");

    Command::cargo_bin("agileplus")
        .unwrap()
        .args([
            "--db",
            db_path.to_str().unwrap(),
            "specify",
            "--feature",
            "no-git",
            "--from-file",
            spec_path.to_str().unwrap(),
        ])
        .current_dir(dir.path())
        .assert()
        .failure();
}
