use super::*;

#[tokio::test]
async fn policy_create_and_list_active() {
    let db = make_adapter();
    let rule = PolicyRule {
        id: 0,
        domain: PolicyDomain::Quality,
        rule: PolicyDefinition {
            description: "All tests must pass".into(),
            check: PolicyCheck::ManualApproval,
        },
        active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let id = StoragePort::create_policy_rule(&db, &rule).await.unwrap();
    assert!(id > 0);
 
    let active = StoragePort::list_active_policies(&db).await.unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].domain, PolicyDomain::Quality);
}
 
#[tokio::test]
async fn governance_contract_create_and_get() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("gc", "GC", [0u8; 32], None))
        .await
        .unwrap();
 
    let contract = GovernanceContract {
        id: 0,
        feature_id: fid,
        version: 1,
        rules: vec![GovernanceRule {
            transition: "created->specified".into(),
            required_evidence: vec![],
            policy_refs: vec![],
        }],
        bound_at: chrono::Utc::now(),
    };
    let cid = StoragePort::create_governance_contract(&db, &contract).await.unwrap();
    assert!(cid > 0);
 
    let got = StoragePort::get_governance_contract(&db, fid, 1).await.unwrap().unwrap();
    assert_eq!(got.feature_id, fid);
    assert_eq!(got.version, 1);
    assert_eq!(got.rules.len(), 1);
 
    let latest = StoragePort::get_latest_governance_contract(&db, fid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest.version, 1);
}
 
#[tokio::test]
async fn governance_contract_versioning() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("gcv", "GCV", [0u8; 32], None))
        .await
        .unwrap();
 
    let c1 = GovernanceContract {
        id: 0,
        feature_id: fid,
        version: 1,
        rules: vec![],
        bound_at: chrono::Utc::now(),
    };
    let c2 = GovernanceContract {
        id: 0,
        feature_id: fid,
        version: 2,
        rules: vec![],
        bound_at: chrono::Utc::now(),
    };
    StoragePort::create_governance_contract(&db, &c1).await.unwrap();
    StoragePort::create_governance_contract(&db, &c2).await.unwrap();
 
    let latest = StoragePort::get_latest_governance_contract(&db, fid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest.version, 2);
 
    let v1 = StoragePort::get_governance_contract(&db, fid, 1).await.unwrap().unwrap();
    assert_eq!(v1.version, 1);
}
 
#[tokio::test]
async fn metric_record_and_get() {
    let db = make_adapter();
    let fid = StoragePort::create_feature(&db, &Feature::new("mf", "MF", [0u8; 32], None))
        .await
        .unwrap();
 
    let m = Metric {
        id: 0,
        feature_id: Some(fid),
        command: "agileplus implement".into(),
        duration_ms: 1234,
        agent_runs: 3,
        review_cycles: 1,
        metadata: Some(serde_json::json!({"model": "claude"})),
        timestamp: chrono::Utc::now(),
    };
    let mid = StoragePort::record_metric(&db, &m).await.unwrap();
    assert!(mid > 0);
 
    let ms = StoragePort::get_metrics_by_feature(&db, fid).await.unwrap();
    assert_eq!(ms.len(), 1);
    assert_eq!(ms[0].command, "agileplus implement");
    assert_eq!(ms[0].duration_ms, 1234);
    assert!(ms[0].metadata.is_some());
}
