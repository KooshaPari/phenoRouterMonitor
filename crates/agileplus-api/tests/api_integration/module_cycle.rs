use agileplus_domain::domain::cycle::{Cycle, CycleFeature, CycleState};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::module::Module;
use agileplus_domain::domain::state_machine::FeatureState;
use axum::http::StatusCode;
use chrono::NaiveDate;

use crate::support::{
    MockStorage, TEST_API_KEY, setup_test_server, setup_test_server_with_storage,
};

fn seeded_mutation_storage() -> MockStorage {
    let storage = MockStorage::default();
    let now = chrono::Utc::now();
    let mut modules = storage.modules.lock().expect("modules lock poisoned");
    modules.push(Module {
        id: 1,
        slug: "platform".to_string(),
        friendly_name: "Platform".to_string(),
        description: Some("Original module".to_string()),
        parent_module_id: None,
        created_at: now,
        updated_at: now,
    });
    drop(modules);

    let mut cycles = storage.cycles.lock().expect("cycles lock poisoned");
    cycles.push(Cycle {
        id: 1,
        name: "Q1".to_string(),
        description: Some("Original cycle".to_string()),
        state: CycleState::Draft,
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
        end_date: NaiveDate::from_ymd_opt(2026, 3, 31).expect("valid date"),
        module_scope_id: None,
        created_at: now,
        updated_at: now,
    });
    drop(cycles);
    storage
}

async fn setup_mutation_server() -> axum_test::TestServer {
    setup_test_server_with_storage(seeded_mutation_storage()).await
}

fn seeded_shipping_gate_storage() -> MockStorage {
    let storage = MockStorage::default();
    let now = chrono::Utc::now();

    storage
        .features
        .lock()
        .expect("features lock poisoned")
        .push(Feature {
            id: 1,
            slug: "blocking-feature".to_string(),
            friendly_name: "Blocking Feature".to_string(),
            state: FeatureState::Implementing,
            spec_hash: [0u8; 32],
            target_branch: "main".to_string(),
            plane_issue_id: None,
            plane_state_id: None,
            labels: vec![],
            module_id: None,
            project_id: None,
            created_at: now,
            updated_at: now,
        });

    storage
        .cycles
        .lock()
        .expect("cycles lock poisoned")
        .push(Cycle {
            id: 1,
            name: "Q2".to_string(),
            description: Some("Shipping gate".to_string()),
            state: CycleState::Review,
            start_date: NaiveDate::from_ymd_opt(2026, 4, 1).expect("valid date"),
            end_date: NaiveDate::from_ymd_opt(2026, 6, 30).expect("valid date"),
            module_scope_id: None,
            created_at: now,
            updated_at: now,
        });

    storage
        .cycle_features
        .lock()
        .expect("cycle_features lock poisoned")
        .push(CycleFeature::new(1, 1));

    storage
}

async fn setup_shipping_gate_server() -> axum_test::TestServer {
    setup_test_server_with_storage(seeded_shipping_gate_storage()).await
}

#[tokio::test]
async fn module_routes_require_auth() {
    let server = setup_test_server().await;
    let resp = server.get("/api/modules").await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_modules_with_valid_key_returns_empty_array() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/modules")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    let arr = body
        .as_array()
        .expect("modules response should be an array");
    assert!(arr.is_empty());
}

#[tokio::test]
async fn get_module_not_found_returns_404() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/modules/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn cycle_routes_require_auth() {
    let server = setup_test_server().await;
    let resp = server.get("/api/cycles").await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_cycles_with_valid_key_returns_empty_array() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/cycles")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    let arr = body.as_array().expect("cycles response should be an array");
    assert!(arr.is_empty());
}

#[tokio::test]
async fn get_cycle_not_found_returns_404() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/cycles/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_cycles_invalid_state_returns_400() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/cycles?state=NotAState")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn patch_module_persists_mutations() {
    let server = setup_mutation_server().await;
    let resp = server
        .patch("/api/modules/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({
            "friendly_name": "Platform Core",
            "description": "Renamed module"
        }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["friendly_name"], "Platform Core");
    assert_eq!(body["slug"], "platform-core");
    assert_eq!(body["description"], "Renamed module");
}

#[tokio::test]
async fn delete_module_removes_record() {
    let server = setup_mutation_server().await;
    let resp = server
        .delete("/api/modules/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status(StatusCode::NO_CONTENT);

    let follow_up = server
        .get("/api/modules/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    follow_up.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn transition_cycle_persists_mutation() {
    let server = setup_mutation_server().await;
    let resp = server
        .post("/api/cycles/1/transition")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({
            "state": "Active"
        }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["state"], "Active");

    let follow_up = server
        .get("/api/cycles/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    follow_up.assert_status_ok();
    let cycle_body: serde_json::Value = follow_up.json();
    assert_eq!(cycle_body["cycle"]["state"], "Active");
}

#[tokio::test]
async fn transition_cycle_to_shipped_blocks_unvalidated_features() {
    let server = setup_shipping_gate_server().await;
    let resp = server
        .post("/api/cycles/1/transition")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({
            "state": "Shipped"
        }))
        .await;
    resp.assert_status(StatusCode::CONFLICT);
}
