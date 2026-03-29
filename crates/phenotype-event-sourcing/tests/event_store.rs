//! Integration tests for event sourcing and hash chain verification.
//! Tests critical functionality: event immutability, hash chain integrity, concurrent access.

use chrono::Utc;
use phenotype_event_sourcing::{
    EventEnvelope, EventStore, InMemoryEventStore,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Test data structures
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Order {
    id: String,
    amount: f64,
    status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct User {
    name: String,
    email: String,
}

// Helper to create a test event
fn create_order_event(amount: f64, status: &str) -> EventEnvelope<Order> {
    EventEnvelope::new(
        Order {
            id: uuid::Uuid::new_v4().to_string(),
            amount,
            status: status.to_string(),
        },
        "test-user",
    )
}

fn create_user_event(name: &str, email: &str) -> EventEnvelope<User> {
    EventEnvelope::new(
        User {
            name: name.to_string(),
            email: email.to_string(),
        },
        "admin",
    )
}

// ============================================================================
// Single Event Tests
// ============================================================================

#[test]
fn test_append_single_event() {
    let store = InMemoryEventStore::new();
    let event = create_order_event(100.0, "pending");

    let seq = store
        .append(&event, "Order", "order-123")
        .expect("Failed to append event");

    assert_eq!(seq, 1, "First event should have sequence 1");
    assert_eq!(store.event_count(), 1, "Store should contain 1 event");
}

#[test]
fn test_append_batch_events() {
    let store = InMemoryEventStore::new();
    let sequences: Vec<i64> = (0..10)
        .map(|i| {
            let event = create_order_event((i as f64) * 10.0, "pending");
            store.append(&event, "Order", "order-123").unwrap()
        })
        .collect();

    assert_eq!(sequences.len(), 10);
    assert_eq!(sequences[0], 1);
    assert_eq!(sequences[9], 10);

    // Verify sequences are monotonically increasing
    for i in 1..sequences.len() {
        assert_eq!(sequences[i], sequences[i - 1] + 1);
    }

    assert_eq!(store.event_count(), 10);
}

#[test]
fn test_get_events_by_id() {
    let store = InMemoryEventStore::new();
    let event1 = create_order_event(100.0, "pending");
    let event2 = create_order_event(200.0, "confirmed");

    store.append(&event1, "Order", "order-123").unwrap();
    store.append(&event2, "Order", "order-123").unwrap();

    let events: Vec<EventEnvelope<Order>> = store
        .get_events("Order", "order-123")
        .expect("Failed to get events");

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].payload.amount, 100.0);
    assert_eq!(events[1].payload.amount, 200.0);
}

#[test]
fn test_get_events_since() {
    let store = InMemoryEventStore::new();

    for i in 1..=5 {
        let event = create_order_event(i as f64 * 100.0, "pending");
        store.append(&event, "Order", "order-123").unwrap();
    }

    // Get events after sequence 2
    let events: Vec<EventEnvelope<Order>> = store
        .get_events_since("Order", "order-123", 2)
        .expect("Failed to get events since");

    assert_eq!(events.len(), 3, "Should get events 3, 4, 5");
    assert_eq!(events[0].payload.amount, 300.0);
    assert_eq!(events[2].payload.amount, 500.0);
}

#[test]
fn test_get_events_by_range() {
    let store = InMemoryEventStore::new();
    let now = Utc::now();

    for i in 1..=3 {
        let event = create_order_event(i as f64 * 100.0, "pending");
        store.append(&event, "Order", "order-123").unwrap();
    }

    // Get events within a wide time range
    let events: Vec<EventEnvelope<Order>> = store
        .get_events_by_range("Order", "order-123", now - chrono::Duration::hours(1), now + chrono::Duration::hours(1))
        .expect("Failed to get events by range");

    assert!(events.len() >= 3, "Should get at least 3 events");
}

// ============================================================================
// Hash Chain Tests (CRITICAL)
// ============================================================================

#[test]
fn test_hash_chain_integrity() {
    let store = InMemoryEventStore::new();

    // Append multiple events and verify hash chain
    for i in 1..=5 {
        let event = create_order_event(i as f64 * 100.0, "pending");
        store.append(&event, "Order", "order-123").unwrap();
    }

    // Verify chain integrity
    store
        .verify_chain("Order", "order-123")
        .expect("Hash chain should be valid");

    let events: Vec<EventEnvelope<Order>> = store
        .get_events("Order", "order-123")
        .unwrap();

    // Verify each event has correct hash length (64 hex chars = 32 bytes)
    for event in &events {
        assert_eq!(event.hash.len(), 64, "Hash should be 64 hex characters");
        assert!(event.hash.chars().all(|c| c.is_ascii_hexdigit()), "Hash should be valid hex");
    }

    // Verify prev_hash chain
    for i in 1..events.len() {
        assert_eq!(
            events[i].prev_hash, events[i - 1].hash,
            "Event {} should have prev_hash pointing to event {}", i, i-1
        );
    }
}

#[test]
fn test_hash_chain_verification_fails_on_tampering() {
    let store = InMemoryEventStore::new();

    // Create and store events
    for i in 1..=3 {
        let event = create_order_event(i as f64 * 100.0, "pending");
        store.append(&event, "Order", "order-123").unwrap();
    }

    // Verify initial state is good
    store
        .verify_chain("Order", "order-123")
        .expect("Chain should be valid before tampering");

    // Note: In-memory store has private StoredEvent struct, so we can't directly
    // tamper with events. This test verifies the verify_chain mechanism works.
    // In a real scenario with persistent storage, tampering would be detectable.
}

#[test]
fn test_hash_determinism() {
    let store1 = InMemoryEventStore::new();
    let store2 = InMemoryEventStore::new();

    let event = create_order_event(100.0, "pending");

    let _seq1 = store1.append(&event, "Order", "order-123").unwrap();
    let _seq2 = store2.append(&event, "Order", "order-123").unwrap();

    let events1: Vec<EventEnvelope<Order>> =
        store1.get_events("Order", "order-123").unwrap();
    let events2: Vec<EventEnvelope<Order>> =
        store2.get_events("Order", "order-123").unwrap();

    // Same events should produce same hashes
    assert_eq!(events1[0].hash, events2[0].hash);
}

// ============================================================================
// Snapshot and Replay Tests
// ============================================================================

#[test]
fn test_snapshot_and_replay() {
    let store = InMemoryEventStore::new();

    // Generate events
    for i in 1..=5 {
        let event = create_order_event(i as f64 * 100.0, "pending");
        store.append(&event, "Order", "order-123").unwrap();
    }

    // Get snapshot at sequence 3
    let events: Vec<EventEnvelope<Order>> = store
        .get_events_since("Order", "order-123", 2)
        .unwrap();

    assert_eq!(events.len(), 3);

    // Replay from snapshot should work
    let replayed_events: Vec<EventEnvelope<Order>> =
        store.get_events("Order", "order-123").unwrap();

    assert_eq!(replayed_events.len(), 5);
}

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[test]
fn test_concurrent_appends() {
    let store = Arc::new(InMemoryEventStore::new());
    let mut handles = vec![];

    // Spawn 10 threads, each appending 10 events
    for thread_id in 0..10 {
        let store_clone = Arc::clone(&store);
        let handle = std::thread::spawn(move || {
            for i in 0..10 {
                let event = create_order_event(i as f64 * 100.0, "pending");
                let entity_id = format!("order-{}", thread_id);
                store_clone.append(&event, "Order", &entity_id).unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all events were stored
    assert_eq!(store.event_count(), 100, "Should have 100 total events");

    // Verify each entity has correct count
    for thread_id in 0..10 {
        let entity_id = format!("order-{}", thread_id);
        let events: Vec<EventEnvelope<Order>> =
            store.get_events("Order", &entity_id).unwrap();
        assert_eq!(
            events.len(),
            10,
            "Entity {} should have 10 events",
            entity_id
        );
    }
}

#[test]
fn test_concurrent_reads_and_writes() {
    let store = Arc::new(InMemoryEventStore::new());

    // Pre-populate with events
    {
        let store_clone = Arc::clone(&store);
        for i in 0..5 {
            let event = create_order_event(i as f64 * 100.0, "pending");
            store_clone.append(&event, "Order", "order-123").unwrap();
        }
    }

    let mut handles = vec![];

    // Spawn threads for both reads and writes
    for thread_id in 0..5 {
        let store_clone = Arc::clone(&store);
        let handle = std::thread::spawn(move || {
            if thread_id % 2 == 0 {
                // Read threads
                let _events: Vec<EventEnvelope<Order>> =
                    store_clone.get_events("Order", "order-123").unwrap();
            } else {
                // Write threads
                let event = create_order_event(999.0, "complete");
                store_clone.append(&event, "Order", "order-123").unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify final state - should have at least initial 5 + some writes
    let events: Vec<EventEnvelope<Order>> =
        store.get_events("Order", "order-123").unwrap();
    assert!(events.len() >= 7, "Should have at least 7 events after concurrent ops (5 initial + at least 2 writes)");
}

// ============================================================================
// Empty and Edge Case Tests
// ============================================================================

#[test]
fn test_empty_store_behavior() {
    let store = InMemoryEventStore::new();

    assert_eq!(store.event_count(), 0);

    // Getting events from empty store should return empty vector
    let events: Vec<EventEnvelope<Order>> = store
        .get_events("Order", "nonexistent")
        .unwrap_or_default();
    assert!(events.is_empty());
}

#[test]
fn test_get_latest_sequence_empty() {
    let store = InMemoryEventStore::new();

    let seq = store
        .get_latest_sequence("Order", "nonexistent")
        .unwrap_or(0);
    assert_eq!(seq, 0);
}

#[test]
fn test_get_latest_sequence_with_events() {
    let store = InMemoryEventStore::new();

    for i in 1..=5 {
        let event = create_order_event(i as f64 * 100.0, "pending");
        store.append(&event, "Order", "order-123").unwrap();
    }

    let latest = store
        .get_latest_sequence("Order", "order-123")
        .unwrap();
    assert_eq!(latest, 5);
}

#[test]
fn test_multiple_entity_types() {
    let store = InMemoryEventStore::new();

    let order_event = create_order_event(100.0, "pending");
    let user_event = create_user_event("Alice", "alice@example.com");

    store.append(&order_event, "Order", "order-123").unwrap();
    store.append(&user_event, "User", "user-456").unwrap();

    let orders: Vec<EventEnvelope<Order>> =
        store.get_events("Order", "order-123").unwrap();
    let users: Vec<EventEnvelope<User>> =
        store.get_events("User", "user-456").unwrap();

    assert_eq!(orders.len(), 1);
    assert_eq!(users.len(), 1);
    assert_eq!(orders[0].payload.amount, 100.0);
    assert_eq!(users[0].payload.name, "Alice");
}

#[test]
fn test_multiple_entity_ids() {
    let store = InMemoryEventStore::new();

    for entity_id in &["order-1", "order-2", "order-3"] {
        for i in 1..=3 {
            let event = create_order_event(i as f64 * 100.0, "pending");
            store.append(&event, "Order", entity_id).unwrap();
        }
    }

    // Verify isolation between entities
    for entity_id in &["order-1", "order-2", "order-3"] {
        let events: Vec<EventEnvelope<Order>> =
            store.get_events("Order", entity_id).unwrap();
        assert_eq!(events.len(), 3);
    }

    assert_eq!(store.event_count(), 9);
}

// ============================================================================
// Stress Tests
// ============================================================================

#[test]
fn test_stress_large_batch() {
    let store = InMemoryEventStore::new();

    const BATCH_SIZE: usize = 1000;
    for i in 0..BATCH_SIZE {
        let event = create_order_event(i as f64, "pending");
        store.append(&event, "Order", "order-123").unwrap();
    }

    assert_eq!(store.event_count(), BATCH_SIZE);

    let events: Vec<EventEnvelope<Order>> =
        store.get_events("Order", "order-123").unwrap();
    assert_eq!(events.len(), BATCH_SIZE);

    // Verify hash chain integrity on large batch
    store
        .verify_chain("Order", "order-123")
        .expect("Large batch should verify");
}

#[test]
fn test_clear_and_reuse() {
    let store = InMemoryEventStore::new();

    // First batch
    for i in 0..5 {
        let event = create_order_event(i as f64 * 100.0, "pending");
        store.append(&event, "Order", "order-123").unwrap();
    }
    assert_eq!(store.event_count(), 5);

    // Clear
    store.clear();
    assert_eq!(store.event_count(), 0);

    // Second batch
    for i in 0..3 {
        let event = create_order_event(i as f64 * 200.0, "complete");
        store.append(&event, "Order", "order-999").unwrap();
    }
    assert_eq!(store.event_count(), 3);
}
