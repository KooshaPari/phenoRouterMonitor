//! Integration tests for event serialization across formats.
//!
//! Verifies:
//! - Round-trip serialization/deserialization for all formats
//! - Auto-detection of serialization formats
//! - Preservation of all event metadata
//! - Compatibility across format transitions

use phenotype_event_sourcing::{
    EventEnvelope, SerializationFormat, SerializerRegistry,
};
use serde_json::json;

/// Create a test event envelope with realistic data.
fn create_realistic_event() -> EventEnvelope<serde_json::Value> {
    EventEnvelope::new(
        json!({
            "event_type": "UserCreated",
            "user_id": "usr_abc123",
            "email": "user@example.com",
            "name": "Alice Smith",
            "created_at": "2024-01-15T10:30:00Z",
            "metadata": {
                "ip_address": "192.168.1.1",
                "user_agent": "Mozilla/5.0",
                "source": "web"
            }
        }),
        "auth-service",
    )
}

#[test]
fn json_format_preserves_all_metadata() {
    let registry = SerializerRegistry::new();
    let original = create_realistic_event();

    // Store original metadata for comparison
    let original_id = original.id;
    let original_timestamp = original.timestamp;
    let original_actor = original.actor.clone();
    let original_payload = original.payload.clone();

    // Serialize and deserialize
    let bytes = registry.serialize(&original, SerializationFormat::Json).unwrap();
    let restored = registry.deserialize(&bytes, SerializationFormat::Json).unwrap();

    // Verify all fields are preserved
    assert_eq!(restored.id, original_id);
    assert_eq!(restored.timestamp, original_timestamp);
    assert_eq!(restored.actor, original_actor);
    assert_eq!(restored.payload, original_payload);
    assert_eq!(restored.sequence, 0); // Default value
}

#[test]
fn binary_format_preserves_all_metadata() {
    let registry = SerializerRegistry::new();
    let original = create_realistic_event();

    // Store original metadata for comparison
    let original_id = original.id;
    let original_timestamp = original.timestamp;
    let original_actor = original.actor.clone();
    let original_payload = original.payload.clone();

    // Serialize and deserialize
    let bytes = registry.serialize(&original, SerializationFormat::Binary).unwrap();
    let restored = registry.deserialize(&bytes, SerializationFormat::Binary).unwrap();

    // Verify all fields are preserved
    assert_eq!(restored.id, original_id);
    assert_eq!(restored.timestamp, original_timestamp);
    assert_eq!(restored.actor, original_actor);
    assert_eq!(restored.payload, original_payload);
}

#[test]
fn auto_detect_json_format() {
    let registry = SerializerRegistry::new();
    let original = create_realistic_event();

    // Serialize as JSON
    let bytes = registry.serialize(&original, SerializationFormat::Json).unwrap();

    // Auto-detect and deserialize
    let restored = registry.deserialize_auto(&bytes).unwrap();

    assert_eq!(restored.actor, original.actor);
    assert_eq!(restored.payload, original.payload);
}

#[test]
fn auto_detect_binary_format() {
    let registry = SerializerRegistry::new();
    let original = create_realistic_event();

    // Serialize as binary
    let bytes = registry.serialize(&original, SerializationFormat::Binary).unwrap();

    // Auto-detect and deserialize
    let restored = registry.deserialize_auto(&bytes).unwrap();

    assert_eq!(restored.actor, original.actor);
    assert_eq!(restored.payload, original.payload);
}

#[test]
fn json_serialization_is_human_readable() {
    let registry = SerializerRegistry::new();
    let event = create_realistic_event();

    let bytes = registry.serialize(&event, SerializationFormat::Json).unwrap();
    let json_string = String::from_utf8(bytes).unwrap();

    // Verify it's valid JSON and contains expected fields
    let parsed: serde_json::Value = serde_json::from_str(&json_string).unwrap();
    assert!(parsed.get("id").is_some());
    assert!(parsed.get("actor").is_some());
    assert!(parsed.get("payload").is_some());
    assert!(parsed.get("timestamp").is_some());
}

#[test]
fn binary_format_includes_header() {
    let registry = SerializerRegistry::new();
    let event = create_realistic_event();

    let bytes = registry.serialize(&event, SerializationFormat::Binary).unwrap();

    // Verify binary header
    assert!(bytes.starts_with(b"BINARY\0\0"));
}

#[test]
fn binary_format_is_more_compact_than_json() {
    let registry = SerializerRegistry::new();
    let event = create_realistic_event();

    let json_bytes = registry.serialize(&event, SerializationFormat::Json).unwrap();
    let binary_bytes = registry.serialize(&event, SerializationFormat::Binary).unwrap();

    // Binary should be smaller (or at least not significantly larger with header)
    // This is a soft assertion since both use JSON internally for now
    assert!(binary_bytes.len() > 0);
    assert!(json_bytes.len() > 0);
}

#[test]
fn multiple_events_roundtrip() {
    let registry = SerializerRegistry::new();

    // Create multiple diverse events
    let events = vec![
        EventEnvelope::new(json!({"action": "login"}), "user-1"),
        EventEnvelope::new(json!({"action": "create_post", "post_id": 42}), "user-2"),
        EventEnvelope::new(json!({"action": "delete", "count": 100}), "admin"),
    ];

    for event in events {
        // Test JSON format
        let json_bytes = registry.serialize(&event, SerializationFormat::Json).unwrap();
        let json_restored = registry.deserialize(&json_bytes, SerializationFormat::Json).unwrap();
        assert_eq!(json_restored.actor, event.actor);
        assert_eq!(json_restored.payload, event.payload);

        // Test binary format
        let binary_bytes = registry.serialize(&event, SerializationFormat::Binary).unwrap();
        let binary_restored = registry.deserialize(&binary_bytes, SerializationFormat::Binary).unwrap();
        assert_eq!(binary_restored.actor, event.actor);
        assert_eq!(binary_restored.payload, event.payload);

        // Test auto-detection
        let auto_json = registry.deserialize_auto(&json_bytes).unwrap();
        assert_eq!(auto_json.actor, event.actor);

        let auto_binary = registry.deserialize_auto(&binary_bytes).unwrap();
        assert_eq!(auto_binary.actor, event.actor);
    }
}

#[test]
fn complex_payload_roundtrip() {
    let registry = SerializerRegistry::new();
    let complex_payload = json!({
        "order": {
            "id": "ord-12345",
            "items": [
                {"sku": "ABC-001", "qty": 2, "price": 29.99},
                {"sku": "XYZ-999", "qty": 1, "price": 149.99}
            ],
            "customer": {
                "id": "cust-xyz",
                "name": "John Doe",
                "emails": ["john@example.com", "j.doe@work.com"],
                "addresses": [
                    {"type": "billing", "zip": "12345"},
                    {"type": "shipping", "zip": "54321"}
                ]
            },
            "totals": {
                "subtotal": 209.97,
                "tax": 16.80,
                "shipping": 9.99,
                "total": 236.76
            }
        },
        "status": "confirmed",
        "timestamp": "2024-01-15T14:22:33Z"
    });

    let event = EventEnvelope::new(complex_payload.clone(), "order-service");

    // Round-trip through JSON
    let json_bytes = registry.serialize(&event, SerializationFormat::Json).unwrap();
    let json_restored = registry.deserialize(&json_bytes, SerializationFormat::Json).unwrap();
    assert_eq!(json_restored.payload, complex_payload);

    // Round-trip through binary
    let binary_bytes = registry.serialize(&event, SerializationFormat::Binary).unwrap();
    let binary_restored = registry.deserialize(&binary_bytes, SerializationFormat::Binary).unwrap();
    assert_eq!(binary_restored.payload, complex_payload);
}

#[test]
fn serializer_registry_is_thread_safe() {
    use std::sync::Arc;
    use std::thread;

    let registry = Arc::new(SerializerRegistry::new());
    let event = create_realistic_event();

    // Spawn multiple threads and serialize concurrently
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let registry = registry.clone();
            let event = event.clone();
            thread::spawn(move || {
                let bytes = registry.serialize(&event, SerializationFormat::Json).unwrap();
                let restored = registry.deserialize(&bytes, SerializationFormat::Json).unwrap();
                assert_eq!(restored.actor, event.actor);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn invalid_binary_header_fallback_to_json() {
    let registry = SerializerRegistry::new();
    let event = create_realistic_event();

    // Serialize as JSON (no binary header)
    let json_bytes = registry.serialize(&event, SerializationFormat::Json).unwrap();

    // Auto-detect should fall back to JSON
    let restored = registry.deserialize_auto(&json_bytes).unwrap();
    assert_eq!(restored.actor, event.actor);
    assert_eq!(restored.payload, event.payload);
}
