use super::*;

#[tokio::test]
async fn feature_create_and_get_by_slug() {
    let db = make_adapter();
    let f = Feature::new("my-feat", "My Feature", [0u8; 32], None);
    let id = StoragePort::create_feature(&db, &f).await.unwrap();
    assert!(id > 0);
 
    let got = StoragePort::get_feature_by_slug(&db, "my-feat")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.id, id);
    assert_eq!(got.slug, "my-feat");
    assert_eq!(got.friendly_name, "My Feature");
    assert_eq!(got.spec_hash, [0u8; 32]);
    assert_eq!(got.state, FeatureState::Created);
}

#[tokio::test]
async fn feature_get_by_id() {
    let db = make_adapter();
    let f = Feature::new("feat-id", "Feat", [1u8; 32], None);
    let id = StoragePort::create_feature(&db, &f).await.unwrap();
    let got = StoragePort::get_feature_by_id(&db, id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.slug, "feat-id");
}
 
#[tokio::test]
async fn feature_update_state() {
    let db = make_adapter();
    let f = Feature::new("upd-feat", "Upd", [0u8; 32], None);
    let id = StoragePort::create_feature(&db, &f).await.unwrap();
 
    StoragePort::update_feature_state(&db, id, FeatureState::Specified)
        .await
        .unwrap();
    let got = StoragePort::get_feature_by_id(&db, id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.state, FeatureState::Specified);
}
 
#[tokio::test]
async fn feature_update_persists_mutable_fields() {
    let db = make_adapter();
    let mut feature = Feature::new("mut-feat", "Mutable Feature", [1u8; 32], Some("main"));
    let id = StoragePort::create_feature(&db, &feature).await.unwrap();
    feature.id = id;
    feature.friendly_name = "Renamed Feature".to_string();
    feature.target_branch = "release/stable".to_string();
    feature.spec_hash = [2u8; 32];
    feature.module_id = None;
    feature.state = FeatureState::Validated;
    feature.updated_at = chrono::Utc::now();
 
    ContentStoragePort::update_feature(&db, &feature).await.unwrap();
 
    let got = StoragePort::get_feature_by_id(&db, id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.friendly_name, "Renamed Feature");
    assert_eq!(got.target_branch, "release/stable");
    assert_eq!(got.spec_hash, [2u8; 32]);
    assert_eq!(got.module_id, None);
    assert_eq!(got.state, FeatureState::Validated);
}
 
#[tokio::test]
async fn feature_list_by_state() {
    let db = make_adapter();
    let f1 = Feature::new("f1", "F1", [0u8; 32], None);
    let f2 = Feature::new("f2", "F2", [0u8; 32], None);
    let id1 = StoragePort::create_feature(&db, &f1).await.unwrap();
    let _id2 = StoragePort::create_feature(&db, &f2).await.unwrap();
    StoragePort::update_feature_state(&db, id1, FeatureState::Specified)
        .await
        .unwrap();
 
    let specified = StoragePort::list_features_by_state(&db, FeatureState::Specified)
        .await
        .unwrap();
    assert_eq!(specified.len(), 1);
    assert_eq!(specified[0].slug, "f1");
 
    let created = StoragePort::list_features_by_state(&db, FeatureState::Created)
        .await
        .unwrap();
    assert_eq!(created.len(), 1);
    assert_eq!(created[0].slug, "f2");
}
 
#[tokio::test]
async fn feature_list_all() {
    let db = make_adapter();
    StoragePort::create_feature(&db, &Feature::new("aa", "AA", [0u8; 32], None))
        .await
        .unwrap();
    StoragePort::create_feature(&db, &Feature::new("bb", "BB", [0u8; 32], None))
        .await
        .unwrap();
    let all = StoragePort::list_all_features(&db).await.unwrap();
    assert_eq!(all.len(), 2);
}
 
#[tokio::test]
async fn feature_duplicate_slug_fails() {
    let db = make_adapter();
    let f = Feature::new("dup", "Dup", [0u8; 32], None);
    StoragePort::create_feature(&db, &f).await.unwrap();
    let result = StoragePort::create_feature(&db, &f).await;
    assert!(result.is_err());
}
 
#[tokio::test]
async fn feature_not_found_returns_none() {
    let db = make_adapter();
    let got = StoragePort::get_feature_by_slug(&db, "no-such-slug")
        .await
        .unwrap();
    assert!(got.is_none());
    let got2 = StoragePort::get_feature_by_id(&db, 9999).await.unwrap();
    assert!(got2.is_none());
}
 
#[tokio::test]
async fn wp_create_and_get() {
    let db = make_adapter();
    let feat = Feature::new("wp-feat", "WP Feat", [0u8; 32], None);
    let fid = StoragePort::create_feature(&db, &feat).await.unwrap();
 
    let mut wp = WorkPackage::new(fid, "Task A", 1, "criteria");
    wp.file_scope = vec!["src/main.rs".into()];
    let wp_id = StoragePort::create_work_package(&db, &wp).await.unwrap();
    assert!(wp_id > 0);
 
    let got = StoragePort::get_work_package(&db, wp_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.title, "Task A");
    assert_eq!(got.file_scope, vec!["src/main.rs"]);
    assert_eq!(got.state, WpState::Planned);
    assert_eq!(got.feature_id, fid);
}
 
#[tokio::test]
async fn wp_update_state() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("f", "F", [0u8; 32], None))
        .await
        .unwrap();
    let wp_id = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "T", 1, "c"))
        .await
        .unwrap();
    StoragePort::update_wp_state(&db, wp_id, WpState::Doing).await.unwrap();
    let got = StoragePort::get_work_package(&db, wp_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.state, WpState::Doing);
}
 
#[tokio::test]
async fn wp_update_persists_mutable_fields() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("f-upd", "F Upd", [0u8; 32], None))
        .await
        .unwrap();
    let mut wp = WorkPackage::new(fid, "Task", 1, "criteria");
    wp.file_scope = vec!["src/lib.rs".into()];
    let wp_id = StoragePort::create_work_package(&db, &wp).await.unwrap();
    wp.id = wp_id;
    wp.title = "Updated Task".to_string();
    wp.sequence = 4;
    wp.file_scope = vec!["src/main.rs".into(), "src/lib.rs".into()];
    wp.acceptance_criteria = "Updated criteria".to_string();
    wp.agent_id = Some("agent-1".to_string());
    wp.pr_url = Some("https://example.com/pr/7".to_string());
    wp.worktree_path = Some("/tmp/worktree".to_string());
    wp.state = WpState::Doing;
    wp.updated_at = chrono::Utc::now();
 
    ContentStoragePort::update_work_package(&db, &wp).await.unwrap();
 
    let got = StoragePort::get_work_package(&db, wp_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got.title, "Updated Task");
    assert_eq!(got.sequence, 4);
    assert_eq!(got.file_scope, vec!["src/main.rs", "src/lib.rs"]);
    assert_eq!(got.acceptance_criteria, "Updated criteria");
    assert_eq!(got.agent_id.as_deref(), Some("agent-1"));
    assert_eq!(got.pr_url.as_deref(), Some("https://example.com/pr/7"));
    assert_eq!(got.worktree_path.as_deref(), Some("/tmp/worktree"));
    assert_eq!(got.state, WpState::Doing);
}
 
#[tokio::test]
async fn wp_list_by_feature_ordered_by_sequence() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("f2", "F2", [0u8; 32], None))
        .await
        .unwrap();
    StoragePort::create_work_package(&db, &WorkPackage::new(fid, "B", 2, "c"))
        .await
        .unwrap();
    StoragePort::create_work_package(&db, &WorkPackage::new(fid, "A", 1, "c"))
        .await
        .unwrap();
 
    let wps = StoragePort::list_wps_by_feature(&db, fid).await.unwrap();
    assert_eq!(wps.len(), 2);
    assert_eq!(wps[0].sequence, 1);
    assert_eq!(wps[1].sequence, 2);
}
 
#[tokio::test]
async fn wp_dependencies_and_ready_wps() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("f3", "F3", [0u8; 32], None))
        .await
        .unwrap();
    let wp1 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP1", 1, "c"))
        .await
        .unwrap();
    let wp2 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP2", 2, "c"))
        .await
        .unwrap();
 
    StoragePort::add_wp_dependency(&db, &WpDependency {
        wp_id: wp2,
        depends_on: wp1,
        dep_type: DependencyType::Explicit,
    })
    .await
    .unwrap();
 
    let ready = StoragePort::get_ready_wps(&db, fid).await.unwrap();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, wp1);
 
    StoragePort::update_wp_state(&db, wp1, WpState::Doing).await.unwrap();
    StoragePort::update_wp_state(&db, wp1, WpState::Review).await.unwrap();
    StoragePort::update_wp_state(&db, wp1, WpState::Done).await.unwrap();
 
    let ready = StoragePort::get_ready_wps(&db, fid).await.unwrap();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, wp2);
}
 
#[tokio::test]
async fn wp_get_dependencies() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("fd", "FD", [0u8; 32], None))
        .await
        .unwrap();
    let w1 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "W1", 1, "c"))
        .await
        .unwrap();
    let w2 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "W2", 2, "c"))
        .await
        .unwrap();
    StoragePort::add_wp_dependency(&db, &WpDependency {
        wp_id: w2,
        depends_on: w1,
        dep_type: DependencyType::Data,
    })
    .await
    .unwrap();
    let deps = StoragePort::get_wp_dependencies(&db, w2).await.unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].depends_on, w1);
}
 
#[tokio::test]
async fn audit_append_and_trail() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("af", "AF", [0u8; 32], None))
        .await
        .unwrap();
 
    let e1 = make_audit_entry(fid, [0u8; 32]);
    let _id1 = StoragePort::append_audit_entry(&db, &e1).await.unwrap();
 
    let e2 = make_audit_entry(fid, e1.hash);
    let _id2 = StoragePort::append_audit_entry(&db, &e2).await.unwrap();
 
    let e3 = make_audit_entry(fid, e2.hash);
    StoragePort::append_audit_entry(&db, &e3).await.unwrap();
 
    let trail = StoragePort::get_audit_trail(&db, fid).await.unwrap();
    assert_eq!(trail.len(), 3);
    assert!(trail[0].id <= trail[1].id);
    assert!(trail[1].id <= trail[2].id);
}
 
#[tokio::test]
async fn audit_wrong_prev_hash_rejected() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("afc", "AFC", [0u8; 32], None))
        .await
        .unwrap();
 
    let e1 = make_audit_entry(fid, [0u8; 32]);
    StoragePort::append_audit_entry(&db, &e1).await.unwrap();
 
    let e_bad = make_audit_entry(fid, [0xFFu8; 32]);
    let result = StoragePort::append_audit_entry(&db, &e_bad).await;
    assert!(result.is_err());
}
 
#[tokio::test]
async fn audit_get_latest() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("al", "AL", [0u8; 32], None))
        .await
        .unwrap();
 
    assert!(StoragePort::get_latest_audit_entry(&db, fid).await.unwrap().is_none());
 
    let e1 = make_audit_entry(fid, [0u8; 32]);
    StoragePort::append_audit_entry(&db, &e1).await.unwrap();
 
    let e2 = make_audit_entry(fid, e1.hash);
    StoragePort::append_audit_entry(&db, &e2).await.unwrap();
 
    let latest = StoragePort::get_latest_audit_entry(&db, fid).await.unwrap().unwrap();
    assert_eq!(latest.hash, e2.hash);
}
 
#[tokio::test]
async fn evidence_create_and_get_by_wp() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("ef", "EF", [0u8; 32], None))
        .await
        .unwrap();
    let wp_id = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP", 1, "c"))
        .await
        .unwrap();
 
    let ev = Evidence {
        id: 0,
        wp_id,
        fr_id: "FR-001".into(),
        evidence_type: EvidenceType::TestResult,
        artifact_path: "results/test.xml".into(),
        metadata: Some(serde_json::json!({"pass": 42})),
        created_at: chrono::Utc::now(),
    };
    let ev_id = StoragePort::create_evidence(&db, &ev).await.unwrap();
    assert!(ev_id > 0);
 
    let evs = StoragePort::get_evidence_by_wp(&db, wp_id).await.unwrap();
    assert_eq!(evs.len(), 1);
    assert_eq!(evs[0].fr_id, "FR-001");
    assert_eq!(evs[0].evidence_type, EvidenceType::TestResult);
    assert!(evs[0].metadata.is_some());
}
 
#[tokio::test]
async fn evidence_get_by_fr() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("efr", "EFR", [0u8; 32], None))
        .await
        .unwrap();
    let wp_id = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP", 1, "c"))
        .await
        .unwrap();
 
    let ev1 = Evidence {
        id: 0,
        wp_id,
        fr_id: "FR-001".into(),
        evidence_type: EvidenceType::CiOutput,
        artifact_path: "ci.log".into(),
        metadata: None,
        created_at: chrono::Utc::now(),
    };
    let ev2 = Evidence {
        id: 0,
        wp_id,
        fr_id: "FR-002".into(),
        evidence_type: EvidenceType::LintResult,
        artifact_path: "lint.log".into(),
        metadata: None,
        created_at: chrono::Utc::now(),
    };
    StoragePort::create_evidence(&db, &ev1).await.unwrap();
    StoragePort::create_evidence(&db, &ev2).await.unwrap();
 
    let fr1 = StoragePort::get_evidence_by_fr(&db, "FR-001").await.unwrap();
    assert_eq!(fr1.len(), 1);
    assert_eq!(fr1[0].fr_id, "FR-001");
}
