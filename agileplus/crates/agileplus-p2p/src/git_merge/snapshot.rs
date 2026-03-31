use std::path::Path;

use agileplus_domain::domain::snapshot::Snapshot;
use tracing::{info, warn};

use super::parser::parse_conflict_blocks;
use super::types::MergeError;

/// Resolve a conflicted snapshot JSON file.
///
/// Parses both sides and keeps the snapshot with the higher `event_sequence`.
pub(crate) fn resolve_snapshot_conflict(path: &Path) -> Result<bool, MergeError> {
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
                    warn!("Skipping unparsable snapshot side in {}: {e}", path.display());
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
