//! Git merge conflict resolution for git-backed state sync.
//!
//! Handles three conflict scenarios that arise when `git merge` or `git pull`
//! produces conflict markers in the sync directory:
//!
//! - Event JSONL files: append-only — deduplicate by hash, keep all unique events.
//! - Snapshot JSON files: latest-wins — keep the snapshot with the higher event_sequence.
//! - sync_state.json: field-level merge — highest sequence per entity, union of mappings.
//!
//! Traceability: WP17 / T103

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::time::Instant;

use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::snapshot::Snapshot;
use agileplus_domain::domain::sync_mapping::SyncMapping;
use tracing::{debug, info, warn};

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum MergeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error in {file}: {source}")]
    Parse {
        file: String,
        source: serde_json::Error,
    },

    #[error("No conflict markers found in {0}")]
    NoConflictMarkers(String),

    #[error("Malformed conflict block in {0}")]
    MalformedConflict(String),
}

// ── Resolution report ─────────────────────────────────────────────────────────

/// Summary of the conflicts resolved in a single `resolve_git_conflicts` call.
#[derive(Debug, Default, Clone)]
pub struct ConflictResolution {
    /// Number of JSONL event files where duplicates were removed.
    pub jsonl_files_resolved: usize,
    /// Number of snapshot files where the latest-wins rule was applied.
    pub snapshot_files_resolved: usize,
    /// Whether `sync_state.json` was re-merged.
    pub sync_state_merged: bool,
    pub duration_ms: u64,
}

// ── Git conflict-marker parser ─────────────────────────────────────────────────

/// Represents one side of a git conflict block.
#[derive(Debug)]
struct ConflictBlock {
    ours: String,
    theirs: String,
}

/// Parse a file that may contain standard git conflict markers:
/// ```text
/// <<<<<<< HEAD
/// ... ours ...
/// =======
/// ... theirs ...
/// >>>>>>> branch
/// ```
/// Returns the list of conflict blocks found.  If none are found the file is
/// returned as-is in a synthetic block with `theirs` set to the same content.
fn parse_conflict_blocks(content: &str) -> Vec<ConflictBlock> {
    let mut blocks: Vec<ConflictBlock> = Vec::new();
    let mut ours_lines: Vec<&str> = Vec::new();
    let mut theirs_lines: Vec<&str> = Vec::new();
    let mut in_conflict = false;
    let mut in_theirs = false;
    let mut found_any = false;

    for line in content.lines() {
        if line.starts_with("<<<<<<<") {
            in_conflict = true;
            in_theirs = false;
            ours_lines.clear();
            theirs_lines.clear();
            found_any = true;
        } else if line.starts_with("=======") && in_conflict {
            in_theirs = true;
        } else if line.starts_with(">>>>>>>") && in_conflict {
            blocks.push(ConflictBlock {
                ours: ours_lines.join("\n"),
                theirs: theirs_lines.join("\n"),
            });
            in_conflict = false;
            in_theirs = false;
        } else if in_conflict {
            if in_theirs {
                theirs_lines.push(line);
            } else {
                ours_lines.push(line);
            }
        }
    }

    if !found_any {
        // No conflict markers — treat the whole file as a synthetic "ours only" block.
        blocks.push(ConflictBlock {
            ours: content.to_string(),
            theirs: content.to_string(),
        });
    }

    blocks
}

// ── JSONL resolution ──────────────────────────────────────────────────────────

/// Resolve a conflicted JSONL event file.
///
/// Parses all lines from both sides of every conflict block, deduplicates by
/// the event `hash` field, and re-writes the file sorted by sequence.
fn resolve_jsonl_conflict(path: &Path) -> Result<bool, MergeError> {
    let content = std::fs::read_to_string(path)?;

    // Fast path: no conflict markers.
    if !content.contains("<<<<<<<") {
        return Ok(false);
    }

    let blocks = parse_conflict_blocks(&content);
    let mut seen_hashes: HashSet<String> = HashSet::new();
    let mut events: Vec<Event> = Vec::new();

    for block in &blocks {
        for side in [&block.ours, &block.theirs] {
            for line in side.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                match serde_json::from_str::<Event>(line) {
                    Ok(event) => {
                        let hash_hex = hex::encode(event.hash);
                        if seen_hashes.insert(hash_hex) {
                            events.push(event);
                        }
                    }
                    Err(e) => {
                        warn!("Skipping unparseable event line in {}: {e}", path.display());
                    }
                }
            }
        }
    }

    // Sort by sequence for determinism.
    events.sort_by_key(|e| e.sequence);

    // Re-write resolved file.
    use std::io::Write as _;
    let mut file = std::fs::File::create(path)?;
    for event in &events {
        let line = serde_json::to_string(event)
            .map_err(|e| MergeError::Parse { file: path.display().to_string(), source: e })?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
    }

    info!(
        "Resolved JSONL conflict in {} — {} unique events",
        path.display(),
        events.len()
    );
    Ok(true)
}

// ── Snapshot resolution ───────────────────────────────────────────────────────

/// Resolve a conflicted snapshot JSON file.
///
/// Parses both sides and keeps the snapshot with the higher `event_sequence`.
fn resolve_snapshot_conflict(path: &Path) -> Result<bool, MergeError> {
    let content = std::fs::read_to_string(path)?;

    if !content.contains("<<<<<<<") {
        return Ok(false);
    }

    let blocks = parse_conflict_blocks(&content);
    let mut winner: Option<Snapshot> = None;

    for block in &blocks {
        for side in [&block.ours, &block.theirs] {
            let text = side.trim();
            if text.is_empty() {
                continue;
            }
            match serde_json::from_str::<Snapshot>(text) {
                Ok(snap) => {
                    let replace = match &winner {
                        None => true,
                        Some(w) => snap.event_sequence > w.event_sequence,
                    };
                    if replace {
                        winner = Some(snap);
                    }
                }
                Err(e) => {
                    warn!("Skipping unparseable snapshot side in {}: {e}", path.display());
                }
            }
        }
    }

    let resolved = match winner {
        Some(snap) => {
            let json = serde_json::to_string_pretty(&snap).map_err(|e| MergeError::Parse {
                file: path.display().to_string(),
                source: e,
            })?;
            std::fs::write(path, json.as_bytes())?;
            true
        }
        None => {
            warn!("No parseable snapshot sides found in {}", path.display());
            false
        }
    };

    if resolved {
        info!("Resolved snapshot conflict in {}", path.display());
    }
    Ok(resolved)
}

// ── sync_state.json resolution ────────────────────────────────────────────────

/// Merge two `sync_state.json` values.
///
/// Strategy:
/// - `sync_mappings`: union by `(entity_type, entity_id)`, keeping the entry
///   with the highest `conflict_count` (most battle-tested).
/// - `sync_vector.entries`: per-key maximum sequence.
fn merge_sync_state(
    ours: &serde_json::Value,
    theirs: &serde_json::Value,
) -> serde_json::Value {
    // Merge sync_mappings by (entity_type, entity_id), highest conflict_count wins.
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

    // Merge sync_vector.entries by taking per-key maximum.
    let merge_vectors =
        |a: &serde_json::Value, b: &serde_json::Value| -> serde_json::Value {
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

    let ours_mappings = ours
        .get("sync_mappings")
        .cloned()
        .unwrap_or_default();
    let theirs_mappings = theirs
        .get("sync_mappings")
        .cloned()
        .unwrap_or_default();
    let ours_vector = ours.get("sync_vector").cloned().unwrap_or_default();
    let theirs_vector = theirs.get("sync_vector").cloned().unwrap_or_default();

    serde_json::json!({
        "sync_mappings": merge_mappings(&ours_mappings, &theirs_mappings),
        "sync_vector": merge_vectors(&ours_vector, &theirs_vector),
    })
}

fn resolve_sync_state_conflict(path: &Path) -> Result<bool, MergeError> {
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

// ── Public entry point ────────────────────────────────────────────────────────

/// Scan `repo_dir` for git conflict markers and resolve them in place.
///
/// Walks the `.agileplus/sync/` sub-directory and applies the appropriate
/// strategy per file type.  After resolution each file is a valid, conflict-
/// free file that can be staged and committed.
pub fn resolve_git_conflicts(repo_dir: &Path) -> Result<ConflictResolution, MergeError> {
    let started = Instant::now();
    let mut resolution = ConflictResolution::default();

    let sync_dir = repo_dir.join(".agileplus").join("sync");

    if !sync_dir.exists() {
        debug!("No .agileplus/sync directory found — nothing to resolve");
        resolution.duration_ms = started.elapsed().as_millis() as u64;
        return Ok(resolution);
    }

    // ── Events (JSONL) ────────────────────────────────────────────────────────
    let events_dir = sync_dir.join("events");
    if events_dir.exists() {
        for entry in walkdir(&events_dir)? {
            if entry.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                if resolve_jsonl_conflict(&entry)? {
                    resolution.jsonl_files_resolved += 1;
                }
            }
        }
    }

    // ── Snapshots (JSON) ──────────────────────────────────────────────────────
    let snapshots_dir = sync_dir.join("snapshots");
    if snapshots_dir.exists() {
        for entry in walkdir(&snapshots_dir)? {
            if entry.extension().and_then(|e| e.to_str()) == Some("json") {
                if resolve_snapshot_conflict(&entry)? {
                    resolution.snapshot_files_resolved += 1;
                }
            }
        }
    }

    // ── sync_state.json ───────────────────────────────────────────────────────
    let sync_state_path = sync_dir.join("sync_state.json");
    if sync_state_path.exists() && resolve_sync_state_conflict(&sync_state_path)? {
        resolution.sync_state_merged = true;
    }

    resolution.duration_ms = started.elapsed().as_millis() as u64;
    Ok(resolution)
}

/// Simple recursive file walker (avoids pulling in the `walkdir` crate).
fn walkdir(dir: &Path) -> Result<Vec<std::path::PathBuf>, MergeError> {
    let mut result = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            result.extend(walkdir(&p)?);
        } else {
            result.push(p);
        }
    }
    Ok(result)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::event::Event;
    use agileplus_domain::domain::snapshot::Snapshot;
    use std::io::Write as _;

    fn make_event_line(seq: i64) -> String {
        let mut e = Event::new("Feature", 1, "created", serde_json::json!({}), "test");
        e.sequence = seq;
        // Give each event a distinct hash so the dedup logic works.
        e.hash[0] = seq as u8;
        serde_json::to_string(&e).unwrap()
    }

    fn conflict_block(ours: &str, theirs: &str) -> String {
        format!("<<<<<<< HEAD\n{}\n=======\n{}\n>>>>>>> branch\n", ours, theirs)
    }

    #[test]
    fn resolve_jsonl_deduplicates() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("events/Feature");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("1.jsonl");

        let ev1 = make_event_line(1);
        let ev2 = make_event_line(2);

        // Both sides contain ev1; only ours has ev2.
        let content = conflict_block(
            &format!("{}\n{}", ev1, ev2),
            &ev1,
        );
        std::fs::write(&path, content).unwrap();

        let changed = resolve_jsonl_conflict(&path).unwrap();
        assert!(changed);

        let result = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<_> = result.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 2, "should have 2 unique events");
    }

    #[test]
    fn resolve_snapshot_latest_wins() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("snapshots/Feature");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("1.json");

        let old_snap = Snapshot::new("Feature", 1, serde_json::json!({"v": 1}), 1);
        let new_snap = Snapshot::new("Feature", 1, serde_json::json!({"v": 5}), 5);

        let ours = serde_json::to_string_pretty(&old_snap).unwrap();
        let theirs = serde_json::to_string_pretty(&new_snap).unwrap();
        let content = conflict_block(&ours, &theirs);
        std::fs::write(&path, content).unwrap();

        let changed = resolve_snapshot_conflict(&path).unwrap();
        assert!(changed);

        let result_text = std::fs::read_to_string(&path).unwrap();
        let result: Snapshot = serde_json::from_str(&result_text).unwrap();
        assert_eq!(result.event_sequence, 5, "newer snapshot should win");
    }

    #[test]
    fn merge_sync_state_takes_max_per_entity() {
        let ours = serde_json::json!({
            "sync_mappings": [
                {"id": 1, "entity_type": "Feature", "entity_id": 1,
                 "plane_issue_id": "p1", "content_hash": "h", "last_synced_at": "2024-01-01T00:00:00Z",
                 "sync_direction": "bidirectional", "conflict_count": 0}
            ],
            "sync_vector": {
                "device_id": "d1",
                "entries": {"Feature/1": 5}
            }
        });

        let theirs = serde_json::json!({
            "sync_mappings": [
                {"id": 1, "entity_type": "Feature", "entity_id": 1,
                 "plane_issue_id": "p1", "content_hash": "h", "last_synced_at": "2024-01-01T00:00:00Z",
                 "sync_direction": "bidirectional", "conflict_count": 2}
            ],
            "sync_vector": {
                "device_id": "d1",
                "entries": {"Feature/1": 10, "Feature/2": 3}
            }
        });

        let merged = merge_sync_state(&ours, &theirs);

        // Mappings: conflict_count 2 should win
        let mappings: Vec<SyncMapping> =
            serde_json::from_value(merged["sync_mappings"].clone()).unwrap();
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].conflict_count, 2);

        // Vector: Feature/1 max(5,10)=10, Feature/2 max(0,3)=3
        let entries = &merged["sync_vector"]["entries"];
        assert_eq!(entries["Feature/1"].as_u64(), Some(10));
        assert_eq!(entries["Feature/2"].as_u64(), Some(3));
    }

    #[test]
    fn resolve_git_conflicts_no_sync_dir() {
        let tmp = tempfile::tempdir().unwrap();
        // No .agileplus/sync dir — should succeed with empty resolution.
        let r = resolve_git_conflicts(tmp.path()).unwrap();
        assert_eq!(r.jsonl_files_resolved, 0);
        assert_eq!(r.snapshot_files_resolved, 0);
        assert!(!r.sync_state_merged);
    }

    #[test]
    fn resolve_git_conflicts_end_to_end() {
        let tmp = tempfile::tempdir().unwrap();
        let sync_dir = tmp.path().join(".agileplus/sync");

        // Create conflicted JSONL
        let events_dir = sync_dir.join("events/Feature");
        std::fs::create_dir_all(&events_dir).unwrap();
        let ev1 = make_event_line(1);
        let ev2 = make_event_line(2);
        std::fs::write(
            events_dir.join("1.jsonl"),
            conflict_block(&format!("{}\n{}", ev1, ev2), &ev1),
        )
        .unwrap();

        // Create conflicted snapshot
        let snap_dir = sync_dir.join("snapshots/Feature");
        std::fs::create_dir_all(&snap_dir).unwrap();
        let s1 = Snapshot::new("Feature", 1, serde_json::json!({}), 1);
        let s5 = Snapshot::new("Feature", 1, serde_json::json!({}), 5);
        std::fs::write(
            snap_dir.join("1.json"),
            conflict_block(
                &serde_json::to_string_pretty(&s1).unwrap(),
                &serde_json::to_string_pretty(&s5).unwrap(),
            ),
        )
        .unwrap();

        // Create conflicted sync_state.json
        let ss1 = serde_json::json!({
            "sync_mappings": [],
            "sync_vector": {"device_id": "d1", "entries": {"Feature/1": 3}}
        });
        let ss2 = serde_json::json!({
            "sync_mappings": [],
            "sync_vector": {"device_id": "d1", "entries": {"Feature/1": 7}}
        });
        std::fs::write(
            sync_dir.join("sync_state.json"),
            conflict_block(
                &serde_json::to_string_pretty(&ss1).unwrap(),
                &serde_json::to_string_pretty(&ss2).unwrap(),
            ),
        )
        .unwrap();

        let r = resolve_git_conflicts(tmp.path()).unwrap();
        assert_eq!(r.jsonl_files_resolved, 1);
        assert_eq!(r.snapshot_files_resolved, 1);
        assert!(r.sync_state_merged);
    }
}
