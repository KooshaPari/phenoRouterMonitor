use super::rate_limit::TokenBucket;
use super::*;

#[test]
fn token_bucket_basic() {
    let mut bucket = TokenBucket::new(5.0, 1.0);
    assert!(bucket.try_acquire());
    assert!(bucket.try_acquire());
}

#[test]
fn token_bucket_exhaustion() {
    let mut bucket = TokenBucket::new(2.0, 0.1);
    assert!(bucket.try_acquire());
    assert!(bucket.try_acquire());
    assert!(!bucket.try_acquire()); // exhausted
}

#[tokio::test]
async fn create_module_sends_post() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "mod-uuid-1",
            "name": "Auth",
            "description": null
        })))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let req = PlaneCreateModuleRequest {
        name: "Auth".to_string(),
        description: None,
    };
    let resp = client.create_module(&req).await.unwrap();
    assert_eq!(resp.id, "mod-uuid-1");
    assert_eq!(resp.name, "Auth");
}

#[tokio::test]
async fn create_module_http_error_propagates() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let req = PlaneCreateModuleRequest {
        name: "Fail".to_string(),
        description: None,
    };
    let result = client.create_module(&req).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("500"), "expected 500 in error: {err_msg}");
}

#[tokio::test]
async fn create_cycle_sends_correct_dates() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/cycles/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "cyc-uuid-1",
            "name": "Sprint 1",
            "start_date": "2026-01-01",
            "end_date": "2026-01-14"
        })))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let req = PlaneCreateCycleRequest {
        name: "Sprint 1".to_string(),
        description: None,
        start_date: "2026-01-01".to_string(),
        end_date: "2026-01-14".to_string(),
    };
    let resp = client.create_cycle(&req).await.unwrap();
    assert_eq!(resp.id, "cyc-uuid-1");
}

#[tokio::test]
async fn add_work_item_to_cycle_sends_post() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/workspaces/ws/projects/proj/cycles/cyc-1/cycle-issues/",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let result = client.add_work_item_to_cycle("cyc-1", "wi-1").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_work_item_uses_work_items_root() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "wi-1",
            "name": "Feature",
            "description_html": null,
            "state": null,
            "updated_at": null
        })))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let work_item = PlaneWorkItem {
        id: None,
        name: "Feature".to_string(),
        description_html: None,
        state: None,
        priority: Some(2),
        parent: None,
        labels: vec![],
    };
    let resp = client.create_work_item(&work_item).await.unwrap();
    assert_eq!(resp.id, "wi-1");
}

#[tokio::test]
async fn update_work_item_uses_work_items_root() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/wi-1/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "wi-1",
            "name": "Feature",
            "description_html": null,
            "state": null,
            "updated_at": null
        })))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let work_item = PlaneWorkItem {
        id: None,
        name: "Feature".to_string(),
        description_html: None,
        state: None,
        priority: Some(2),
        parent: None,
        labels: vec![],
    };
    let resp = client.update_work_item("wi-1", &work_item).await.unwrap();
    assert_eq!(resp.id, "wi-1");
}

#[tokio::test]
async fn get_work_item_uses_work_items_root() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/wi-1/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "wi-1",
            "name": "Feature",
            "description_html": null,
            "state": null,
            "updated_at": null
        })))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let resp = client.get_work_item("wi-1").await.unwrap();
    assert_eq!(resp.id, "wi-1");
}

#[tokio::test]
async fn delete_work_item_from_cycle_sends_delete() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(
            "/api/v1/workspaces/ws/projects/proj/cycles/cyc-1/cycle-issues/wi-1/",
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let result = client.delete_work_item_from_cycle("cyc-1", "wi-1").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn add_work_item_to_module_sends_post() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/workspaces/ws/projects/proj/modules/mod-1/module-issues/",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let result = client.add_work_item_to_module("mod-1", "wi-1").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_work_item_from_module_sends_delete() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(
            "/api/v1/workspaces/ws/projects/proj/modules/mod-1/module-issues/wi-1/",
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let result = client.delete_work_item_from_module("mod-1", "wi-1").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_module_sends_delete() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/mod-1/"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let result = client.delete_module("mod-1").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_cycle_sends_delete() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/workspaces/ws/projects/proj/cycles/cyc-1/"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let result = client.delete_cycle("cyc-1").await;
    assert!(result.is_ok());
}

#[test]
fn plane_work_item_serialize() {
    let work_item = PlaneWorkItem {
        id: None,
        name: "Test work item".to_string(),
        description_html: Some("<p>desc</p>".to_string()),
        state: None,
        priority: Some(2),
        parent: None,
        labels: vec!["agileplus".to_string()],
    };
    let json = serde_json::to_string(&work_item).unwrap();
    assert!(json.contains("Test work item"));
}
