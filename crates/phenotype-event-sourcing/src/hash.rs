//! Hash utilities for event sourcing — SHA-256 hash chain.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::HashError;

/// Compute SHA-256 hash for an event envelope.
pub fn compute_hash(
    id: &Uuid,
    timestamp: DateTime<Utc>,
    entity_type: &str,
    payload: &serde_json::Value,
    actor: &str,
    prev_hash: &str,
) -> crate::error::Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(id.to_string().as_bytes());
    hasher.update(timestamp.to_rfc3339().as_bytes());
    hasher.update(entity_type.as_bytes());
    hasher.update(payload.to_string().as_bytes());
    hasher.update(actor.as_bytes());
    hasher.update(prev_hash.as_bytes());
    Ok(hex::encode(hasher.finalize()))
}

/// Public alias used by lib.rs re-export.
pub fn compute_event_hash(
    id: &Uuid,
    timestamp: DateTime<Utc>,
    entity_type: &str,
    payload: &serde_json::Value,
    actor: &str,
    prev_hash: &str,
) -> crate::error::Result<String> {
    compute_hash(id, timestamp, entity_type, payload, actor, prev_hash)
}

/// Verify an event hash matches expected value.
pub fn verify_event_hash(hash: &str, expected: &str) -> crate::error::Result<bool> {
    Ok(hash == expected)
}

/// Verify a chain of (hash, prev_hash) pairs is consistent.
pub fn verify_chain(chain: &[(String, String)]) -> crate::error::Result<()> {
    for (i, window) in chain.windows(2).enumerate() {
        let (ref current_hash, _) = window[0];
        let (_, ref next_prev) = window[1];
        if current_hash != next_prev {
            return Err(HashError::ChainBroken {
                sequence: (i + 1) as i64,
            }
            .into());
        }
    }
    Ok(())
}
