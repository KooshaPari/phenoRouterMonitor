use super::harness::setup_test_repo;
use git2::Repository;

#[test]
fn test_get_feature_history() {
    let (dir, adapter) = setup_test_repo();
    let repo = Repository::open(dir.path()).unwrap();

    super::harness::make_commit(
        &repo,
        dir.path(),
        "agileplus/my-feature/spec.md",
        "# Spec\n",
        "Add spec for my-feature",
    );
    super::harness::make_commit(
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
