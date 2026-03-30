//! Hash utilities - blake3 hash chain (3-5x faster than SHA-256).

use blake3::Hasher;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::HashError;

pub const ZERO_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

pub fn compute_hash(
    id: &Uuid,
    timestamp: DateTime<Utc>,
    entity_type: &str,
    payload: &serde_json::Value,
    actor: &str,
    prev_hash: &str,
) -> Result<String, HashError> {
    let mut hasher = Hasher::new();
    hasher.update(id.as_bytes());
    hasher.update(timestamp.to_rfc3339().as_bytes());
    hasher.update(entity_type.as_bytes());
    let payload_bytes =
        serde_json::to_string(payload).map_err(|_| HashError::InvalidHashLength(0))?;
    hasher.update(payload_bytes.as_bytes());
    hasher.update(actor.as_bytes());
    let prev_bytes =
        decode_hex(prev_hash).map_err(|_| HashError::InvalidHashLength(prev_hash.len()))?;
    if prev_bytes.len() != 32 {
        return Err(HashError::InvalidHashLength(prev_bytes.len()));
    }
    hasher.update(&prev_bytes);
    Ok(hasher.finalize().to_hex().to_string())
}

fn decode_hex(s: &str) -> Result<Vec<u8>, ()> {
    if s.len() % 2 != 0 {
        return Err(());
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    for chunk in s.as_bytes().chunks(2) {
        let high = match chunk[0] {
            b @ b'0'..=b'9' => b - b'0',
            b @ b'a'..=b'f' => b - b'a' + 10,
            b @ b'A'..=b'F' => b - b'A' + 10,
            _ => return Err(()),
        };
        let low = match chunk[1] {
            b @ b'0'..=b'9' => b - b'0',
            b @ b'a'..=b'f' => b - b'a' + 10,
            b @ b'A'..=b'F' => b - b'A' + 10,
            _ => return Err(()),
        };
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

pub fn verify_chain(events: &[(String, String)]) -> Result<(), HashError> {
    if events.is_empty() {
        return Ok(());
    }
    let zero = ZERO_HASH.to_string();
    if events[0].1 != zero && events[0].1 != "0".repeat(64) {
        return Err(HashError::ChainBroken { sequence: 1 });
    }
    for (i, (_, prev_hash)) in events.iter().enumerate().skip(1) {
        if prev_hash != &events[i - 1].0 {
            return Err(HashError::ChainBroken {
                sequence: (i + 1) as i64,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_hash_deterministic() {
        let id = Uuid::nil();
        let ts = DateTime::parse_from_rfc3339("2026-03-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let payload = serde_json::json!({"n": "t"});
        let h1 = compute_hash(&id, ts, "test", &payload, "u1", ZERO_HASH).unwrap();
        let h2 = compute_hash(&id, ts, "test", &payload, "u1", ZERO_HASH).unwrap();
        assert_eq!(h1, h2);
        assert_ne!(h1, ZERO_HASH);
    }

    #[test]
    fn verify_chain_empty() {
        verify_chain(&[]).unwrap();
    }

    #[test]
    fn verify_chain_two_events() {
        let zero = "0".repeat(64);
        let h1 = "abc123".to_string();
        let h2 = "def456".to_string();
        verify_chain(&[(h1.clone(), zero), (h2, h1)]).unwrap();
    }

    #[test]
    fn decode_hex_valid() {
        assert_eq!(decode_hex("ff").unwrap(), vec![255]);
        assert_eq!(
            decode_hex("deadbeef").unwrap(),
            vec![0xde, 0xad, 0xbe, 0xef]
        );
    }
}
