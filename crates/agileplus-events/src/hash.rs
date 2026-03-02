//! SHA-256 hash chain computation and verification.

use agileplus_domain::domain::event::Event;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error)]
pub enum HashError {
    #[error("Hash chain broken at sequence {sequence}")]
    ChainBroken { sequence: i64 },
    #[error("Invalid hash length: expected 32, got {0}")]
    InvalidHashLength(usize),
    #[error("Hash mismatch at sequence {sequence}")]
    HashMismatch { sequence: i64 },
}

/// Compute SHA-256 hash for a new event.
///
/// Hash inputs (in order, length-prefixed where noted):
/// 1. entity_id (8 bytes, big-endian)
/// 2. entity_type (length-prefixed UTF-8)
/// 3. event_type (length-prefixed UTF-8)
/// 4. payload (length-prefixed JSON)
/// 5. timestamp (length-prefixed ISO 8601)
/// 6. actor (length-prefixed UTF-8)
/// 7. prev_hash (32 bytes)
pub fn compute_hash(
    entity_id: i64,
    entity_type: &str,
    event_type: &str,
    payload: &serde_json::Value,
    timestamp: DateTime<Utc>,
    actor: &str,
    prev_hash: &[u8; 32],
) -> Result<[u8; 32], HashError> {
    let mut hasher = Sha256::new();

    hasher.update(entity_id.to_be_bytes());

    hasher.update((entity_type.len() as u32).to_be_bytes());
    hasher.update(entity_type.as_bytes());

    hasher.update((event_type.len() as u32).to_be_bytes());
    hasher.update(event_type.as_bytes());

    let payload_json =
        serde_json::to_string(payload).map_err(|_| HashError::InvalidHashLength(0))?;
    hasher.update((payload_json.len() as u32).to_be_bytes());
    hasher.update(payload_json.as_bytes());

    let timestamp_str = timestamp.to_rfc3339();
    hasher.update((timestamp_str.len() as u32).to_be_bytes());
    hasher.update(timestamp_str.as_bytes());

    hasher.update((actor.len() as u32).to_be_bytes());
    hasher.update(actor.as_bytes());

    hasher.update(prev_hash);

    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result[..]);
    Ok(hash)
}

/// Verify the integrity of an event chain.
///
/// Ensures each event's hash is correctly computed and chains to its predecessor.
pub fn verify_chain(events: &[Event]) -> Result<(), HashError> {
    if events.is_empty() {
        return Ok(());
    }

    // First event must chain from zeros
    if events[0].prev_hash != [0u8; 32] {
        return Err(HashError::ChainBroken {
            sequence: events[0].sequence,
        });
    }

    let expected = compute_hash(
        events[0].entity_id,
        &events[0].entity_type,
        &events[0].event_type,
        &events[0].payload,
        events[0].timestamp,
        &events[0].actor,
        &[0u8; 32],
    )?;
    if expected != events[0].hash {
        return Err(HashError::HashMismatch {
            sequence: events[0].sequence,
        });
    }

    for i in 1..events.len() {
        let prev = &events[i - 1];
        let curr = &events[i];

        if curr.sequence != prev.sequence + 1 {
            return Err(HashError::ChainBroken {
                sequence: curr.sequence,
            });
        }
        if curr.prev_hash != prev.hash {
            return Err(HashError::ChainBroken {
                sequence: curr.sequence,
            });
        }

        let expected = compute_hash(
            curr.entity_id,
            &curr.entity_type,
            &curr.event_type,
            &curr.payload,
            curr.timestamp,
            &curr.actor,
            &prev.hash,
        )?;
        if expected != curr.hash {
            return Err(HashError::HashMismatch {
                sequence: curr.sequence,
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
        let ts = DateTime::parse_from_rfc3339("2026-03-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let h1 = compute_hash(1, "Feature", "created", &serde_json::json!({"n": "t"}), ts, "u1", &[0u8; 32]).unwrap();
        let h2 = compute_hash(1, "Feature", "created", &serde_json::json!({"n": "t"}), ts, "u1", &[0u8; 32]).unwrap();
        assert_eq!(h1, h2);
        assert_ne!(h1, [0u8; 32]);
    }

    #[test]
    fn verify_chain_empty() {
        verify_chain(&[]).unwrap();
    }

    #[test]
    fn verify_chain_single() {
        let ts = DateTime::parse_from_rfc3339("2026-03-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let payload = serde_json::json!({"x": 1});
        let hash = compute_hash(1, "F", "c", &payload, ts, "a", &[0u8; 32]).unwrap();
        let event = Event {
            id: 1,
            entity_type: "F".into(),
            entity_id: 1,
            event_type: "c".into(),
            payload,
            actor: "a".into(),
            timestamp: ts,
            prev_hash: [0u8; 32],
            hash,
            sequence: 1,
        };
        verify_chain(&[event]).unwrap();
    }

    #[test]
    fn verify_chain_two_events() {
        let ts = DateTime::parse_from_rfc3339("2026-03-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let p1 = serde_json::json!({"v": 1});
        let h1 = compute_hash(1, "F", "c", &p1, ts, "a", &[0u8; 32]).unwrap();
        let e1 = Event {
            id: 1, entity_type: "F".into(), entity_id: 1, event_type: "c".into(),
            payload: p1, actor: "a".into(), timestamp: ts,
            prev_hash: [0u8; 32], hash: h1, sequence: 1,
        };

        let p2 = serde_json::json!({"v": 2});
        let h2 = compute_hash(1, "F", "u", &p2, ts, "a", &h1).unwrap();
        let e2 = Event {
            id: 2, entity_type: "F".into(), entity_id: 1, event_type: "u".into(),
            payload: p2, actor: "a".into(), timestamp: ts,
            prev_hash: h1, hash: h2, sequence: 2,
        };

        verify_chain(&[e1, e2]).unwrap();
    }

    #[test]
    fn verify_chain_detects_tamper() {
        let ts = DateTime::parse_from_rfc3339("2026-03-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let p1 = serde_json::json!({"v": 1});
        let h1 = compute_hash(1, "F", "c", &p1, ts, "a", &[0u8; 32]).unwrap();
        let mut e1 = Event {
            id: 1, entity_type: "F".into(), entity_id: 1, event_type: "c".into(),
            payload: p1, actor: "a".into(), timestamp: ts,
            prev_hash: [0u8; 32], hash: h1, sequence: 1,
        };
        // Tamper
        e1.hash[0] ^= 0xFF;
        assert!(verify_chain(&[e1]).is_err());
    }
}
