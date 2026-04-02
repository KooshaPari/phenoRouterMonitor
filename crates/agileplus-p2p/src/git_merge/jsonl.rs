use std::collections::HashSet;
use std::path::Path;

use agileplus_domain::domain::event::Event;
use tracing::{info, warn};

use super::parser::parse_conflict_blocks;
use super::types::MergeError;

/// Resolve a conflicted JSONL event file.
///
/// Parses all lines from both sides of every conflict block, deduplicates by
/// the event `hash` field, and re-writes the file sorted by sequence.
pub(crate) fn resolve_jsonl_conflict(path: &Path) -> Result<bool, MergeError> {
    let content = std::fs::read_to_string(path)?;

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
                        let hash_hex = encode_hash(event.hash);
                        if seen_hashes.insert(hash_hex) {
                            events.push(event);
                        }
                    }
                    Err(e) => {
                        warn!("Skipping unparsable event line in {}: {e}", path.display());
                    }
                }
            }
        }
    }

    events.sort_by_key(|e| e.sequence);

    use std::io::Write as _;
    let mut file = std::fs::File::create(path)?;
    for event in &events {
        let line = serde_json::to_string(event).map_err(|e| MergeError::Parse {
            file: path.display().to_string(),
            source: e,
        })?;
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

fn encode_hash(bytes: [u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
