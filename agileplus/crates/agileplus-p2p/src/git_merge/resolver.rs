use std::path::Path;
use std::time::Instant;

use tracing::debug;

use super::jsonl::resolve_jsonl_conflict;
use super::snapshot::resolve_snapshot_conflict;
use super::sync_state::resolve_sync_state_conflict;
use super::types::{ConflictResolution, MergeError, finish_resolution};

pub(crate) fn resolve_git_conflicts(repo_dir: &Path) -> Result<ConflictResolution, MergeError> {
    let started = Instant::now();
    let mut resolution = ConflictResolution::default();

    let sync_dir = repo_dir.join(".agileplus").join("sync");

    if !sync_dir.exists() {
        debug!("No .agileplus/sync directory found — nothing to resolve");
        return Ok(finish_resolution(started, resolution));
    }

    let events_dir = sync_dir.join("events");
    if events_dir.exists() {
        for entry in walkdir(&events_dir)? {
            if entry.extension().and_then(|e| e.to_str()) == Some("jsonl")
                && resolve_jsonl_conflict(&entry)?
            {
                resolution.jsonl_files_resolved += 1;
            }
        }
    }

    let snapshots_dir = sync_dir.join("snapshots");
    if snapshots_dir.exists() {
        for entry in walkdir(&snapshots_dir)? {
            if entry.extension().and_then(|e| e.to_str()) == Some("json")
                && resolve_snapshot_conflict(&entry)?
            {
                resolution.snapshot_files_resolved += 1;
            }
        }
    }

    let sync_state_path = sync_dir.join("sync_state.json");
    if sync_state_path.exists() && resolve_sync_state_conflict(&sync_state_path)? {
        resolution.sync_state_merged = true;
    }

    Ok(finish_resolution(started, resolution))
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
