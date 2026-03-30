//! Hash computation for events.

use sha2::{Digest, Sha256};

pub fn compute_event_hash<T: serde::Serialize>(event: &T) -> String {
    let mut hasher = Sha256::new();
    hasher.update(serde_json::to_string(event).unwrap_or_default().as_bytes());
    hex::encode(hasher.finalize())
}

pub fn verify_event_hash<T: serde::Serialize>(event: &T, expected: &str) -> bool {
    compute_event_hash(event) == expected
}
