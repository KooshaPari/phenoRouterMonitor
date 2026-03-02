//! T115: Contract tests for agileplus-api ↔ agileplus-events boundary.
//!
//! Verifies the contract between API event query endpoints (consumer) and
//! the EventStore trait + EventQuery (provider). Tests pagination shape,
//! filter application, field completeness, and ordering guarantees that
//! the API relies on.

use agileplus_domain::domain::event::Event;
use agileplus_events::query::EventQuery;
use chrono::Utc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_event(seq: i64, entity_type: &str, entity_id: i64, event_type: &str) -> Event {
    Event {
        id: seq,
        entity_type: entity_type.to_string(),
        entity_id,
        event_type: event_type.to_string(),
        payload: serde_json::json!({"seq": seq}),
        actor: "system".to_string(),
        timestamp: Utc::now(),
        prev_hash: [0u8; 32],
        hash: {
            let mut h = [0u8; 32];
            h[0] = seq as u8;
            h
        },
        sequence: seq,
    }
}

// ---------------------------------------------------------------------------
// Contract: Event fields are complete (API can map to response)
// ---------------------------------------------------------------------------

#[test]
fn contract_event_has_all_api_required_fields() {
    let event = make_event(1, "Feature", 10, "Created");

    // API response contract: these fields must be accessible.
    assert!(!event.entity_type.is_empty());
    assert!(event.entity_id > 0);
    assert!(!event.event_type.is_empty());
    assert!(event.sequence > 0);
    assert!(!event.actor.is_empty());
    // timestamp is DateTime<Utc> — always present
    let _ts_str = event.timestamp.to_rfc3339();
}

#[test]
fn contract_event_payload_is_json_value() {
    let event = make_event(1, "Feature", 1, "Transitioned");
    // API serializes payload as JSON object — must be a Value.
    let serialized = serde_json::to_value(&event.payload).expect("payload is JSON");
    assert!(serialized.is_object() || serialized.is_null() || serialized.is_array());
}

// ---------------------------------------------------------------------------
// Contract: EventQuery filters — entity_type
// ---------------------------------------------------------------------------

#[test]
fn contract_query_filter_by_entity_type_isolates_stream() {
    let events = vec![
        make_event(1, "Feature", 1, "Created"),
        make_event(2, "WorkPackage", 1, "Created"),
        make_event(3, "Feature", 2, "Created"),
    ];

    let result = EventQuery::new()
        .entity_type("Feature")
        .filter(&events);

    assert_eq!(result.len(), 2, "must return only Feature events");
    assert!(result.iter().all(|e| e.entity_type == "Feature"));
}

// ---------------------------------------------------------------------------
// Contract: EventQuery filters — entity_id
// ---------------------------------------------------------------------------

#[test]
fn contract_query_filter_by_entity_id() {
    let events = vec![
        make_event(1, "Feature", 1, "Created"),
        make_event(2, "Feature", 2, "Created"),
        make_event(3, "Feature", 1, "Transitioned"),
    ];

    let result = EventQuery::new()
        .entity_type("Feature")
        .entity_id(1)
        .filter(&events);

    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|e| e.entity_id == 1));
}

// ---------------------------------------------------------------------------
// Contract: EventQuery pagination — limit
// ---------------------------------------------------------------------------

#[test]
fn contract_query_limit_matches_api_page_size() {
    let events: Vec<Event> = (1..=25).map(|seq| make_event(seq, "Feature", 1, "Step")).collect();

    // API uses limit=20 as default page size.
    let page = EventQuery::new().limit(20).filter(&events);
    assert_eq!(page.len(), 20);

    let small_page = EventQuery::new().limit(5).filter(&events);
    assert_eq!(small_page.len(), 5);
}

// ---------------------------------------------------------------------------
// Contract: EventQuery pagination — sequence range for offset simulation
// ---------------------------------------------------------------------------

#[test]
fn contract_query_sequence_range_enables_offset_pagination() {
    let events: Vec<Event> = (1..=10).map(|seq| make_event(seq, "Feature", 1, "Step")).collect();

    // Simulate page 1 (sequences 1-5) and page 2 (sequences 6-10).
    let page1 = EventQuery::new()
        .from_sequence(1)
        .to_sequence(5)
        .filter(&events);
    let page2 = EventQuery::new()
        .from_sequence(6)
        .to_sequence(10)
        .filter(&events);

    assert_eq!(page1.len(), 5);
    assert_eq!(page2.len(), 5);
    assert!(page1.iter().all(|e| e.sequence <= 5));
    assert!(page2.iter().all(|e| e.sequence >= 6));
}

// ---------------------------------------------------------------------------
// Contract: EventQuery returns events in sequence order (no reordering)
// ---------------------------------------------------------------------------

#[test]
fn contract_query_preserves_sequence_order() {
    // Build events in forward order — filter must preserve order.
    let events: Vec<Event> = (1..=5).map(|seq| make_event(seq, "Feature", 1, "Step")).collect();
    let result = EventQuery::new().entity_type("Feature").filter(&events);

    let sequences: Vec<i64> = result.iter().map(|e| e.sequence).collect();
    let mut sorted = sequences.clone();
    sorted.sort();
    assert_eq!(sequences, sorted, "events must be in sequence order");
}

// ---------------------------------------------------------------------------
// Contract: EventQuery by event_type filter
// ---------------------------------------------------------------------------

#[test]
fn contract_query_filter_by_event_type() {
    let events = vec![
        make_event(1, "Feature", 1, "Created"),
        make_event(2, "Feature", 1, "Transitioned"),
        make_event(3, "Feature", 1, "Validated"),
        make_event(4, "Feature", 1, "Transitioned"),
    ];

    let result = EventQuery::new()
        .event_type("Transitioned")
        .filter(&events);

    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|e| e.event_type == "Transitioned"));
}

// ---------------------------------------------------------------------------
// Contract: EventQuery with no results returns empty vec (not error)
// ---------------------------------------------------------------------------

#[test]
fn contract_query_no_match_returns_empty_not_error() {
    let events = vec![make_event(1, "Feature", 1, "Created")];
    let result = EventQuery::new().entity_type("NonExistent").filter(&events);
    assert!(result.is_empty());
}

// ---------------------------------------------------------------------------
// Contract: Event serialization to JSON for API response
// ---------------------------------------------------------------------------

#[test]
fn contract_event_serializes_to_api_json_shape() {
    let event = make_event(5, "Feature", 1, "Transitioned");
    let json = serde_json::to_value(&event).expect("serialize event");

    // API contract: these fields must be present in the JSON response.
    assert!(json["id"].is_number());
    assert!(json["entity_type"].is_string());
    assert!(json["entity_id"].is_number());
    assert!(json["event_type"].is_string());
    assert!(json["sequence"].is_number());
    assert!(json["actor"].is_string());
    assert!(json["timestamp"].is_string());
    assert!(json["payload"].is_object() || json["payload"].is_null());
}

// ---------------------------------------------------------------------------
// Contract: Event timestamp is RFC3339 serializable
// ---------------------------------------------------------------------------

#[test]
fn contract_event_timestamp_is_rfc3339() {
    let event = make_event(1, "Feature", 1, "Created");
    let ts_str = event.timestamp.to_rfc3339();
    let parsed = ts_str.parse::<chrono::DateTime<chrono::Utc>>();
    assert!(parsed.is_ok(), "event timestamp must be valid RFC3339");
}

// ---------------------------------------------------------------------------
// Contract: hash fields are 32-byte arrays (API serializes as hex)
// ---------------------------------------------------------------------------

#[test]
fn contract_event_hash_fields_are_32_bytes() {
    let event = make_event(1, "Feature", 1, "Created");
    assert_eq!(event.prev_hash.len(), 32);
    assert_eq!(event.hash.len(), 32);
}

// ---------------------------------------------------------------------------
// Contract: combined filter (entity_type + entity_id + event_type + limit)
// ---------------------------------------------------------------------------

#[test]
fn contract_combined_filter_and_limit() {
    let events: Vec<Event> = (1..=20)
        .flat_map(|seq| {
            vec![
                make_event(seq * 2 - 1, "Feature", 1, "Transitioned"),
                make_event(seq * 2, "Feature", 2, "Transitioned"),
            ]
        })
        .collect();

    let result = EventQuery::new()
        .entity_type("Feature")
        .entity_id(1)
        .event_type("Transitioned")
        .limit(5)
        .filter(&events);

    assert_eq!(result.len(), 5);
    assert!(result.iter().all(|e| e.entity_id == 1 && e.event_type == "Transitioned"));
}
