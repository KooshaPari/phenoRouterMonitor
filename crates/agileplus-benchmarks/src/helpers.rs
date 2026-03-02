//! Shared test-data helpers used by all benchmark files.
//!
//! All helpers are sync (return values directly) so they can be called
//! from both Criterion bench closures and regular `#[test]` functions.

use agileplus_domain::domain::{
    event::Event,
    feature::Feature,
    state_machine::FeatureState,
};
use agileplus_sqlite::SqliteStorageAdapter;
use chrono::Utc;

// ---------------------------------------------------------------------------
// SQLite adapter helpers
// ---------------------------------------------------------------------------

/// Open an in-memory SQLite adapter with WAL mode and all migrations applied.
pub fn make_in_memory_adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().expect("in-memory adapter init failed")
}

// ---------------------------------------------------------------------------
// Event helpers
// ---------------------------------------------------------------------------

/// Build a domain `Event` for the given entity / sequence without a real hash.
pub fn make_event(entity_id: i64, sequence: i64) -> Event {
    Event {
        id: 0,
        entity_type: "Feature".to_string(),
        entity_id,
        event_type: "StateTransitioned".to_string(),
        payload: serde_json::json!({"sequence": sequence, "state": "Specified"}),
        actor: "bench-agent".to_string(),
        timestamp: Utc::now(),
        prev_hash: [0u8; 32],
        hash: [0u8; 32],
        sequence,
    }
}

/// Build N events for entity_id=1, sequences 1..=count.
pub fn make_events(count: i64) -> Vec<Event> {
    (1..=count).map(|seq| make_event(1, seq)).collect()
}

/// Build N events spread across `entity_count` entities.
///
/// Entity IDs run 1..=entity_count, sequences restart per entity.
pub fn make_events_multi_entity(count: i64, entity_count: i64) -> Vec<Event> {
    (0..count)
        .map(|i| {
            let entity_id = (i % entity_count) + 1;
            let sequence = (i / entity_count) + 1;
            make_event(entity_id, sequence)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Feature helpers
// ---------------------------------------------------------------------------

/// Build a minimal `Feature` with the given numeric ID embedded in slug.
pub fn make_feature(id: i64) -> Feature {
    let mut f = Feature::new(
        &format!("feature-{id}"),
        &format!("Feature {id}"),
        [0u8; 32],
        None,
    );
    f.id = id;
    f
}

/// Build N features with IDs 1..=n.
pub fn make_features(n: i64) -> Vec<Feature> {
    (1..=n).map(make_feature).collect()
}

// ---------------------------------------------------------------------------
// Aggregate helper for replay benchmarks
// ---------------------------------------------------------------------------

/// A minimal aggregate that simply counts the events applied to it.
#[derive(Default)]
pub struct CountingAggregate {
    pub version: i64,
    pub events_applied: u64,
    pub last_state: String,
}

#[async_trait::async_trait]
impl agileplus_events::Aggregate for CountingAggregate {
    async fn apply(
        &mut self,
        event: &Event,
    ) -> Result<(), agileplus_events::ReplayError> {
        self.events_applied += 1;
        self.last_state = event
            .payload
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        self.version = event.sequence;
        Ok(())
    }

    fn version(&self) -> i64 {
        self.version
    }

    fn set_version(&mut self, v: i64) {
        self.version = v;
    }
}

// ---------------------------------------------------------------------------
// Snapshot helper
// ---------------------------------------------------------------------------

use agileplus_domain::domain::snapshot::Snapshot;

/// Build a `Snapshot` representing the aggregate state at `seq`.
pub fn make_snapshot(entity_id: i64, seq: i64) -> Snapshot {
    Snapshot {
        id: 0,
        entity_type: "Feature".to_string(),
        entity_id,
        event_sequence: seq,
        state: serde_json::json!({"events_applied": seq, "last_state": "Specified"}),
        created_at: Utc::now(),
    }
}

// ---------------------------------------------------------------------------
// Sync / vector helpers
// ---------------------------------------------------------------------------

/// A lightweight representation of a feature sync payload.
#[derive(Debug, Clone)]
pub struct SyncPayload {
    pub id: i64,
    pub slug: String,
    pub state: FeatureState,
    pub description: String,
}

impl SyncPayload {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            slug: format!("feature-{id}"),
            state: FeatureState::Specified,
            description: "x".repeat(256), // ~256-byte description
        }
    }
}

/// Build N sync payloads.
pub fn make_sync_payloads(n: i64) -> Vec<SyncPayload> {
    (1..=n).map(SyncPayload::new).collect()
}

/// Simulate serialising and deserialising a sync payload (the core hot-path
/// of push/pull without requiring a live Plane.so connection).
pub fn simulate_sync_roundtrip(payload: &SyncPayload) -> SyncPayload {
    let json = serde_json::json!({
        "id":          payload.id,
        "slug":        payload.slug,
        "state":       format!("{:?}", payload.state),
        "description": payload.description,
    });
    let s: serde_json::Value = serde_json::from_str(&json.to_string()).unwrap();
    SyncPayload {
        id:          s["id"].as_i64().unwrap(),
        slug:        s["slug"].as_str().unwrap().to_string(),
        state:       FeatureState::Specified,
        description: s["description"].as_str().unwrap().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_event_valid() {
        let e = make_event(1, 5);
        assert_eq!(e.entity_id, 1);
        assert_eq!(e.sequence, 5);
        assert_eq!(e.entity_type, "Feature");
    }

    #[test]
    fn make_events_count() {
        let events = make_events(100);
        assert_eq!(events.len(), 100);
        assert_eq!(events.last().unwrap().sequence, 100);
    }

    #[test]
    fn make_events_multi_entity_count() {
        let events = make_events_multi_entity(100, 10);
        assert_eq!(events.len(), 100);
    }

    #[test]
    fn make_feature_valid() {
        let f = make_feature(42);
        assert_eq!(f.id, 42);
        assert_eq!(f.slug, "feature-42");
    }

    #[test]
    fn make_sync_payloads_count() {
        let payloads = make_sync_payloads(50);
        assert_eq!(payloads.len(), 50);
    }

    #[test]
    fn simulate_roundtrip_preserves_fields() {
        let p = SyncPayload::new(7);
        let out = simulate_sync_roundtrip(&p);
        assert_eq!(out.id, 7);
        assert_eq!(out.slug, "feature-7");
    }

    #[test]
    fn make_in_memory_adapter_ok() {
        let _ = make_in_memory_adapter();
    }

    #[test]
    fn make_snapshot_valid() {
        let snap = make_snapshot(1, 900);
        assert_eq!(snap.event_sequence, 900);
        assert_eq!(snap.entity_id, 1);
    }
}
