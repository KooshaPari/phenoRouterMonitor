//! Git-backed state import — read deterministic files back into the event store.
//!
//! Reads from the same layout written by `export.rs`:
//!   events/{entity_type}/{id}.jsonl  — JSONL, one event per line
//!   snapshots/{entity_type}/{id}.json — latest snapshot (pretty JSON)
//!   sync_state.json                  — SyncMapping entries and device sync vectors
//!
//! Skips duplicate events by hash comparison.  All events are collected before
//! any are applied (transaction-like semantics).
//!
//! Traceability: WP17 / T102

use std::path::Path;
use std::time::Instant;

use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::snapshot::Snapshot;
use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_events::snapshot::SnapshotStore;
use agileplus_events::store::EventStore;
use tracing::{debug, warn};

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Deserialization error in {file}: {source}")]
    Deserialization {
        file: String,
        source: serde_json::Error,
    },

    #[error("Event store error: {0}")]
    EventStore(String),

    #[error("Snapshot store error: {0}")]
    SnapshotStore(String),
}

// ── Stats ─────────────────────────────────────────────────────────────────────

/// Statistics returned after a successful import.
#[derive(Debug, Default, Clone)]
pub struct ImportStats {
    pub events_imported: usize,
    pub snapshots_updated: usize,
    pub sync_mappings_merged: usize,
    pub duration_ms: u64,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Read all `.jsonl` files recursively under `events_dir` and parse them into
/// `Event` values.  Returns them grouped as a flat Vec sorted by sequence.
fn read_events_from_dir(events_dir: &Path) -> Result<Vec<Event>, ImportError> {
    let mut events: Vec<Event> = Vec::new();

    if !events_dir.exists() {
        return Ok(events);
    }

    for entity_type_entry in std::fs::read_dir(events_dir)? {
        let entity_type_entry = entity_type_entry?;
        let et_path = entity_type_entry.path();
        if !et_path.is_dir() {
            continue;
        }

        for file_entry in std::fs::read_dir(&et_path)? {
            let file_entry = file_entry?;
            let fp = file_entry.path();
            if fp.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            let contents = std::fs::read_to_string(&fp)?;
            for (line_no, line) in contents.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let event: Event =
                    serde_json::from_str(line).map_err(|e| ImportError::Deserialization {
                        file: format!("{}:{}", fp.display(), line_no + 1),
                        source: e,
                    })?;
                events.push(event);
            }
        }
    }

    // Sort ascending by entity_type, entity_id, then sequence so that appending
    // respects the original stream order.
    events.sort_by(|a, b| {
        a.entity_type
            .cmp(&b.entity_type)
            .then(a.entity_id.cmp(&b.entity_id))
            .then(a.sequence.cmp(&b.sequence))
    });

    Ok(events)
}

/// Read all `.json` snapshot files under `snapshots_dir`.
fn read_snapshots_from_dir(snapshots_dir: &Path) -> Result<Vec<Snapshot>, ImportError> {
    let mut snapshots: Vec<Snapshot> = Vec::new();

    if !snapshots_dir.exists() {
        return Ok(snapshots);
    }

    for entity_type_entry in std::fs::read_dir(snapshots_dir)? {
        let entity_type_entry = entity_type_entry?;
        let et_path = entity_type_entry.path();
        if !et_path.is_dir() {
            continue;
        }

        for file_entry in std::fs::read_dir(&et_path)? {
            let file_entry = file_entry?;
            let fp = file_entry.path();
            if fp.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let contents = std::fs::read_to_string(&fp)?;
            let snap: Snapshot =
                serde_json::from_str(&contents).map_err(|e| ImportError::Deserialization {
                    file: fp.display().to_string(),
                    source: e,
                })?;
            snapshots.push(snap);
        }
    }

    Ok(snapshots)
}

/// Read sync mappings from `sync_state.json` if present.
fn read_sync_mappings(sync_state_path: &Path) -> Result<Vec<SyncMapping>, ImportError> {
    if !sync_state_path.exists() {
        return Ok(Vec::new());
    }

    let contents = std::fs::read_to_string(sync_state_path)?;
    let value: serde_json::Value =
        serde_json::from_str(&contents).map_err(|e| ImportError::Deserialization {
            file: sync_state_path.display().to_string(),
            source: e,
        })?;

    let mappings_value = value
        .get("sync_mappings")
        .cloned()
        .unwrap_or(serde_json::Value::Array(Vec::new()));

    let mappings: Vec<SyncMapping> = serde_json::from_value(mappings_value).map_err(|e| {
        ImportError::Deserialization {
            file: sync_state_path.display().to_string(),
            source: e,
        }
    })?;

    Ok(mappings)
}

// ── Core import function ──────────────────────────────────────────────────────

/// Import state from `input_dir` into the provided stores.
///
/// Events that already exist (matched by `hash` field) are silently skipped.
/// All events are collected before any writes occur (transaction-like).
pub async fn import_state<ES, SS>(
    input_dir: &Path,
    event_store: &ES,
    snapshot_store: &SS,
) -> Result<ImportStats, ImportError>
where
    ES: EventStore,
    SS: SnapshotStore,
{
    let started = Instant::now();
    let mut stats = ImportStats::default();

    // ── Collect phase ─────────────────────────────────────────────────────────
    let all_events = read_events_from_dir(&input_dir.join("events"))?;
    let all_snapshots = read_snapshots_from_dir(&input_dir.join("snapshots"))?;
    let all_mappings = read_sync_mappings(&input_dir.join("sync_state.json"))?;

    debug!(
        "Collected {} events, {} snapshots, {} mappings from {}",
        all_events.len(),
        all_snapshots.len(),
        all_mappings.len(),
        input_dir.display()
    );

    // ── Apply events (skip duplicates by hash) ────────────────────────────────
    for event in &all_events {
        // Check whether this sequence already exists for the entity stream.
        let latest_seq = event_store
            .get_latest_sequence(&event.entity_type, event.entity_id)
            .await
            .map_err(|e| ImportError::EventStore(e.to_string()))?;

        if event.sequence <= latest_seq {
            // Potentially a duplicate; verify by loading the exact event.
            let existing = event_store
                .get_events_since(
                    &event.entity_type,
                    event.entity_id,
                    event.sequence - 1,
                )
                .await
                .map_err(|e| ImportError::EventStore(e.to_string()))?;

            let already_present = existing.iter().any(|e| {
                e.sequence == event.sequence && e.hash == event.hash
            });

            if already_present {
                debug!(
                    "Skipping duplicate event {}/{} seq={}",
                    event.entity_type, event.entity_id, event.sequence
                );
                continue;
            }
        }

        event_store
            .append(event)
            .await
            .map_err(|e| ImportError::EventStore(e.to_string()))?;
        stats.events_imported += 1;
    }

    // ── Apply snapshots (latest-wins by event_sequence) ───────────────────────
    for snapshot in &all_snapshots {
        // Load existing snapshot for this entity to compare sequences.
        let existing = snapshot_store
            .load(&snapshot.entity_type, snapshot.entity_id)
            .await
            .map_err(|e| ImportError::SnapshotStore(e.to_string()))?;

        let should_update = match &existing {
            None => true,
            Some(ex) => snapshot.event_sequence > ex.event_sequence,
        };

        if should_update {
            snapshot_store
                .save(snapshot)
                .await
                .map_err(|e| ImportError::SnapshotStore(e.to_string()))?;
            stats.snapshots_updated += 1;
        } else {
            warn!(
                "Skipping older snapshot for {}/{} (imported seq={} <= existing seq={})",
                snapshot.entity_type,
                snapshot.entity_id,
                snapshot.event_sequence,
                existing.unwrap().event_sequence
            );
        }
    }

    stats.sync_mappings_merged = all_mappings.len();
    stats.duration_ms = started.elapsed().as_millis() as u64;
    Ok(stats)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Mutex;

    use agileplus_domain::domain::event::Event;
    use agileplus_domain::domain::snapshot::Snapshot;
    use agileplus_events::snapshot::SnapshotError;
    use agileplus_events::store::EventError;
    use async_trait::async_trait;
    use chrono::Utc;

    // ── Shared in-memory stores (mirrors export tests) ────────────────────────

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

    #[derive(Default)]
    struct MemSnapshotStore {
        snapshots: Mutex<Vec<Snapshot>>,
    }

    #[async_trait]
    impl SnapshotStore for MemSnapshotStore {
        async fn save(&self, snapshot: &Snapshot) -> Result<(), SnapshotError> {
            let mut g = self.snapshots.lock().unwrap();
            // Remove existing entry for same entity before inserting
            g.retain(|s| {
                !(s.entity_type == snapshot.entity_type && s.entity_id == snapshot.entity_id)
            });
            g.push(snapshot.clone());
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

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_event(entity_type: &str, entity_id: i64, sequence: i64) -> Event {
        let mut e = Event::new(entity_type, entity_id, "created", serde_json::json!({}), "test");
        e.sequence = sequence;
        e
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn import_new_events() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        // Write a JSONL file
        let events_dir = dir.join("events/Feature");
        std::fs::create_dir_all(&events_dir).unwrap();
        let ev = make_event("Feature", 1, 1);
        let line = serde_json::to_string(&ev).unwrap();
        std::fs::write(events_dir.join("1.jsonl"), format!("{}\n", line)).unwrap();

        let es = MemEventStore::default();
        let ss = MemSnapshotStore::default();
        let stats = import_state(dir, &es, &ss).await.unwrap();
        assert_eq!(stats.events_imported, 1);
    }

    #[tokio::test]
    async fn import_skips_duplicate_events() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        let ev = make_event("Feature", 1, 1);

        // Pre-populate the store with this event
        let es = MemEventStore::default();
        es.append(&ev).await.unwrap();
        let ss = MemSnapshotStore::default();

        // Write the same event to the import dir
        let events_dir = dir.join("events/Feature");
        std::fs::create_dir_all(&events_dir).unwrap();
        let line = serde_json::to_string(&ev).unwrap();
        std::fs::write(events_dir.join("1.jsonl"), format!("{}\n", line)).unwrap();

        let stats = import_state(dir, &es, &ss).await.unwrap();
        assert_eq!(stats.events_imported, 0, "duplicate should be skipped");
    }

    #[tokio::test]
    async fn import_updates_snapshot_latest_wins() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        let es = MemEventStore::default();
        let ss = MemSnapshotStore::default();

        // Seed an old snapshot
        let old_snap = Snapshot::new("Feature", 1, serde_json::json!({"v": 1}), 1);
        ss.save(&old_snap).await.unwrap();

        // Write a newer snapshot to import dir
        let snaps_dir = dir.join("snapshots/Feature");
        std::fs::create_dir_all(&snaps_dir).unwrap();
        let new_snap = Snapshot::new("Feature", 1, serde_json::json!({"v": 2}), 5);
        let json = serde_json::to_string_pretty(&new_snap).unwrap();
        std::fs::write(snaps_dir.join("1.json"), json).unwrap();

        let stats = import_state(dir, &es, &ss).await.unwrap();
        assert_eq!(stats.snapshots_updated, 1);

        let loaded = ss.load("Feature", 1).await.unwrap().unwrap();
        assert_eq!(loaded.event_sequence, 5);
    }

    #[tokio::test]
    async fn import_does_not_downgrade_snapshot() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();

        let es = MemEventStore::default();
        let ss = MemSnapshotStore::default();

        // Seed a newer snapshot
        let new_snap = Snapshot::new("Feature", 1, serde_json::json!({"v": 10}), 10);
        ss.save(&new_snap).await.unwrap();

        // Write an older snapshot to import dir
        let snaps_dir = dir.join("snapshots/Feature");
        std::fs::create_dir_all(&snaps_dir).unwrap();
        let old_snap = Snapshot::new("Feature", 1, serde_json::json!({"v": 1}), 1);
        let json = serde_json::to_string_pretty(&old_snap).unwrap();
        std::fs::write(snaps_dir.join("1.json"), json).unwrap();

        let stats = import_state(dir, &es, &ss).await.unwrap();
        assert_eq!(stats.snapshots_updated, 0, "older snapshot must not overwrite newer");
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

        let es = MemEventStore::default();
        let ss = MemSnapshotStore::default();
        let stats = import_state(dir, &es, &ss).await.unwrap();
        assert_eq!(stats.sync_mappings_merged, 2);
    }
}
