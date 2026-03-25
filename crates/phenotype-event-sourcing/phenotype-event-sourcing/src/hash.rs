//! SHA-256 hash chain computation and verification.

use chrono::{DateTime, Utc};
use hex::FromHex;
use sha2::{Digest, Sha256};

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
    id: &uuid::Uuid,
    timestamp: DateTime<Utc>,
    event_type: &str,
    payload: &serde_json::Value,
    actor: &str,
    prev_hash: &str,
) -> Result<String, HashError> {
    let mut hasher = Sha256::new();

    // UUID bytes (16 bytes)
    hasher.update(id.as_bytes());

    // Timestamp (ISO 8601 string)
    let timestamp_str = timestamp.to_rfc3339();
    hasher.update((timestamp_str.len() as u32).to_be_bytes());
    hasher.update(timestamp_str.as_bytes());

    // Event type
    hasher.update((event_type.len() as u32).to_be_bytes());
    hasher.update(event_type.as_bytes());

    // Payload (JSON)
    let payload_json = serde_json::to_string(payload)
        .map_err(|_| HashError::InvalidHashLength(0))?;
    hasher.update((payload_json.len() as u32).to_be_bytes());
    hasher.update(payload_json.as_bytes());

    // Actor
    hasher.update((actor.len() as u32).to_be_bytes());
    hasher.update(actor.as_bytes());

    // Previous hash (decode from hex)
    let prev_bytes = <Vec<u8>>::from_hex(prev_hash)
        .map_err(|_| HashError::InvalidHashLength(prev_hash.len()))?;
    if prev_bytes.len() != 32 {
        return Err(HashError::InvalidHashLength(prev_bytes.len()));
    }
    hasher.update(&prev_bytes);

    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// Verify the integrity of an event chain.
///
/// Ensures each event's hash is correctly computed and chains to its predecessor.
pub fn verify_chain(events: &[(String, String)]) -> Result<(), HashError> {
    if events.is_empty() {
        return Ok(());
    }

    // First event must chain from zero hash
    let zero_hash = "0".repeat(64);
    if events[0].1 != zero_hash {
        return Err(HashError::ChainBroken { sequence: 1 });
    }

    // Verify sequence continuity and hashes
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

/// Detect gaps in a sequence of events.
///
/// Returns the first missing sequence number, or None if the sequence is continuous.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_hash_deterministic() {
        let id = uuid::Uuid::nil();
        let ts = DateTime::parse_from_rfc3339("2026-03-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let payload = serde_json::json!({"n": "t"});
        let zero_hash = "0".repeat(64);

        let h1 = compute_hash(&id, ts, "created", &payload, "u1", &zero_hash).unwrap();
        let h2 = compute_hash(&id, ts, "created", &payload, "u1", &zero_hash).unwrap();

        assert_eq!(h1, h2);
        assert_ne!(h1, zero_hash);
    }

    #[test]
    fn compute_hash_changes_with_payload() {
        let id = uuid::Uuid::nil();
        let ts = DateTime::parse_from_rfc3339("2026-03-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let zero_hash = "0".repeat(64);

        let h1 = compute_hash(&id, ts, "created", &serde_json::json!({"n": "t"}), "u1", &zero_hash).unwrap();
        let h2 = compute_hash(&id, ts, "created", &serde_json::json!({"n": "x"}), "u1", &zero_hash).unwrap();

        assert_ne!(h1, h2);
    }

    #[test]
    fn verify_chain_empty() {
        verify_chain(&[]).unwrap();
    }

    #[test]
    fn verify_chain_single() {
        let zero_hash = "0".repeat(64);
        let hash = "abc123".to_string();
        verify_chain(&[(hash, zero_hash)]).unwrap();
    }

    #[test]
    fn verify_chain_two_events() {
        let zero_hash = "0".repeat(64);
        let h1 = "abc123".to_string();
        let h2 = "def456".to_string();

        verify_chain(&[(h1.clone(), zero_hash), (h2, h1)]).unwrap();
    }

    #[test]
    fn detect_gaps_no_gap() {
        assert_eq!(detect_gaps(&[1, 2, 3, 4, 5]), None);
    }

    #[test]
    fn detect_gaps_with_gap() {
        assert_eq!(detect_gaps(&[1, 2, 4, 5]), Some(3));
    }

    #[test]
    fn detect_gaps_empty() {
        assert_eq!(detect_gaps(&[]), None);
    }
}
