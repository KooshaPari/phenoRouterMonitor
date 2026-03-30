use std::io::Write as _;
use std::path::Path;
use std::time::Instant;

use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::snapshot::Snapshot;
use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_events::snapshot::SnapshotStore;
use agileplus_events::store::EventStore;
use serde_json::Value;
use tracing::debug;

use crate::device::{DeviceNode, DeviceStore};

use super::errors::ExportError;
use super::serialization::{to_sorted_line, to_sorted_pretty};
use super::types::{EntityRef, ExportStats};

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

    write_device(device_store, output_dir)?;

    for entity in entities {
        stats.events_exported += export_events(event_store, entity, output_dir).await?;
        stats.snapshots_exported += export_snapshot(snapshot_store, entity, output_dir).await?;
    }

    write_sync_state(sync_mappings, sync_vector_json, output_dir)?;
    stats.sync_mappings_exported = sync_mappings.len();
    stats.duration_ms = started.elapsed().as_millis() as u64;
    Ok(stats)
}

fn write_device(device_store: &dyn DeviceStore, output_dir: &Path) -> Result<(), ExportError> {
    let device_path = output_dir.join("device.json");
    if let Some(parent) = device_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let device: Option<DeviceNode> = device_store.get_device()?;
    let device_json = serde_json::to_value(&device)?;
    std::fs::write(&device_path, to_sorted_pretty(device_json)?.as_bytes())?;
    debug!("Wrote device.json");
    Ok(())
}

async fn export_events<ES: EventStore>(
    event_store: &ES,
    entity: &EntityRef,
    output_dir: &Path,
) -> Result<usize, ExportError> {
    let events: Vec<Event> = event_store
        .get_events(&entity.entity_type, entity.entity_id)
        .await
        .map_err(|error| ExportError::EventStore(error.to_string()))?;

    if events.is_empty() {
        return Ok(0);
    }

    let events_dir = output_dir.join("events").join(&entity.entity_type);
    std::fs::create_dir_all(&events_dir)?;
    let file_path = events_dir.join(format!("{}.jsonl", entity.entity_id));
    let mut file = std::fs::File::create(&file_path)?;

    for event in &events {
        let line = to_sorted_line(serde_json::to_value(event)?)?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
    }

    debug!(
        "Wrote {} events for {}/{}",
        events.len(),
        entity.entity_type,
        entity.entity_id
    );
    Ok(events.len())
}

async fn export_snapshot<SS: SnapshotStore>(
    snapshot_store: &SS,
    entity: &EntityRef,
    output_dir: &Path,
) -> Result<usize, ExportError> {
    let snapshot: Option<Snapshot> = snapshot_store
        .load(&entity.entity_type, entity.entity_id)
        .await
        .map_err(|error| ExportError::SnapshotStore(error.to_string()))?;

    let Some(snapshot) = snapshot else {
        return Ok(0);
    };

    let snapshot_dir = output_dir.join("snapshots").join(&entity.entity_type);
    std::fs::create_dir_all(&snapshot_dir)?;
    let file_path = snapshot_dir.join(format!("{}.json", entity.entity_id));
    let snapshot_json = serde_json::to_value(&snapshot)?;
    std::fs::write(&file_path, to_sorted_pretty(snapshot_json)?.as_bytes())?;
    debug!(
        "Wrote snapshot for {}/{}",
        entity.entity_type, entity.entity_id
    );
    Ok(1)
}

fn write_sync_state(
    sync_mappings: &[SyncMapping],
    sync_vector_json: Value,
    output_dir: &Path,
) -> Result<(), ExportError> {
    let sync_state = serde_json::json!({
        "sync_mappings": sync_mappings,
        "sync_vector": sync_vector_json,
    });
    let sync_state_path = output_dir.join("sync_state.json");
    std::fs::write(&sync_state_path, to_sorted_pretty(sync_state)?.as_bytes())?;
    debug!(
        "Wrote sync_state.json with {} mappings",
        sync_mappings.len()
    );
    Ok(())
}
