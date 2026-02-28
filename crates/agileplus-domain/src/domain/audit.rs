use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::DomainError;

// ---------------------------------------------------------------------------
// hex_bytes serde module for [u8; 32]
// ---------------------------------------------------------------------------

pub mod hex_bytes {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        serializer.serialize_str(&hex_string)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.len() != 64 {
            return Err(serde::de::Error::custom("expected 64 hex characters"));
        }
        let mut bytes = [0u8; 32];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte =
                u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(serde::de::Error::custom)?;
        }
        Ok(bytes)
    }
}

// ---------------------------------------------------------------------------
// AuditEntry (T019)
// ---------------------------------------------------------------------------

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub evidence_id: i64,
    pub fr_id: String,
}

impl PartialEq for AuditEntry {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for AuditEntry {}

impl AuditEntry {
    pub fn genesis(feature_id: i64, actor: impl Into<String>) -> Self {
        let id = 0;
        let timestamp = Utc::now();
        let actor = actor.into();
        let transition = "genesis".to_string();
        let evidence_refs = Vec::new();
        let prev_hash = [0u8; 32];
        let hash = hash_entry(
            id,
            &timestamp,
            &actor,
            &transition,
            &evidence_refs,
            &prev_hash,
        );
        Self {
            id,
            feature_id,
            wp_id: None,
            timestamp,
            actor,
            transition,
            evidence_refs,
            prev_hash,
            hash,
        }
    }

    pub fn new(
        id: i64,
        feature_id: i64,
        wp_id: Option<i64>,
        actor: impl Into<String>,
        transition: impl Into<String>,
        evidence_refs: Vec<EvidenceRef>,
        prev_hash: [u8; 32],
    ) -> Self {
        let timestamp = Utc::now();
        let actor = actor.into();
        let transition = transition.into();
        let hash = hash_entry(
            id,
            &timestamp,
            &actor,
            &transition,
            &evidence_refs,
            &prev_hash,
        );
        Self {
            id,
            feature_id,
            wp_id,
            timestamp,
            actor,
            transition,
            evidence_refs,
            prev_hash,
            hash,
        }
    }

    pub fn compute_hash(&self) -> [u8; 32] {
        hash_entry(
            self.id,
            &self.timestamp,
            &self.actor,
            &self.transition,
            &self.evidence_refs,
            &self.prev_hash,
        )
    }

    pub fn verify_hash(&self) -> bool {
        self.hash == self.compute_hash()
    }
}

// ---------------------------------------------------------------------------
// hash_entry (T020)
// ---------------------------------------------------------------------------

pub fn hash_entry(
    id: i64,
    timestamp: &DateTime<Utc>,
    actor: &str,
    transition: &str,
    evidence_refs: &[EvidenceRef],
    prev_hash: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(id.to_be_bytes());
    hasher.update(b"\x00");
    hasher.update(timestamp.to_rfc3339().as_bytes());
    hasher.update(b"\x00");
    hasher.update(actor.as_bytes());
    hasher.update(b"\x00");
    hasher.update(transition.as_bytes());
    hasher.update(b"\x00");
    let mut sorted_refs = evidence_refs.to_vec();
    sorted_refs.sort_by_key(|r| r.evidence_id);
    for eref in &sorted_refs {
        hasher.update(eref.evidence_id.to_be_bytes());
        hasher.update(eref.fr_id.as_bytes());
        hasher.update(b"\x00");
    }
    hasher.update(prev_hash);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

// ---------------------------------------------------------------------------
// AuditChainError & verify_chain (T021)
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum AuditChainError {
    #[error("invalid genesis entry {entry_id}: prev_hash not zero")]
    InvalidGenesis {
        entry_id: i64,
        expected_prev_hash: [u8; 32],
        actual_prev_hash: [u8; 32],
    },
    #[error("hash mismatch at entry {entry_id}")]
    HashMismatch {
        entry_id: i64,
        expected: [u8; 32],
        actual: [u8; 32],
    },
    #[error("broken link at entry {entry_id}")]
    BrokenLink {
        entry_id: i64,
        expected_prev_hash: [u8; 32],
        actual_prev_hash: [u8; 32],
    },
    #[error("non-sequential ID at {entry_id}, expected {expected_id}")]
    NonSequentialId { entry_id: i64, expected_id: i64 },
}

/// Verifies the integrity of an audit chain. Returns the number of verified entries.
pub fn verify_chain(entries: &[AuditEntry]) -> Result<usize, AuditChainError> {
    if entries.is_empty() {
        return Ok(0);
    }

    // Check genesis
    let genesis = &entries[0];
    if genesis.prev_hash != [0u8; 32] {
        return Err(AuditChainError::InvalidGenesis {
            entry_id: genesis.id,
            expected_prev_hash: [0u8; 32],
            actual_prev_hash: genesis.prev_hash,
        });
    }

    for (i, entry) in entries.iter().enumerate() {
        // Check sequential IDs
        let expected_id = i as i64;
        if entry.id != expected_id {
            return Err(AuditChainError::NonSequentialId {
                entry_id: entry.id,
                expected_id,
            });
        }

        // Check hash integrity
        let computed = entry.compute_hash();
        if entry.hash != computed {
            return Err(AuditChainError::HashMismatch {
                entry_id: entry.id,
                expected: computed,
                actual: entry.hash,
            });
        }

        // Check chain link (skip genesis)
        if i > 0 {
            let prev = &entries[i - 1];
            if entry.prev_hash != prev.hash {
                return Err(AuditChainError::BrokenLink {
                    entry_id: entry.id,
                    expected_prev_hash: prev.hash,
                    actual_prev_hash: entry.prev_hash,
                });
            }
        }
    }

    Ok(entries.len())
}

// ---------------------------------------------------------------------------
// AuditChain (T021)
// ---------------------------------------------------------------------------

pub struct AuditChain {
    entries: Vec<AuditEntry>,
}

impl AuditChain {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(
        &mut self,
        feature_id: i64,
        wp_id: Option<i64>,
        actor: &str,
        transition: &str,
        evidence_refs: &[EvidenceRef],
    ) -> Result<&AuditEntry, DomainError> {
        let id = self.entries.len() as i64;
        let prev_hash = self.last_hash();

        if id == 0 && transition != "genesis" {
            // First entry must be genesis — auto-create one.
            let genesis = AuditEntry::genesis(feature_id, actor);
            self.entries.push(genesis);
            return self.append(feature_id, wp_id, actor, transition, evidence_refs);
        }

        let entry = AuditEntry::new(
            id,
            feature_id,
            wp_id,
            actor,
            transition,
            evidence_refs.to_vec(),
            prev_hash,
        );
        self.entries.push(entry);
        Ok(self.entries.last().unwrap())
    }

    pub fn verify(&self) -> Result<usize, AuditChainError> {
        verify_chain(&self.entries)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn last_hash(&self) -> [u8; 32] {
        self.entries.last().map_or([0u8; 32], |e| e.hash)
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }
}

impl Default for AuditChain {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests (T024)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Audit chain tests --

    #[test]
    fn test_genesis_entry() {
        let entry = AuditEntry::genesis(1, "alice");
        assert_eq!(entry.id, 0);
        assert_eq!(entry.prev_hash, [0u8; 32]);
        assert_eq!(entry.transition, "genesis");
        assert!(entry.verify_hash());
    }

    #[test]
    fn test_new_entry_hash_valid() {
        let entry = AuditEntry::new(
            1,
            1,
            Some(10),
            "bob",
            "created -> specified",
            vec![],
            [0u8; 32],
        );
        assert!(entry.verify_hash());
    }

    #[test]
    fn test_chain_verification_valid() {
        let mut chain = AuditChain::new();
        chain.append(1, None, "alice", "genesis", &[]).unwrap();
        chain
            .append(1, Some(1), "alice", "created -> specified", &[])
            .unwrap();
        chain
            .append(1, Some(1), "bob", "specified -> researched", &[])
            .unwrap();
        assert_eq!(chain.verify().unwrap(), 3);
    }

    #[test]
    fn test_tamper_detection() {
        let mut chain = AuditChain::new();
        chain.append(1, None, "alice", "genesis", &[]).unwrap();
        chain
            .append(1, Some(1), "alice", "created -> specified", &[])
            .unwrap();

        // Tamper with the first entry's actor
        let entries = &mut chain.entries;
        entries[0].actor = "mallory".to_string();
        // Hash no longer matches
        assert!(chain.verify().is_err());
    }

    #[test]
    fn test_broken_link() {
        let genesis = AuditEntry::genesis(1, "alice");
        let mut second = AuditEntry::new(
            1,
            1,
            None,
            "alice",
            "created -> specified",
            vec![],
            genesis.hash,
        );
        // Corrupt prev_hash
        second.prev_hash = [0xffu8; 32];
        second.hash = second.compute_hash();

        let entries = vec![genesis, second];
        let result = verify_chain(&entries);
        assert!(matches!(result, Err(AuditChainError::BrokenLink { .. })));
    }

    #[test]
    fn test_empty_chain() {
        assert_eq!(verify_chain(&[]).unwrap(), 0);
    }

    #[test]
    fn test_hash_determinism() {
        let ts = Utc::now();
        let h1 = hash_entry(1, &ts, "alice", "genesis", &[], &[0u8; 32]);
        let h2 = hash_entry(1, &ts, "alice", "genesis", &[], &[0u8; 32]);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_field_sensitivity_actor() {
        let ts = Utc::now();
        let h1 = hash_entry(1, &ts, "alice", "genesis", &[], &[0u8; 32]);
        let h2 = hash_entry(1, &ts, "bob", "genesis", &[], &[0u8; 32]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_field_sensitivity_id() {
        let ts = Utc::now();
        let h1 = hash_entry(1, &ts, "alice", "genesis", &[], &[0u8; 32]);
        let h2 = hash_entry(2, &ts, "alice", "genesis", &[], &[0u8; 32]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_field_sensitivity_transition() {
        let ts = Utc::now();
        let h1 = hash_entry(1, &ts, "alice", "genesis", &[], &[0u8; 32]);
        let h2 = hash_entry(1, &ts, "alice", "created -> specified", &[], &[0u8; 32]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_field_sensitivity_prev_hash() {
        let ts = Utc::now();
        let h1 = hash_entry(1, &ts, "alice", "genesis", &[], &[0u8; 32]);
        let h2 = hash_entry(1, &ts, "alice", "genesis", &[], &[1u8; 32]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_field_sensitivity_evidence_refs() {
        let ts = Utc::now();
        let refs = vec![EvidenceRef {
            evidence_id: 1,
            fr_id: "FR-01".to_string(),
        }];
        let h1 = hash_entry(1, &ts, "alice", "genesis", &[], &[0u8; 32]);
        let h2 = hash_entry(1, &ts, "alice", "genesis", &refs, &[0u8; 32]);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_non_sequential_id() {
        let genesis = AuditEntry::genesis(1, "alice");
        let mut second = AuditEntry::new(5, 1, None, "alice", "t", vec![], genesis.hash);
        // Fix hash so the only error is non-sequential ID
        second.hash = second.compute_hash();
        let entries = vec![genesis, second];
        assert!(matches!(
            verify_chain(&entries),
            Err(AuditChainError::NonSequentialId { .. })
        ));
    }

    #[test]
    fn test_partial_eq_by_hash() {
        let a = AuditEntry::genesis(1, "alice");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_audit_chain_len_and_empty() {
        let chain = AuditChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);
    }

    #[test]
    fn test_audit_chain_last_hash_empty() {
        let chain = AuditChain::new();
        assert_eq!(chain.last_hash(), [0u8; 32]);
    }

    #[test]
    fn test_audit_chain_auto_genesis() {
        let mut chain = AuditChain::new();
        // Appending a non-genesis entry to empty chain should auto-create genesis first
        chain
            .append(1, None, "alice", "created -> specified", &[])
            .unwrap();
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.entries()[0].transition, "genesis");
        assert!(chain.verify().is_ok());
    }

    #[test]
    fn test_hex_bytes_roundtrip() {
        let entry = AuditEntry::genesis(1, "alice");
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: AuditEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry.hash, deserialized.hash);
        assert_eq!(entry.prev_hash, deserialized.prev_hash);
    }
}
