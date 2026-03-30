//! SHA-256 hash chain computation and verification.

use chrono::{DateTime, Utc};
use hex::FromHex;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::HashError;

/// Compute SHA-256 hash for an event.
///
/// Hash inputs (in order, length-prefixed where noted):
/// 1. UUID (16 bytes)
/// 2. timestamp (length-prefixed ISO 8601)
/// 3. event_type (length-prefixed UTF-8)
/// 4. payload (length-prefixed JSON)
/// 5. actor (length-prefixed UTF-8)
/// 6. prev_hash (64 hex chars = 32 bytes decoded)
pub fn compute_hash(
    id: &Uuid,
    timestamp: DateTime<Utc>,
    event_type: &str,
    payload: &Value,
    actor: &str,
    prev_hash: &str,
) -> Result<String, HashError> {
    let mut hasher = Sha256::new();

    hasher.update(id.as_bytes());

    let timestamp_str = timestamp.to_rfc3339();
    hasher.update((timestamp_str.len() as u32).to_be_bytes());
    hasher.update(timestamp_str.as_bytes());

    hasher.update((event_type.len() as u32).to_be_bytes());
    hasher.update(event_type.as_bytes());

    let payload_json =
        serde_json::to_string(payload).map_err(|_| HashError::InvalidHashLength(0))?;
    hasher.update((payload_json.len() as u32).to_be_bytes());
    hasher.update(payload_json.as_bytes());

    hasher.update((actor.len() as u32).to_be_bytes());
    hasher.update(actor.as_bytes());

    let prev_bytes = <Vec<u8>>::from_hex(prev_hash)
        .map_err(|_| HashError::InvalidHashLength(prev_hash.len()))?;
    if prev_bytes.len() != 32 {
        return Err(HashError::InvalidHashLength(prev_bytes.len()));
    }
    hasher.update(&prev_bytes);

    Ok(hex::encode(hasher.finalize()))
}

/// Verify `prev_hash` linkage (genesis is 64 ASCII `'0'` digits, matching the in-memory store).
pub fn verify_chain(events: &[(String, String)]) -> Result<(), HashError> {
    if events.is_empty() {
        return Ok(());
    }

    let zero_hash = "0".repeat(64);
    if events[0].1 != zero_hash {
        return Err(HashError::ChainBroken { sequence: 1 });
    }

    for (i, (_hash, prev_hash)) in events.iter().enumerate() {
        if i == 0 {
            continue;
        }
        let seq = (i + 1) as i64;
        if prev_hash != &events[i - 1].0 {
            return Err(HashError::ChainBroken { sequence: seq });
        }
    }

    Ok(())
}

/// Returns the first missing sequence number, or None if continuous.
pub fn detect_gaps(sequences: &[i64]) -> Option<i64> {
    if sequences.is_empty() {
        return None;
    }

    let mut sorted = sequences.to_vec();
    sorted.sort_unstable();

    for i in 1..sorted.len() {
        if sorted[i] != sorted[i - 1] + 1 {
            return Some(sorted[i - 1] + 1);
        }
    }

    None
}

/// SHA-256 over JSON serialization of `event` (convenience API).
pub fn compute_event_hash<T: Serialize>(event: &T) -> String {
    let mut hasher = Sha256::new();
    hasher.update(serde_json::to_string(event).unwrap_or_default().as_bytes());
    hex::encode(hasher.finalize())
}

pub fn verify_event_hash<T: Serialize>(event: &T, expected: &str) -> bool {
    compute_event_hash(event) == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_hash_deterministic() {
        let id = Uuid::nil();
        let ts = DateTime::parse_from_rfc3339("2026-03-02T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let payload = serde_json::json!({"n": "t"});
        let zero_hash = "0".repeat(64);

        let h1 = compute_hash(&id, ts, "created", &payload, "u1", &zero_hash).unwrap();
        let h2 = compute_hash(&id, ts, "created", &payload, "u1", &zero_hash).unwrap();

        assert_eq!(h1, h2);
        assert_ne!(h1, zero_hash);
    }

    #[test]
    fn verify_chain_two_events() {
        let zero_hash = "0".repeat(64);
        let h1 = "abc123".to_string();
        let h2 = "def456".to_string();

        verify_chain(&[(h1.clone(), zero_hash), (h2, h1)]).unwrap();
    }
}
