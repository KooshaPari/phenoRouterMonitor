use git2::{Repository, Signature};
use std::path::Path;
use tempfile::TempDir;

use agileplus_git::GitVcsAdapter;

/// Create a temp git repo with an initial commit and return adapter + tempdir.
pub(crate) fn setup_test_repo() -> (TempDir, GitVcsAdapter) {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = Repository::init(dir.path()).expect("git init");

    // Configure identity so commits work.
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test User").unwrap();
    config.set_str("user.email", "test@test.com").unwrap();
    drop(config);

    // Create an initial commit so HEAD exists.
    make_commit(
        &repo,
        dir.path(),
        "README.md",
        "# Test repo\n",
        "Initial commit",
    );

    let adapter = GitVcsAdapter::new(dir.path().to_path_buf()).expect("adapter");
    (dir, adapter)
}

/// Helper: write a file and create a commit.
pub(crate) fn make_commit(
    repo: &Repository,
    workdir: &Path,
    filename: &str,
    content: &str,
    message: &str,
) {
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

pub(crate) fn get_default_branch(repo: &Repository) -> String {
    if let Ok(head) = repo.head() {
        if let Some(name) = head.shorthand() {
            return name.to_string();
        }
    }
    "main".to_string()
}
