use super::harness::{get_default_branch, make_commit, setup_test_repo};
use agileplus_domain::ports::VcsPort;
use git2::Repository;

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

    adapter.create_branch("feat", "HEAD").await.unwrap();
    adapter.checkout_branch("feat").await.unwrap();

    let repo = Repository::open(dir.path()).unwrap();
    make_commit(
        &repo,
        dir.path(),
        "new-file.txt",
        "new content\n",
        "feat commit",
    );
    drop(repo);

    let result = adapter.merge_to_target("feat", "master").await;
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

    adapter.create_branch("branch-a", "HEAD").await.unwrap();
    adapter.create_branch("branch-b", "HEAD").await.unwrap();

    adapter.checkout_branch("branch-a").await.unwrap();
    let repo = Repository::open(dir.path()).unwrap();
    make_commit(
        &repo,
        dir.path(),
        "conflict.txt",
        "version from A\n",
        "branch-a commit",
    );
    drop(repo);

    adapter.checkout_branch("branch-b").await.unwrap();
    let repo = Repository::open(dir.path()).unwrap();
    make_commit(
        &repo,
        dir.path(),
        "conflict.txt",
        "version from B\n",
        "branch-b commit",
    );
    drop(repo);

    adapter.checkout_branch(&default_branch).await.unwrap();
    adapter
        .merge_to_target("branch-a", &default_branch)
        .await
        .unwrap();

    let result = adapter
        .merge_to_target("branch-b", &default_branch)
        .await
        .unwrap();

    if !result.success {
        assert!(
            !result.conflicts.is_empty(),
            "conflicts should be non-empty on failure"
        );
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

    let conflicts = adapter
        .detect_conflicts("branch-x", "branch-y")
        .await
        .unwrap();
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
    make_commit(
        &repo,
        dir.path(),
        "file-a.txt",
        "a content\n",
        "clean-a commit",
    );
    drop(repo);

    adapter.checkout_branch("clean-b").await.unwrap();
    let repo = Repository::open(dir.path()).unwrap();
    make_commit(
        &repo,
        dir.path(),
        "file-b.txt",
        "b content\n",
        "clean-b commit",
    );
    drop(repo);

    let conflicts = adapter
        .detect_conflicts("clean-a", "clean-b")
        .await
        .unwrap();
    assert!(
        conflicts.is_empty(),
        "no conflicts expected for non-overlapping changes"
    );
    drop(dir);
}
