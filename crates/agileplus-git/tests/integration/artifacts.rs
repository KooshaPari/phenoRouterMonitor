use super::harness::setup_test_repo;
use agileplus_domain::ports::VcsPort;
use git2::Repository;

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
    let result = adapter.read_artifact("nonexistent", "spec.md").await;
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

    let artifacts = adapter.scan_feature_artifacts(slug).await.unwrap();

    assert!(artifacts.meta_json.is_some(), "meta.json should be present");
    assert!(
        artifacts.audit_chain.is_some(),
        "chain.jsonl should be present"
    );
    assert!(
        !artifacts.evidence_paths.is_empty(),
        "evidence should be found"
    );
    drop(dir);
}

#[test]
fn test_scan_all_features_finds_two_features() {
    let (dir, adapter) = setup_test_repo();

    for slug in &["feature-a", "feature-b"] {
        let path = dir.path().join("agileplus").join(slug).join("meta.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, r#"{"slug":"x"}"#).unwrap();
    }
    std::fs::create_dir_all(dir.path().join("agileplus").join("no-meta")).unwrap();

    let slugs = agileplus_git::scan_all_features(&adapter).unwrap();
    assert_eq!(slugs.len(), 2);
    assert!(slugs.contains(&"feature-a".to_string()));
    assert!(slugs.contains(&"feature-b".to_string()));
    drop(dir);
}

#[test]
fn test_scan_excludes_dirs_without_meta() {
    let (dir, adapter) = setup_test_repo();

    std::fs::create_dir_all(dir.path().join("agileplus").join("no-meta")).unwrap();
    let slugs = agileplus_git::scan_all_features(&adapter).unwrap();
    assert!(slugs.is_empty());
    drop(dir);
}
