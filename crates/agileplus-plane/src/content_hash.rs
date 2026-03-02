//! T051: Content Hash Tracking — SHA-256 of (title + description + state + labels).
//!
//! Traceability: WP08-T051

use sha2::{Digest, Sha256};

/// Compute a SHA-256 content hash for an entity.
///
/// Input is a stable concatenation of title, description, state, and sorted labels.
/// The sort ensures label order doesn't cause spurious conflicts.
pub fn compute_content_hash(
    title: &str,
    description: &str,
    state: &str,
    labels: &[String],
) -> String {
    let mut sorted_labels = labels.to_vec();
    sorted_labels.sort_unstable();

    let mut hasher = Sha256::new();
    hasher.update(title.as_bytes());
    hasher.update(b"\x00");
    hasher.update(description.as_bytes());
    hasher.update(b"\x00");
    hasher.update(state.as_bytes());
    hasher.update(b"\x00");
    for label in &sorted_labels {
        hasher.update(label.as_bytes());
        hasher.update(b"\x00");
    }
    format!("{:x}", hasher.finalize())
}

/// Detect whether a content conflict exists.
///
/// A conflict is when:
/// - The remote hash (what Plane.so currently has) differs from our last known hash AND
/// - The local hash (what we want to write) also differs from the last known hash.
///
/// This means both sides have made independent edits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictStatus {
    /// No conflict; one or both sides are unchanged.
    Clean,
    /// Both sides differ from the baseline — a true conflict.
    Conflict,
}

pub fn detect_conflict(baseline_hash: &str, local_hash: &str, remote_hash: &str) -> ConflictStatus {
    let local_changed = local_hash != baseline_hash;
    let remote_changed = remote_hash != baseline_hash;
    if local_changed && remote_changed {
        ConflictStatus::Conflict
    } else {
        ConflictStatus::Clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_deterministic() {
        let h1 = compute_content_hash("Title", "Desc", "implementing", &["bug".into(), "feature".into()]);
        let h2 = compute_content_hash("Title", "Desc", "implementing", &["bug".into(), "feature".into()]);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_label_order_independent() {
        let h1 = compute_content_hash("T", "D", "s", &["a".into(), "b".into()]);
        let h2 = compute_content_hash("T", "D", "s", &["b".into(), "a".into()]);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_state_change_detected() {
        let h1 = compute_content_hash("T", "D", "created", &[]);
        let h2 = compute_content_hash("T", "D", "implementing", &[]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn no_conflict_when_only_one_side_changed() {
        let baseline = "base";
        let local = "new-local";
        let remote = "base"; // remote unchanged
        assert_eq!(detect_conflict(baseline, local, remote), ConflictStatus::Clean);
    }

    #[test]
    fn conflict_when_both_sides_changed() {
        let baseline = "base";
        let local = "new-local";
        let remote = "new-remote";
        assert_eq!(detect_conflict(baseline, local, remote), ConflictStatus::Conflict);
    }

    #[test]
    fn no_conflict_when_both_unchanged() {
        assert_eq!(detect_conflict("base", "base", "base"), ConflictStatus::Clean);
    }
}
