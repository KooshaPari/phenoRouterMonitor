//! Hash utilities for event sourcing (SHA-256 chain).

use chrono::{DateTime, Utc};
use hex;
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::HashError;

const HASH_HEX_LEN: usize = 64;

fn validate_hex_hash(s: &str) -> Result<(), HashError> {
    if s.len() != HASH_HEX_LEN {
        return Err(HashError::InvalidHashLength(s.len()));
    }
    if !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(HashError::InvalidHashLength(s.len()));
    }
    Ok(())
}

/// Compute the canonical SHA-256 hash (64 lowercase hex chars) for an event.
pub fn compute_hash(
    id: &Uuid,
    timestamp: DateTime<Utc>,
    entity_type: &str,
    payload: &Value,
    actor: &str,
    prev_hash: &str,
) -> Result<String, HashError> {
    validate_hex_hash(prev_hash)?;
    let payload_str = serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string());
    let canonical = format!(
        "{}|{}|{}|{}|{}|{}",
        id,
        timestamp.to_rfc3339(),
        entity_type,
        payload_str,
        actor,
        prev_hash
    );
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    Ok(hex::encode(hasher.finalize()))
}

/// Public alias for [`compute_hash`].
pub fn compute_event_hash(
    id: &Uuid,
    timestamp: DateTime<Utc>,
    entity_type: &str,
    payload: &Value,
    actor: &str,
    prev_hash: &str,
) -> Result<String, HashError> {
    compute_hash(id, timestamp, entity_type, payload, actor, prev_hash)
}

/// Verify two hex-encoded hashes match (both must be 64-char hex).
pub fn verify_event_hash(computed_hex: &str, expected_hex: &str) -> Result<(), HashError> {
    validate_hex_hash(computed_hex)?;
    validate_hex_hash(expected_hex)?;
    if computed_hex != expected_hex {
        return Err(HashError::HashMismatch { sequence: 0 });
    }
    Ok(())
}

/// Verify a linear hash chain: each row is `(event_hash, prev_hash)`.
/// The first row's `prev_hash` must be the genesis string (`"0"` × 64).
pub fn verify_chain(chain: &[(String, String)]) -> Result<(), HashError> {
    let genesis = "0".repeat(HASH_HEX_LEN);
    if chain.is_empty() {
        return Ok(());
    }
    validate_hex_hash(&chain[0].0)?;
    validate_hex_hash(&chain[0].1)?;
    if chain[0].1 != genesis {
        return Err(HashError::ChainBroken { sequence: 1 });
    }
    for i in 1..chain.len() {
        validate_hex_hash(&chain[i].0)?;
        validate_hex_hash(&chain[i].1)?;
        if chain[i].1 != chain[i - 1].0 {
            return Err(HashError::ChainBroken {
                sequence: i as i64 + 1,
            });
        }
    }
    Ok(())
}
