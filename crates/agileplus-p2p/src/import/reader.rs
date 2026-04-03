use std::path::Path;

use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::snapshot::Snapshot;
use agileplus_domain::domain::sync_mapping::SyncMapping;

use super::ImportError;

/// Read all `.jsonl` files recursively under `events_dir` and parse them into
/// `Event` values. Returns a flat Vec sorted by stream order.
pub(super) fn read_events_from_dir(events_dir: &Path) -> Result<Vec<Event>, ImportError> {
    let mut events: Vec<Event> = Vec::new();

    if !events_dir.exists() {
        return Ok(events);
    }

    for entity_type_entry in std::fs::read_dir(events_dir)? {
        let entity_type_entry = entity_type_entry?;
        let entity_type_path = entity_type_entry.path();
        if !entity_type_path.is_dir() {
            continue;
        }

        for file_entry in std::fs::read_dir(&entity_type_path)? {
            let file_entry = file_entry?;
            let file_path = file_entry.path();
            if file_path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            let contents = std::fs::read_to_string(&file_path)?;
            for (line_no, line) in contents.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let event: Event =
                    serde_json::from_str(line).map_err(|source| ImportError::Deserialization {
                        file: format!("{}:{}", file_path.display(), line_no + 1),
                        source,
                    })?;
                events.push(event);
            }
        }
    }

    events.sort_by(|a, b| {
        a.entity_type
            .cmp(&b.entity_type)
            .then(a.entity_id.cmp(&b.entity_id))
            .then(a.sequence.cmp(&b.sequence))
    });

    Ok(events)
}

/// Read all `.json` snapshot files under `snapshots_dir`.
pub(super) fn read_snapshots_from_dir(snapshots_dir: &Path) -> Result<Vec<Snapshot>, ImportError> {
    let mut snapshots: Vec<Snapshot> = Vec::new();

    if !snapshots_dir.exists() {
        return Ok(snapshots);
    }

    for entity_type_entry in std::fs::read_dir(snapshots_dir)? {
        let entity_type_entry = entity_type_entry?;
        let entity_type_path = entity_type_entry.path();
        if !entity_type_path.is_dir() {
            continue;
        }

        for file_entry in std::fs::read_dir(&entity_type_path)? {
            let file_entry = file_entry?;
            let file_path = file_entry.path();
            if file_path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let contents = std::fs::read_to_string(&file_path)?;
            let snapshot: Snapshot =
                serde_json::from_str(&contents).map_err(|source| ImportError::Deserialization {
                    file: file_path.display().to_string(),
                    source,
                })?;
            snapshots.push(snapshot);
        }
    }

    Ok(snapshots)
}

/// Read sync mappings from `sync_state.json` if present.
pub(super) fn read_sync_mappings(sync_state_path: &Path) -> Result<Vec<SyncMapping>, ImportError> {
    if !sync_state_path.exists() {
        return Ok(Vec::new());
    }

    let contents = std::fs::read_to_string(sync_state_path)?;
    let value: serde_json::Value =
        serde_json::from_str(&contents).map_err(|source| ImportError::Deserialization {
            file: sync_state_path.display().to_string(),
            source,
        })?;

    let mappings_value = value
        .get("sync_mappings")
        .cloned()
        .unwrap_or(serde_json::Value::Array(Vec::new()));

    let mappings: Vec<SyncMapping> =
        serde_json::from_value(mappings_value).map_err(|source| ImportError::Deserialization {
            file: sync_state_path.display().to_string(),
            source,
        })?;

    Ok(mappings)
}
