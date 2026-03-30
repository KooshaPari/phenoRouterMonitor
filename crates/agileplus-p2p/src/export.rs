//! Git-backed state export — serialize SQLite state to deterministic files.
//!
//! Writes to `.agileplus/sync/` with the layout:
//!   events/{entity_type}/{id}.jsonl  — one JSON per line, ordered by sequence
//!   snapshots/{entity_type}/{id}.json — latest snapshot, pretty-printed sorted keys
//!   sync_state.json                  — SyncMapping entries and device sync vectors
//!   device.json                      — local DeviceNode info
//!
//! All JSON uses sorted keys, 2-space indent, UTF-8.
//!
//! Traceability: WP17 / T101

use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::Path;
use std::time::Instant;

use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::snapshot::Snapshot;
use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_events::store::EventStore;
use agileplus_events::snapshot::SnapshotStore;
use serde_json::Value;
use tracing::debug;

use crate::device::{DeviceNode, DeviceStore};
use crate::error::ConnectionError;

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Event store error: {0}")]
    EventStore(String),

    #[error("Snapshot store error: {0}")]
    SnapshotStore(String),

    #[error("Device store error: {0}")]
    DeviceStore(#[from] ConnectionError),

    #[error("Sync store error: {0}")]
    SyncStore(String),
}

// ── Stats ─────────────────────────────────────────────────────────────────────

/// Statistics returned after a successful export.
#[derive(Debug, Default, Clone)]
pub struct ExportStats {
    pub events_exported: usize,
    pub snapshots_exported: usize,
    pub sync_mappings_exported: usize,
    pub duration_ms: u64,
}

// ── Entity registry helper ────────────────────────────────────────────────────

/// Describes a single (entity_type, entity_id) pair to export.
#[derive(Debug, Clone)]
pub struct EntityRef {
    pub entity_type: String,
    pub entity_id: i64,
}

// ── Sorted-key serialization helper ──────────────────────────────────────────

/// Recursively convert a `serde_json::Value` so that all Object variants use
/// `BTreeMap` (which serializes with sorted keys).
fn to_sorted(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> =
                map.into_iter().map(|(k, v)| (k, to_sorted(v))).collect();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(to_sorted).collect()),
        other => other,
    }
}

fn to_sorted_pretty(v: Value) -> Result<String, serde_json::Error> {
    let sorted = to_sorted(v);
    serde_json::to_string_pretty(&sorted)
}

fn to_sorted_line(v: Value) -> Result<String, serde_json::Error> {
    let sorted = to_sorted(v);
    serde_json::to_string(&sorted)
}

// ── Core export function ──────────────────────────────────────────────────────

/// Export all state to `output_dir` in a deterministic, git-friendly format.
///
/// Parameters:
/// - `entities` — the set of (entity_type, entity_id) pairs to export.  In a
///   production system this would be retrieved from a catalog; here callers
///   supply it to keep the function generic over `EventStore` implementations.
/// - `sync_mappings` — pre-fetched list of `SyncMapping` rows.
/// - `sync_vector_json` — the current device sync vector serialized to JSON.
pub async fn export_state<ES, SS>(
    event_store: &ES,
    snapshot_store: &SS,
    device_store: &dyn DeviceStore,
    sync_mappings: &[SyncMapping],
    sync_vector_json: Value,
    entities: &[EntityRef],
    output_dir: &Path,
) -> Result<ExportStats, ExportError>
where
    ES: EventStore,
    SS: SnapshotStore,
{
    let started = Instant::now();
    let mut stats = ExportStats::default();

    // ── 1. Device info ────────────────────────────────────────────────────────
    let device_path = output_dir.join("device.json");
    if let Some(parent) = device_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let device: Option<DeviceNode> = device_store.get_device()?;
    let device_json = serde_json::to_value(&device)?;
    std::fs::write(&device_path, to_sorted_pretty(device_json)?.as_bytes())?;
    debug!("Wrote device.json");

    // ── 2. Events + snapshots ─────────────────────────────────────────────────
    for entity in entities {
        // Events
        let events: Vec<Event> = event_store
            .get_events(&entity.entity_type, entity.entity_id)
            .await
            .map_err(|e| ExportError::EventStore(e.to_string()))?;

        if !events.is_empty() {
            let events_dir = output_dir
                .join("events")
                .join(&entity.entity_type);
            std::fs::create_dir_all(&events_dir)?;
            let file_path = events_dir.join(format!("{}.jsonl", entity.entity_id));
            let mut file = std::fs::File::create(&file_path)?;

            for event in &events {
                let line = to_sorted_line(serde_json::to_value(event)?)?;
                file.write_all(line.as_bytes())?;
                file.write_all(b"\n")?;
            }
            stats.events_exported += events.len();
            debug!(
                "Wrote {} events for {}/{}",
                events.len(),
                entity.entity_type,
                entity.entity_id
            );
        }

        // Snapshot
        let snapshot: Option<Snapshot> = snapshot_store
            .load(&entity.entity_type, entity.entity_id)
            .await
            .map_err(|e| ExportError::SnapshotStore(e.to_string()))?;

        if let Some(snap) = snapshot {
            let snap_dir = output_dir
                .join("snapshots")
                .join(&entity.entity_type);
            std::fs::create_dir_all(&snap_dir)?;
            let file_path = snap_dir.join(format!("{}.json", entity.entity_id));
            let snap_json = serde_json::to_value(&snap)?;
            std::fs::write(&file_path, to_sorted_pretty(snap_json)?.as_bytes())?;
            stats.snapshots_exported += 1;
            debug!(
                "Wrote snapshot for {}/{}",
                entity.entity_type, entity.entity_id
            );
        }
    }

    // ── 3. sync_state.json ────────────────────────────────────────────────────
    let sync_state = serde_json::json!({
        "sync_mappings": sync_mappings,
        "sync_vector": sync_vector_json,
    });
    let sync_state_path = output_dir.join("sync_state.json");
    std::fs::write(
        &sync_state_path,
        to_sorted_pretty(sync_state)?.as_bytes(),
    )?;
    stats.sync_mappings_exported = sync_mappings.len();
    debug!("Wrote sync_state.json with {} mappings", sync_mappings.len());

    stats.duration_ms = started.elapsed().as_millis() as u64;
    Ok(stats)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use agileplus_domain::domain::event::Event;
    use agileplus_domain::domain::snapshot::Snapshot;
    use agileplus_domain::domain::sync_mapping::SyncMapping;
    use agileplus_events::snapshot::SnapshotError;
    use agileplus_events::store::EventError;
    use async_trait::async_trait;
    use chrono::Utc;

    use crate::device::InMemoryDeviceStore;

    // ── Minimal in-memory EventStore ──────────────────────────────────────────

    #[derive(Default)]
    struct MemEventStore {
        events: Mutex<Vec<Event>>,
    }

    #[async_trait]
    impl EventStore for MemEventStore {
        async fn append(&self, event: &Event) -> Result<i64, EventError> {
            let mut g = self.events.lock().unwrap();
            g.push(event.clone());
            Ok(event.sequence)
        }

        async fn get_events(
            &self,
            entity_type: &str,
            entity_id: i64,
        ) -> Result<Vec<Event>, EventError> {
            let g = self.events.lock().unwrap();
            Ok(g.iter()
                .filter(|e| e.entity_type == entity_type && e.entity_id == entity_id)
                .cloned()
                .collect())
        }

        async fn get_events_since(
            &self,
            entity_type: &str,
            entity_id: i64,
            sequence: i64,
        ) -> Result<Vec<Event>, EventError> {
            let g = self.events.lock().unwrap();
            Ok(g.iter()
                .filter(|e| {
                    e.entity_type == entity_type
                        && e.entity_id == entity_id
                        && e.sequence > sequence
                })
                .cloned()
                .collect())
        }

        async fn get_events_by_range(
            &self,
            entity_type: &str,
            entity_id: i64,
            from: chrono::DateTime<Utc>,
            to: chrono::DateTime<Utc>,
        ) -> Result<Vec<Event>, EventError> {
            let g = self.events.lock().unwrap();
            Ok(g.iter()
                .filter(|e| {
                    e.entity_type == entity_type
                        && e.entity_id == entity_id
                        && e.timestamp >= from
                        && e.timestamp <= to
                })
                .cloned()
                .collect())
        }

        async fn get_latest_sequence(
            &self,
            entity_type: &str,
            entity_id: i64,
        ) -> Result<i64, EventError> {
            let g = self.events.lock().unwrap();
            Ok(g.iter()
                .filter(|e| e.entity_type == entity_type && e.entity_id == entity_id)
                .map(|e| e.sequence)
                .max()
                .unwrap_or(0))
        }
    }

    // ── Minimal in-memory SnapshotStore ───────────────────────────────────────

    #[derive(Default)]
    struct MemSnapshotStore {
        snapshots: Mutex<Vec<Snapshot>>,
    }

    #[async_trait]
    impl SnapshotStore for MemSnapshotStore {
        async fn save(&self, snapshot: &Snapshot) -> Result<(), SnapshotError> {
            self.snapshots.lock().unwrap().push(snapshot.clone());
            Ok(())
        }

        async fn load(
            &self,
            entity_type: &str,
            entity_id: i64,
        ) -> Result<Option<Snapshot>, SnapshotError> {
            let g = self.snapshots.lock().unwrap();
            Ok(g.iter()
                .filter(|s| s.entity_type == entity_type && s.entity_id == entity_id)
                .max_by_key(|s| s.event_sequence)
                .cloned())
        }

        async fn delete_before(
            &self,
            _entity_type: &str,
            _entity_id: i64,
            _sequence: i64,
        ) -> Result<(), SnapshotError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn export_creates_expected_files() {
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path();

        let es = MemEventStore::default();
        let ss = MemSnapshotStore::default();
        let ds = InMemoryDeviceStore::default();

        // Seed one event
        let mut ev = Event::new("Feature", 1, "created", serde_json::json!({"title": "T1"}), "test");
        ev.sequence = 1;
        es.append(&ev).await.unwrap();

        // Seed one snapshot
        let snap = Snapshot::new("Feature", 1, serde_json::json!({"title": "T1"}), 1);
        ss.save(&snap).await.unwrap();

        let mappings = vec![SyncMapping::new("Feature", 1, "plane-001", "hash-aaa")];
        let entities = vec![EntityRef { entity_type: "Feature".into(), entity_id: 1 }];

        let stats = export_state(
            &es,
            &ss,
            &ds,
            &mappings,
            serde_json::json!({}),
            &entities,
            out,
        )
        .await
        .unwrap();

        assert_eq!(stats.events_exported, 1);
        assert_eq!(stats.snapshots_exported, 1);
        assert_eq!(stats.sync_mappings_exported, 1);

        // Verify files exist
        assert!(out.join("device.json").exists());
        assert!(out.join("events/Feature/1.jsonl").exists());
        assert!(out.join("snapshots/Feature/1.json").exists());
        assert!(out.join("sync_state.json").exists());
    }

    #[test]
    fn to_sorted_sorts_object_keys() {
        let v = serde_json::json!({"z": 1, "a": 2, "m": 3});
        let s = to_sorted_pretty(v).unwrap();
        let pos_a = s.find('"').unwrap();
        let first_key = &s[pos_a + 1..pos_a + 2];
        assert_eq!(first_key, "a");
    }
}
