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

mod jsonl;
mod parser;
mod resolver;
mod snapshot;
mod sync_state;
mod types;

#[cfg(test)]
mod tests;

pub use types::{ConflictResolution, MergeError};

use std::path::Path;

/// Scan `repo_dir` for git conflict markers and resolve them in place.
///
/// Walks the `.agileplus/sync/` sub-directory and applies the appropriate
/// strategy per file type. After resolution each file is a valid, conflict-free
/// file that can be staged and committed.
pub fn resolve_git_conflicts(repo_dir: &Path) -> Result<ConflictResolution, MergeError> {
    resolver::resolve_git_conflicts(repo_dir)
}
