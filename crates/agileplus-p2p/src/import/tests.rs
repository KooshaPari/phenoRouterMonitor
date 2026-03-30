use std::sync::Mutex;

use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::snapshot::Snapshot;
use agileplus_events::snapshot::SnapshotError;
use agileplus_events::store::EventError;
use async_trait::async_trait;
use chrono::Utc;

use super::*;

#[derive(Default)]
struct MemEventStore {
    events: Mutex<Vec<Event>>,
}

#[async_trait]
impl EventStore for MemEventStore {
    async fn append(&self, event: &Event) -> Result<i64, EventError> {
        self.events.lock().unwrap().push(event.clone());
        Ok(event.sequence)
    }

    async fn get_events(&self, entity_type: &str, entity_id: i64) -> Result<Vec<Event>, EventError> {
        let events = self.events.lock().unwrap();
        Ok(events
            .iter()
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
        let events = self.events.lock().unwrap();
        Ok(events
            .iter()
            .filter(|e| {
                e.entity_type == entity_type && e.entity_id == entity_id && e.sequence > sequence
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
        let events = self.events.lock().unwrap();
        Ok(events
            .iter()
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
        let events = self.events.lock().unwrap();
        Ok(events
            .iter()
            .filter(|e| e.entity_type == entity_type && e.entity_id == entity_id)
            .map(|e| e.sequence)
            .max()
            .unwrap_or(0))
    }
}

#[derive(Default)]
struct MemSnapshotStore {
    snapshots: Mutex<Vec<Snapshot>>,
}

#[async_trait]
impl SnapshotStore for MemSnapshotStore {
    async fn save(&self, snapshot: &Snapshot) -> Result<(), SnapshotError> {
        let mut snapshots = self.snapshots.lock().unwrap();
        snapshots.retain(|s| {
            !(s.entity_type == snapshot.entity_type && s.entity_id == snapshot.entity_id)
        });
        snapshots.push(snapshot.clone());
        Ok(())
    }

    async fn load(&self, entity_type: &str, entity_id: i64) -> Result<Option<Snapshot>, SnapshotError> {
        let snapshots = self.snapshots.lock().unwrap();
        Ok(snapshots
            .iter()
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

fn make_event(entity_type: &str, entity_id: i64, sequence: i64) -> Event {
    let mut event = Event::new(entity_type, entity_id, "created", serde_json::json!({}), "test");
    event.sequence = sequence;
    event
}

#[tokio::test]
async fn import_new_events() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let events_dir = dir.join("events/Feature");
    std::fs::create_dir_all(&events_dir).unwrap();
    let event = make_event("Feature", 1, 1);
    let line = serde_json::to_string(&event).unwrap();
    std::fs::write(events_dir.join("1.jsonl"), format!("{line}\n")).unwrap();

    let event_store = MemEventStore::default();
    let snapshot_store = MemSnapshotStore::default();
    let stats = import_state(dir, &event_store, &snapshot_store).await.unwrap();
    assert_eq!(stats.events_imported, 1);
}

#[tokio::test]
async fn import_skips_duplicate_events() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let event = make_event("Feature", 1, 1);
    let event_store = MemEventStore::default();
    event_store.append(&event).await.unwrap();
    let snapshot_store = MemSnapshotStore::default();

    let events_dir = dir.join("events/Feature");
    std::fs::create_dir_all(&events_dir).unwrap();
    let line = serde_json::to_string(&event).unwrap();
    std::fs::write(events_dir.join("1.jsonl"), format!("{line}\n")).unwrap();

    let stats = import_state(dir, &event_store, &snapshot_store).await.unwrap();
    assert_eq!(stats.events_imported, 0);
}

#[tokio::test]
async fn import_updates_snapshot_latest_wins() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let event_store = MemEventStore::default();
    let snapshot_store = MemSnapshotStore::default();

    let old_snapshot = Snapshot::new("Feature", 1, serde_json::json!({"v": 1}), 1);
    snapshot_store.save(&old_snapshot).await.unwrap();

    let snapshots_dir = dir.join("snapshots/Feature");
    std::fs::create_dir_all(&snapshots_dir).unwrap();
    let new_snapshot = Snapshot::new("Feature", 1, serde_json::json!({"v": 2}), 5);
    let json = serde_json::to_string_pretty(&new_snapshot).unwrap();
    std::fs::write(snapshots_dir.join("1.json"), json).unwrap();

    let stats = import_state(dir, &event_store, &snapshot_store).await.unwrap();
    assert_eq!(stats.snapshots_updated, 1);

    let loaded = snapshot_store.load("Feature", 1).await.unwrap().unwrap();
    assert_eq!(loaded.event_sequence, 5);
}

#[tokio::test]
async fn import_does_not_downgrade_snapshot() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let event_store = MemEventStore::default();
    let snapshot_store = MemSnapshotStore::default();

    let new_snapshot = Snapshot::new("Feature", 1, serde_json::json!({"v": 10}), 10);
    snapshot_store.save(&new_snapshot).await.unwrap();

    let snapshots_dir = dir.join("snapshots/Feature");
    std::fs::create_dir_all(&snapshots_dir).unwrap();
    let old_snapshot = Snapshot::new("Feature", 1, serde_json::json!({"v": 1}), 1);
    let json = serde_json::to_string_pretty(&old_snapshot).unwrap();
    std::fs::write(snapshots_dir.join("1.json"), json).unwrap();

    let stats = import_state(dir, &event_store, &snapshot_store).await.unwrap();
    assert_eq!(stats.snapshots_updated, 0);
}

#[tokio::test]
async fn import_sync_mappings_counted() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    let mappings = vec![
        agileplus_domain::domain::sync_mapping::SyncMapping::new("Feature", 1, "p1", "h1"),
        agileplus_domain::domain::sync_mapping::SyncMapping::new("Feature", 2, "p2", "h2"),
    ];
    let sync_state = serde_json::json!({ "sync_mappings": mappings, "sync_vector": {} });
    std::fs::write(
        dir.join("sync_state.json"),
        serde_json::to_string_pretty(&sync_state).unwrap(),
    )
    .unwrap();

    let event_store = MemEventStore::default();
    let snapshot_store = MemSnapshotStore::default();
    let stats = import_state(dir, &event_store, &snapshot_store).await.unwrap();
    assert_eq!(stats.sync_mappings_merged, 2);
}
