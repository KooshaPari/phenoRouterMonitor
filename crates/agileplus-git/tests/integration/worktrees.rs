use super::harness::setup_test_repo;
use agileplus_domain::ports::VcsPort;

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

    let path = adapter.create_worktree("feat-x", "WP02").await.unwrap();

    assert!(path.exists());

    adapter
        .cleanup_worktree(&path)
        .await
        .expect("cleanup_worktree");

    assert!(
        !path.exists(),
        "worktree dir should be removed after cleanup"
    );
    drop(dir);
}

#[tokio::test]
async fn test_cleanup_worktree_safety_check() {
    let (dir, adapter) = setup_test_repo();
    let result = adapter.cleanup_worktree(dir.path()).await;
    assert!(
        result.is_err(),
        "should not allow cleanup outside .worktrees/"
    );
    drop(dir);
}
