//! Audit domain types — immutable audit trail with hash-chain integrity.
//!
//! Traceability: FR-AUDIT-* / WP04

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::feature::hex_bytes;

/// A reference from an audit entry to a piece of evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub evidence_id: i64,
    pub fr_id: String,
}

/// An immutable audit log entry recording a state transition or action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub feature_id: i64,
    pub wp_id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub transition: String,
    pub evidence_refs: Vec<EvidenceRef>,
    #[serde(with = "hex_bytes")]
    pub prev_hash: [u8; 32],
    #[serde(with = "hex_bytes")]
    pub hash: [u8; 32],
}

/// Errors arising from audit chain verification.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AuditChainError {
    #[error("chain is empty")]
    EmptyChain,
    #[error("hash mismatch at entry {index}: expected {expected}, got {actual}")]
    HashMismatch {
        index: usize,
        expected: String,
        actual: String,
    },
    #[error("prev_hash mismatch at entry {index}")]
    PrevHashMismatch { index: usize },
}

/// Compute the SHA-256 hash for an audit entry.
#[must_use]
pub fn hash_entry(entry: &AuditEntry) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(entry.feature_id.to_le_bytes());
    if let Some(wp_id) = entry.wp_id {
        hasher.update(wp_id.to_le_bytes());
    }
    hasher.update(entry.timestamp.to_rfc3339().as_bytes());
    hasher.update(entry.actor.as_bytes());
    hasher.update(entry.transition.as_bytes());
    hasher.update(entry.prev_hash);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// An ordered audit chain with verification support.
#[derive(Debug, Clone)]
pub struct AuditChain {
    pub entries: Vec<AuditEntry>,
}

impl AuditChain {
    /// Verify the integrity of the entire audit chain.
    pub fn verify_chain(&self) -> Result<(), AuditChainError> {
        if self.entries.is_empty() {
            return Err(AuditChainError::EmptyChain);
        }
        for (i, entry) in self.entries.iter().enumerate() {
            // Verify prev_hash links
            if i > 0 {
                let prev = &self.entries[i - 1];
                if entry.prev_hash != prev.hash {
                    return Err(AuditChainError::PrevHashMismatch { index: i });
                }
            }
            // Verify self-hash
            let computed = hash_entry(entry);
            if computed != entry.hash {
                let expected = entry.hash.iter().map(|b| format!("{b:02x}")).collect();
                let actual = computed.iter().map(|b| format!("{b:02x}")).collect();
                return Err(AuditChainError::HashMismatch {
                    index: i,
                    expected,
                    actual,
                });
            }
        }
        Ok(())
    }
}
