use phenotype_test_infra::{
    builders::StateBuilder, capture_logs, fixtures::UserBuilder, fixtures::OrderBuilder,
    mocks::{MockCache, MockEventStore, MockLogger, MockRepository}, MockClock, TempDir,
};
use serde_json::json;
use std::time::Duration;

#[test]
fn test_temp_dir_and_fixtures() {
    let tmp = TempDir::new("integration-test").expect("create temp dir");
    let path = tmp.path();
    assert!(path.exists());

    let user = UserBuilder::new("alice")
        .with_email("alice@example.com")
        .with_active(true)
        .build();

    let order = OrderBuilder::new()
        .with_user_id(user.id.clone())
        .with_amount(99.99)
        .with_status("completed")
        .build();

    assert_eq!(order.user_id, user.id);
    assert_eq!(order.amount, 99.99);
}

#[test]
fn test_mock_repository() {
    let repo = MockRepository::new();
    repo.insert("user:1", json!({"id": 1, "name": "Alice"}));
    assert_eq!(repo.len(), 1);
}

#[test]
fn test_mock_cache() {
    let cache = MockCache::new();
    cache.set("key1", json!({"value": 42}));
    assert!(cache.get("key1").is_some());
    assert_eq!(cache.hit_count(), 1);
}

#[test]
fn test_mock_event_store() {
    let store = MockEventStore::new();
    store.append(json!({"type": "UserCreated"}));
    assert_eq!(store.event_count(), 1);
}

#[test]
fn test_mock_logger() {
    let logger = MockLogger::new();
    logger.info("test message");
    assert_eq!(logger.message_count(), 1);
}

#[test]
fn test_mock_clock() {
    let clock = MockClock::new();
    assert_eq!(clock.now(), Duration::ZERO);
    clock.advance(Duration::from_secs(10));
    assert_eq!(clock.now(), Duration::from_secs(10));
}

#[test]
fn test_builder() {
    let config = StateBuilder::new(vec![])
        .with(|v| v.push("key1=value1"))
        .build();
    assert_eq!(config.len(), 1);
}

#[test]
fn test_log_capture() {
    let logs = capture_logs(|| {
        tracing::info!("test info");
    });
    assert!(logs.contains("test info"));
}

#[test]
fn test_complete_scenario() {
    let tmp = TempDir::new("scenario").expect("temp dir");
    let repo = MockRepository::new();
    let cache = MockCache::new();
    let store = MockEventStore::new();
    let logger = MockLogger::new();
    let clock = MockClock::new();

    let user = UserBuilder::new("charlie").build();
    repo.insert(format!("user:{}", user.id), json!(user));
    logger.info("created user");
    store.append(json!({"type": "UserCreated"}));
    cache.set("user:cache", json!({}));

    assert_eq!(repo.len(), 1);
    assert_eq!(logger.message_count(), 1);
    assert_eq!(store.event_count(), 1);
    assert_eq!(cache.hit_count(), 0);

    clock.advance(Duration::from_secs(60));
    assert!(tmp.path().exists());
}
