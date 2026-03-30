use std::time::Instant;

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

pub(crate) fn finish_resolution(
    started: Instant,
    mut resolution: ConflictResolution,
) -> ConflictResolution {
    resolution.duration_ms = started.elapsed().as_millis() as u64;
    resolution
}
