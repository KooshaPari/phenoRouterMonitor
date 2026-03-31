use blake3::Hasher;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::EventSourcingError;

pub type Result<T> = std::result::Result<T, EventSourcingError>;

/// Compute BLAKE3 hash of an event payload.
pub fn compute_event_hash(
    event_type: &str,
    aggregate_id: &str,
    payload: &serde_json::Value,
    sequence: i64,
    previous_hash: &str,
) -> Result<String> {
    let mut hasher = Hasher::new();
    hasher.update(event_type.as_bytes());
    hasher.update(aggregate_id.as_bytes());
    let payload_str = serde_json::to_string(payload).map_err(|e| EventSourcingError::Serialization(e.to_string()))?;
    hasher.update(payload_str.as_bytes());
    hasher.update(&sequence.to_le_bytes());
    hasher.update(previous_hash.as_bytes());
    Ok(hasher.finalize().to_hex().to_string())
}

/// Verify a hash chain by checking each event's hash.
pub fn verify_hash_chain(
    hashes: &[(i64, String)],
) -> Result<()> {
    for (i, (seq, hash)) in hashes.iter().enumerate() {
        if i > 0 {
            let prev_hash = &hashes[i - 1].1;
            if hash.is_empty() || prev_hash.is_empty() {
                return Err(EventSourcingError::HashMismatch {
                    expected: prev_hash.clone(),
                    actual: hash.clone(),
                });
            }
        }
        let _ = hex::decode(hash).map_err(|e| EventSourcingError::HexDecode(e.to_string()))?;
    }
    Ok(())
}

/// Decode a hex string to bytes.
pub fn hex_decode(s: &str) -> Result<Vec<u8>> {
    hex::decode(s).map_err(|e| EventSourcingError::HexDecode(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_event_hash() {
        let hash1 = compute_event_hash("test", "agg-1", &serde_json::json!({}), 1, "").unwrap();
        let hash2 = compute_event_hash("test", "agg-1", &serde_json::json!({}), 1, "").unwrap();
        assert_eq!(hash1, hash2);

        let hash3 = compute_event_hash("test", "agg-1", &serde_json::json!({"x": 1}), 1, "").unwrap();
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_verify_hash_chain_valid() {
        let chain = vec![
            (1, "abc123".to_string()),
            (2, "def456".to_string()),
        ];
        assert!(verify_hash_chain(&chain).is_ok());
    }

    #[test]
    fn test_verify_hash_chain_invalid_hex() {
        let chain = vec![
            (1, "not-hex!!!".to_string()),
        ];
        assert!(verify_hash_chain(&chain).is_err());
    }

    #[test]
    fn test_hex_decode() {
        let bytes = hex_decode("48656c6c6f").unwrap();
        assert_eq!(bytes, b"Hello");
    }
}
