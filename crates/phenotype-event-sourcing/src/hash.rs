//! Hash utilities for event sourcing.

use hex;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::HashError;

/// Compute the event hash for a single event envelope.
pub fn compute_event_hash(
    id: &Uuid,
    timestamp: chrono::DateTime<chrono::Utc>,
    entity_type: &str,
    payload: &serde_json::Value,
    actor: &str,
    prev_hash: &str,
) -> std::result::Result<String, HashError> {
    let payload_json =
        serde_json::to_string(payload).map_err(|e| HashError::InvalidPayload(e.to_string()))?;

    let mut hasher = Sha256::new();
    hasher.update(id.as_bytes());
    hasher.update(timestamp.to_rfc3339());
    hasher.update(entity_type.as_bytes());
    hasher.update(payload_json.as_bytes());
    hasher.update(actor.as_bytes());
    hasher.update(prev_hash.as_bytes());

    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

/// Verify that a chain of event hashes is internally consistent.
///
/// Each stored tuple is (hash, prev_hash). The chain is valid when each `prev_hash`
/// equals the previous event's `hash`.
pub fn verify_event_hash(chain: &[(String, String)]) -> std::result::Result<(), HashError> {
    if chain.is_empty() {
        return Ok(());
    }

    for (index, (hash, prev_hash)) in chain.iter().enumerate() {
        if hash.len() != 64 {
            return Err(HashError::InvalidHashLength(hash.len()));
        }

        if prev_hash.len() != 64 {
            return Err(HashError::InvalidHashLength(prev_hash.len()));
        }

        if index > 0 {
            let expected_prev_hash = &chain[index - 1].0;
            if prev_hash != expected_prev_hash {
                return Err(HashError::ChainBroken {
                    sequence: index as i64 + 1,
                });
            }
        }
    }

    Ok(())
}
