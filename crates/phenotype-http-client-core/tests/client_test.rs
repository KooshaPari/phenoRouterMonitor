//! Comprehensive tests for HttpClient.

use phenotype_http_client_core::HttpClient;
use serde_json::{json, Value};
use std::time::Duration;

#[test]
fn test_http_client_creation() {
    let client = HttpClient::new();
    assert_eq!(client.timeout, Duration::from_secs(30));
}

#[test]
fn test_http_client_with_custom_timeout() {
    let custom_timeout = Duration::from_secs(60);
    let client = HttpClient::with_timeout(custom_timeout);
    assert_eq!(client.timeout, custom_timeout);
}

#[test]
fn test_http_client_default_impl() {
    let client1 = HttpClient::new();
    let client2 = HttpClient::default();
    assert_eq!(client1.timeout, client2.timeout);
}

#[test]
fn test_http_client_clone() {
    let client = HttpClient::new();
    let cloned = client.clone();
    assert_eq!(client.timeout, cloned.timeout);
}

#[test]
fn test_http_client_with_single_header() {
    let client = HttpClient::new().with_default_header("Authorization", "Bearer token");
    assert!(client.default_headers.contains_key("Authorization"));
}

#[test]
fn test_http_client_with_multiple_headers() {
    let client = HttpClient::new()
        .with_default_header("Authorization", "Bearer token")
        .with_default_header("User-Agent", "phenotype-client/0.2.0")
        .with_default_header("Accept", "application/json");

    assert_eq!(client.default_headers.len(), 3);
    assert_eq!(
        client.default_headers.get("Authorization"),
        Some(&"Bearer token".to_string())
    );
    assert_eq!(
        client.default_headers.get("User-Agent"),
        Some(&"phenotype-client/0.2.0".to_string())
    );
    assert_eq!(
        client.default_headers.get("Accept"),
        Some(&"application/json".to_string())
    );
}

#[test]
fn test_http_client_header_override() {
    let client = HttpClient::new()
        .with_default_header("X-Custom", "value1")
        .with_default_header("X-Custom", "value2");

    // Last value wins
    assert_eq!(
        client.default_headers.get("X-Custom"),
        Some(&"value2".to_string())
    );
}

#[test]
fn test_http_client_builder_pattern() {
    let client = HttpClient::with_timeout(Duration::from_secs(45))
        .with_default_header("X-API-Key", "secret123")
        .with_default_header("Content-Type", "application/json");

    assert_eq!(client.timeout, Duration::from_secs(45));
    assert_eq!(client.default_headers.len(), 2);
}

#[tokio::test]
async fn test_http_client_get_stub() {
    // Stub test: actual network calls would require mocking
    // In real tests, use mockito or wiremock
    let _client = HttpClient::new();

    // This is a placeholder test for compilation
    // Real implementation would use actual HTTP mock/stub
}

#[tokio::test]
async fn test_http_client_post_stub() {
    let _client = HttpClient::new();
    let _body = json!({"key": "value"});
}

#[tokio::test]
async fn test_http_client_put_stub() {
    let _client = HttpClient::new();
    let _body = json!({"id": 1, "updated": true});
}

#[tokio::test]
async fn test_http_client_delete_stub() {
    let _client = HttpClient::new();
}

#[test]
fn test_json_value_serialization() {
    let value = json!({
        "name": "test",
        "count": 42,
        "active": true
    });

    assert_eq!(value["name"], "test");
    assert_eq!(value["count"], 42);
    assert_eq!(value["active"], true);
}

#[test]
fn test_json_value_from_string() {
    let json_str = r#"{"id": 1, "name": "example"}"#;
    let value: Value = serde_json::from_str(json_str).unwrap();

    assert_eq!(value["id"], 1);
    assert_eq!(value["name"], "example");
}

#[test]
fn test_json_array_serialization() {
    let array = json!([1, 2, 3, 4, 5]);
    assert_eq!(array.as_array().map(|a| a.len()), Some(5));
}

#[test]
fn test_json_nested_object() {
    let obj = json!({
        "user": {
            "id": 123,
            "profile": {
                "name": "John Doe",
                "email": "john@example.com"
            }
        }
    });

    assert_eq!(obj["user"]["id"], 123);
    assert_eq!(obj["user"]["profile"]["name"], "John Doe");
}

#[test]
fn test_http_client_timeout_boundaries() {
    let min_timeout = HttpClient::with_timeout(Duration::from_millis(1));
    assert_eq!(min_timeout.timeout, Duration::from_millis(1));

    let max_timeout = HttpClient::with_timeout(Duration::from_secs(3600));
    assert_eq!(max_timeout.timeout, Duration::from_secs(3600));
}

#[test]
fn test_http_client_thread_safe() {
    let _client = HttpClient::new();

    // HttpClient should be Send + Sync
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<HttpClient>();
}

#[test]
fn test_http_client_debug_trait() {
    let client = HttpClient::new();
    let debug_str = format!("{:?}", client);
    assert!(!debug_str.is_empty());
}

#[test]
fn test_empty_json_object() {
    let empty = json!({});
    assert!(empty.is_object());
    assert_eq!(empty.as_object().map(|o| o.len()), Some(0));
}

#[test]
fn test_empty_json_array() {
    let empty = json!([]);
    assert!(empty.is_array());
    assert_eq!(empty.as_array().map(|a| a.len()), Some(0));
}

#[test]
fn test_json_null_value() {
    let null_val = json!(null);
    assert!(null_val.is_null());
}

#[test]
fn test_json_boolean_values() {
    let t = json!(true);
    let f = json!(false);

    assert!(t.is_boolean());
    assert!(f.is_boolean());
    assert_eq!(t.as_bool(), Some(true));
    assert_eq!(f.as_bool(), Some(false));
}

#[test]
fn test_json_number_values() {
    let int = json!(42);
    let float = json!(3.14);

    assert!(int.is_number());
    assert!(float.is_number());
}

#[test]
fn test_json_string_values() {
    let s = json!("hello");
    assert!(s.is_string());
    assert_eq!(s.as_str(), Some("hello"));
}

#[test]
fn test_json_type_conversions() {
    let value = json!({
        "int": 42,
        "float": 3.14,
        "string": "text",
        "bool": true,
        "null": null,
        "array": [1, 2, 3],
        "object": {"nested": "value"}
    });

    assert_eq!(value["int"].as_i64(), Some(42));
    assert!(value["float"].is_number());
    assert_eq!(value["string"].as_str(), Some("text"));
    assert_eq!(value["bool"].as_bool(), Some(true));
    assert!(value["null"].is_null());
    assert!(value["array"].is_array());
    assert!(value["object"].is_object());
}

#[test]
fn test_http_client_clone_with_headers() {
    let original = HttpClient::new()
        .with_default_header("X-Request-ID", "12345")
        .with_default_header("X-Trace-ID", "abc-def");

    let cloned = original.clone();

    assert_eq!(cloned.default_headers.len(), 2);
    assert_eq!(
        cloned.default_headers.get("X-Request-ID"),
        Some(&"12345".to_string())
    );
}
