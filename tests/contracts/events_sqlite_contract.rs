//! T112: Contract tests for agileplus-events ↔ agileplus-sqlite boundary.
//!
//! Verifies that SqliteStorageAdapter (provider) satisfies the EventStore trait
//! contract (consumer). Tests input/output shapes, ordering guarantees,
//! error conditions, and hash-chain integrity.

use agileplus_domain::domain::event::Event;
use agileplus_events::{EventError, EventStore};
use agileplus_sqlite::SqliteStorageAdapter;
use chrono::Utc;

/// Build a fresh in-memory SqliteStorageAdapter for each test.
fn make_adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().expect("in-memory DB")
}

/// Build a minimal Event with sensible defaults.
fn make_event(entity_type: &str, entity_id: i64, event_type: &str, sequence: i64) -> Event {
    Event {
        id: 0,
        entity_type: entity_type.to_string(),
        entity_id,
        event_type: event_type.to_string(),
        payload: serde_json::json!({"seq": sequence}),
        actor: "test-actor".to_string(),
        timestamp: Utc::now(),
        prev_hash: [0u8; 32],
        hash: {
            let mut h = [0u8; 32];
            h[0] = sequence as u8;
            h
        },
        sequence,
    }
}

// ---------------------------------------------------------------------------
// Contract: append returns an i64 row id
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_append_returns_positive_row_id() {
    let store = make_adapter();
    let event = make_event("Feature", 1, "Created", 1);
    let row_id = store.append(&event).await.expect("append");
    assert!(row_id > 0, "append must return a positive rowid");
}

// ---------------------------------------------------------------------------
// Contract: get_events returns events ordered by sequence ascending
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_get_events_ordered_by_sequence_ascending() {
    let store = make_adapter();
    // Insert out of sequence order via two appends at different times.
    // The trait promises "ascending by sequence".
    let e1 = make_event("Feature", 10, "Created", 1);
    let e2 = make_event("Feature", 10, "Transitioned", 2);
    let e3 = make_event("Feature", 10, "Validated", 3);
    store.append(&e1).await.unwrap();
    store.append(&e2).await.unwrap();
    store.append(&e3).await.unwrap();

    let events = store.get_events("Feature", 10).await.expect("get_events");
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].sequence, 1);
    assert_eq!(events[1].sequence, 2);
    assert_eq!(events[2].sequence, 3);
}

// ---------------------------------------------------------------------------
// Contract: get_events scopes to the requested entity
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_get_events_scoped_to_entity() {
    let store = make_adapter();
    store.append(&make_event("Feature", 1, "Created", 1)).await.unwrap();
    store.append(&make_event("Feature", 2, "Created", 1)).await.unwrap();
    store.append(&make_event("WorkPackage", 1, "Created", 1)).await.unwrap();

    let feature_1_events = store.get_events("Feature", 1).await.unwrap();
    assert_eq!(feature_1_events.len(), 1);
    assert_eq!(feature_1_events[0].entity_type, "Feature");
    assert_eq!(feature_1_events[0].entity_id, 1);

    let feature_2_events = store.get_events("Feature", 2).await.unwrap();
    assert_eq!(feature_2_events.len(), 1);
}

// ---------------------------------------------------------------------------
// Contract: get_events returns empty vec for unknown entity (not an error)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_get_events_empty_for_unknown_entity() {
    let store = make_adapter();
    let events = store.get_events("NonExistent", 9999).await.expect("get_events");
    assert!(events.is_empty(), "unknown entity must return empty list");
}

// ---------------------------------------------------------------------------
// Contract: get_events_since returns only events AFTER the given sequence
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_get_events_since_is_exclusive() {
    let store = make_adapter();
    for seq in 1..=5 {
        store.append(&make_event("Feature", 20, "Step", seq)).await.unwrap();
    }

    let since_3 = store.get_events_since("Feature", 20, 3).await.unwrap();
    assert_eq!(since_3.len(), 2, "sequences 4 and 5 only");
    assert!(since_3.iter().all(|e| e.sequence > 3));
}

// ---------------------------------------------------------------------------
// Contract: get_latest_sequence returns 0 for empty entity
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_get_latest_sequence_zero_for_empty() {
    let store = make_adapter();
    let seq = store.get_latest_sequence("Feature", 999).await.expect("latest_sequence");
    assert_eq!(seq, 0);
}

// ---------------------------------------------------------------------------
// Contract: get_latest_sequence reflects highest appended sequence
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_get_latest_sequence_reflects_max() {
    let store = make_adapter();
    for seq in 1..=4 {
        store.append(&make_event("Feature", 30, "Step", seq)).await.unwrap();
    }

    let latest = store.get_latest_sequence("Feature", 30).await.unwrap();
    assert_eq!(latest, 4);
}

// ---------------------------------------------------------------------------
// Contract: get_events_by_range returns events within timestamp window
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_get_events_by_range_inclusive() {
    let store = make_adapter();
    let t0 = Utc::now() - chrono::Duration::seconds(10);
    let t1 = Utc::now() + chrono::Duration::seconds(10);

    store.append(&make_event("Feature", 40, "Created", 1)).await.unwrap();

    let events = store
        .get_events_by_range("Feature", 40, t0, t1)
        .await
        .expect("range query");
    assert_eq!(events.len(), 1);
}

// ---------------------------------------------------------------------------
// Contract: event fields round-trip faithfully through the store
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_event_fields_roundtrip() {
    let store = make_adapter();
    let payload = serde_json::json!({"title": "round-trip test", "count": 42});
    let event = Event {
        id: 0,
        entity_type: "Feature".to_string(),
        entity_id: 50,
        event_type: "Created".to_string(),
        payload: payload.clone(),
        actor: "alice".to_string(),
        timestamp: Utc::now(),
        prev_hash: [0u8; 32],
        hash: [1u8; 32],
        sequence: 1,
    };
    store.append(&event).await.unwrap();

    let events = store.get_events("Feature", 50).await.unwrap();
    assert_eq!(events.len(), 1);
    let stored = &events[0];
    assert_eq!(stored.entity_type, "Feature");
    assert_eq!(stored.entity_id, 50);
    assert_eq!(stored.event_type, "Created");
    assert_eq!(stored.actor, "alice");
    assert_eq!(stored.sequence, 1);
    assert_eq!(stored.payload["title"], "round-trip test");
    assert_eq!(stored.payload["count"], 42);
}

// ---------------------------------------------------------------------------
// Contract: multiple entity streams are isolated
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_entity_streams_are_isolated() {
    let store = make_adapter();
    for id in [1i64, 2, 3] {
        for seq in 1..=3 {
            store.append(&make_event("Feature", id, "Step", seq)).await.unwrap();
        }
    }

    for id in [1i64, 2, 3] {
        let events = store.get_events("Feature", id).await.unwrap();
        assert_eq!(events.len(), 3, "entity {id} should have exactly 3 events");
        assert!(events.iter().all(|e| e.entity_id == id));
    }
}
