use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use agileplus_domain::domain::sync_mapping::SyncMapping;
use tracing::info;

use super::parser::parse_conflict_blocks;
use super::types::MergeError;

/// Merge two `sync_state.json` values.
///
/// Strategy:
/// - `sync_mappings`: union by `(entity_type, entity_id)`, keeping the entry
///   with the highest `conflict_count` (most battle-tested).
/// - `sync_vector.entries`: per-key maximum sequence.
pub(crate) fn merge_sync_state(
    ours: &serde_json::Value,
    theirs: &serde_json::Value,
) -> serde_json::Value {
    let merge_mappings = |a: &serde_json::Value, b: &serde_json::Value| -> serde_json::Value {
        let a_vec: Vec<SyncMapping> = serde_json::from_value(a.clone()).unwrap_or_default();
        let b_vec: Vec<SyncMapping> = serde_json::from_value(b.clone()).unwrap_or_default();

        let mut map: BTreeMap<(String, i64), SyncMapping> = BTreeMap::new();
        for m in a_vec.into_iter().chain(b_vec) {
            let key = (m.entity_type.clone(), m.entity_id);
            let replace = match map.get(&key) {
                None => true,
                Some(existing) => m.conflict_count > existing.conflict_count,
            };
            if replace {
                map.insert(key, m);
            }
        }

        serde_json::to_value(map.into_values().collect::<Vec<_>>())
            .unwrap_or(serde_json::Value::Array(vec![]))
    };

    let merge_vectors = |a: &serde_json::Value, b: &serde_json::Value| -> serde_json::Value {
        let a_entries: HashMap<String, u64> =
            serde_json::from_value(a.get("entries").cloned().unwrap_or_default())
                .unwrap_or_default();
        let b_entries: HashMap<String, u64> =
            serde_json::from_value(b.get("entries").cloned().unwrap_or_default())
                .unwrap_or_default();

        let mut merged: BTreeMap<String, u64> = BTreeMap::new();
        for (k, v) in a_entries.into_iter().chain(b_entries) {
            let entry = merged.entry(k).or_insert(0);
            if v > *entry {
                *entry = v;
            }
        }

        let device_id = a
            .get("device_id")
            .or_else(|| b.get("device_id"))
            .cloned()
            .unwrap_or_default();

        serde_json::json!({
            "device_id": device_id,
            "entries": merged,
        })
    };

    let ours_mappings = ours.get("sync_mappings").cloned().unwrap_or_default();
    let theirs_mappings = theirs.get("sync_mappings").cloned().unwrap_or_default();
    let ours_vector = ours.get("sync_vector").cloned().unwrap_or_default();
    let theirs_vector = theirs.get("sync_vector").cloned().unwrap_or_default();

    serde_json::json!({
        "sync_mappings": merge_mappings(&ours_mappings, &theirs_mappings),
        "sync_vector": merge_vectors(&ours_vector, &theirs_vector),
    })
}

/// Resolve a conflicted `sync_state.json` file.
pub(crate) fn resolve_sync_state_conflict(path: &Path) -> Result<bool, MergeError> {
    let content = std::fs::read_to_string(path)?;

    if !content.contains("<<<<<<<") {
        return Ok(false);
    }

    let blocks = parse_conflict_blocks(&content);
    let mut merged: Option<serde_json::Value> = None;

    for block in &blocks {
        let parse_side = |text: &str| -> Option<serde_json::Value> {
            let t = text.trim();
            if t.is_empty() {
                None
            } else {
                serde_json::from_str(t).ok()
            }
        };

        match (parse_side(&block.ours), parse_side(&block.theirs)) {
            (Some(ours), Some(theirs)) => {
                let partial = merge_sync_state(&ours, &theirs);
                merged = Some(match merged {
                    None => partial,
                    Some(prev) => merge_sync_state(&prev, &partial),
                });
            }
            (Some(v), None) | (None, Some(v)) => {
                merged = Some(match merged {
                    None => v.clone(),
                    Some(prev) => merge_sync_state(&prev, &v),
                });
            }
            (None, None) => {}
        }
    }

    let result = match merged {
        Some(v) => v,
        None => {
            return Err(MergeError::MalformedConflict(path.display().to_string()));
        }
    };

    let json = serde_json::to_string_pretty(&result).map_err(|e| MergeError::Parse {
        file: path.display().to_string(),
        source: e,
    })?;
    std::fs::write(path, json.as_bytes())?;

    info!("Resolved sync_state.json conflict at {}", path.display());
    Ok(true)
}
