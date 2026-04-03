//! Integration tests for casbin backend in phenotype-policy-engine.
#![cfg(feature = "casbin-backend")]

use phenotype_policy_engine::casbin_backend::{
    evaluation_context_to_casbin_request, evaluation_context_to_casbin_requests, CasbinBackend,
    CasbinRequest, PolicyBackend,
};
use std::fs;
use tempfile::TempDir;

fn create_test_model_and_policy() -> (TempDir, String, String) {
    let dir = TempDir::new().unwrap();
    let model_path = dir.path().join("model.conf");
    let policy_path = dir.path().join("policy.csv");

    let model_content = r#"
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = r.sub == p.sub && r.obj == p.obj && r.act == p.act
"#;

    let policy_content = "p, alice, data1, read\np, bob, data1, write\n";

    fs::write(&model_path, model_content).unwrap();
    fs::write(&policy_path, policy_content).unwrap();

    (
        dir,
        model_path.to_string_lossy().to_string(),
        policy_path.to_string_lossy().to_string(),
    )
}

#[tokio::test]
async fn test_casbin_backend_enforce_allowed() {
    let (_dir, model_path, policy_path) = create_test_model_and_policy();
    let backend = CasbinBackend::new(model_path, policy_path)
        .await
        .expect("Failed to create CasbinBackend");

    let request = CasbinRequest::new("alice", "data1", "read");
    let response = backend
        .enforce(request)
        .await
        .expect("Enforce should succeed");

    assert!(response.allowed, "alice should be allowed to read data1");
}

#[tokio::test]
async fn test_casbin_backend_enforce_denied() {
    let (_dir, model_path, policy_path) = create_test_model_and_policy();
    let backend = CasbinBackend::new(model_path, policy_path)
        .await
        .expect("Failed to create CasbinBackend");

    let request = CasbinRequest::new("charlie", "data1", "read");
    let response = backend
        .enforce(request)
        .await
        .expect("Enforce should succeed");

    assert!(!response.allowed, "charlie should be denied");
}

#[tokio::test]
async fn test_casbin_backend_batch_enforce() {
    let (_dir, model_path, policy_path) = create_test_model_and_policy();
    let backend = CasbinBackend::new(model_path, policy_path)
        .await
        .expect("Failed to create CasbinBackend");

    let requests = vec![
        CasbinRequest::new("alice", "data1", "read"),
        CasbinRequest::new("bob", "data1", "write"),
        CasbinRequest::new("charlie", "data1", "read"),
    ];

    let responses = backend
        .batch_enforce(&requests)
        .await
        .expect("Batch enforce should succeed");

    assert_eq!(responses.len(), 3);
    assert!(responses[0].allowed);
    assert!(responses[1].allowed);
    assert!(!responses[2].allowed);
}

#[tokio::test]
async fn test_casbin_backend_modify_policy() {
    let (_dir, model_path, policy_path) = create_test_model_and_policy();
    let backend = CasbinBackend::new(model_path, policy_path)
        .await
        .expect("Failed to create CasbinBackend");

    let before = backend
        .enforce(CasbinRequest::new("charlie", "data1", "read"))
        .await
        .expect("Enforce should succeed");
    assert!(
        !before.allowed,
        "charlie should be denied before policy change"
    );

    let rules = vec![vec![
        "charlie".to_string(),
        "data1".to_string(),
        "read".to_string(),
    ]];
    backend
        .modify_policy("p", rules)
        .await
        .expect("Modify policy should succeed");

    let after = backend
        .enforce(CasbinRequest::new("charlie", "data1", "read"))
        .await
        .expect("Enforce should succeed");
    assert!(
        after.allowed,
        "charlie should be allowed after policy change"
    );
}

#[tokio::test]
async fn test_casbin_backend_remove_policy() {
    let (_dir, model_path, policy_path) = create_test_model_and_policy();
    let backend = CasbinBackend::new(model_path, policy_path)
        .await
        .expect("Failed to create CasbinBackend");

    let before = backend
        .enforce(CasbinRequest::new("alice", "data1", "read"))
        .await
        .expect("Enforce should succeed");
    assert!(before.allowed, "alice should be allowed before removal");

    let rules = vec![vec![
        "alice".to_string(),
        "data1".to_string(),
        "read".to_string(),
    ]];
    backend
        .remove_policy("p", rules)
        .await
        .expect("Remove policy should succeed");

    let after = backend
        .enforce(CasbinRequest::new("alice", "data1", "read"))
        .await
        .expect("Enforce should succeed");
    assert!(!after.allowed, "alice should be denied after removal");
}

#[tokio::test]
async fn test_casbin_backend_reload_policy() {
    let (_dir, model_path, policy_path) = create_test_model_and_policy();
    let backend = CasbinBackend::new(model_path, policy_path)
        .await
        .expect("Failed to create CasbinBackend");

    backend
        .reload_policy()
        .await
        .expect("Reload policy should succeed");
}

#[test]
fn test_evaluation_context_to_casbin_request() {
    use phenotype_policy_engine::EvaluationContext;

    let mut ctx = EvaluationContext::new();
    ctx.set_string("subject", "alice");
    ctx.set_string("object", "data1");
    ctx.set_string("action", "read");

    let request = evaluation_context_to_casbin_request(&ctx, "subject", "object", "action")
        .expect("Should convert successfully");

    assert_eq!(request.sub, "alice");
    assert_eq!(request.obj, "data1");
    assert_eq!(request.act, "read");
}

#[test]
fn test_evaluation_context_to_casbin_request_missing_field() {
    use phenotype_policy_engine::EvaluationContext;

    let mut ctx = EvaluationContext::new();
    ctx.set_string("subject", "alice");

    let result = evaluation_context_to_casbin_request(&ctx, "subject", "object", "action");
    assert!(result.is_err(), "Should fail when fields are missing");
}

#[test]
fn test_evaluation_context_to_casbin_requests_arrays() {
    use phenotype_policy_engine::EvaluationContext;

    let mut ctx = EvaluationContext::new();
    ctx.set("subjects", serde_json::json!(["alice", "bob"]));
    ctx.set("objects", serde_json::json!(["data1", "data2"]));
    ctx.set("actions", serde_json::json!(["read", "write"]));

    let requests = evaluation_context_to_casbin_requests(&ctx, "subjects", "objects", "actions")
        .expect("Should convert successfully");

    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].sub, "alice");
    assert_eq!(requests[0].obj, "data1");
    assert_eq!(requests[0].act, "read");
    assert_eq!(requests[1].sub, "bob");
    assert_eq!(requests[1].obj, "data2");
    assert_eq!(requests[1].act, "write");
}

#[test]
fn test_evaluation_context_to_casbin_requests_mismatched_lengths() {
    use phenotype_policy_engine::EvaluationContext;

    let mut ctx = EvaluationContext::new();
    ctx.set("subjects", serde_json::json!(["alice", "bob"]));
    ctx.set("objects", serde_json::json!(["data1"]));
    ctx.set("actions", serde_json::json!(["read", "write"]));

    let result = evaluation_context_to_casbin_requests(&ctx, "subjects", "objects", "actions");
    assert!(result.is_err(), "Should fail when array lengths mismatch");
}

#[test]
fn test_casbin_request_new() {
    let request = CasbinRequest::new("user", "resource", "action");
    assert_eq!(request.sub, "user");
    assert_eq!(request.obj, "resource");
    assert_eq!(request.act, "action");
}

#[tokio::test]
async fn test_policy_engine_with_casbin_backend_integration() {
    let (_dir, model_path, policy_path) = create_test_model_and_policy();
    let backend = CasbinBackend::new(model_path, policy_path)
        .await
        .expect("Failed to create CasbinBackend");

    fn assert_policy_backend<B: PolicyBackend>(_: &B) {}
    assert_policy_backend(&backend);

    let allowed = backend
        .enforce(CasbinRequest::new("alice", "data1", "read"))
        .await
        .expect("Should succeed");
    assert!(allowed.allowed);
}
