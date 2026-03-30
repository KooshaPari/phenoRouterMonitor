//! SHA-256 hash chain for event sourcing.
//!
//! Provides a generic hash chain builder with configurable encoding.
//!
//! # Features
//!
//! - SHA-256 hash computation
//! - Configurable encoding (binary, hex, base64)
//! - Chain verification
//! - Event ordering
//!
//! # Example
//!
//! ```rust,ignore
//! use phenotype_hash_chain::{HashChain, HashChainBuilder};
//!
//! let mut chain = HashChainBuilder::new()
//!     .with_genesis_hash([0u8; 32])
//!     .build();
//!
//! chain.push(event_bytes)?;
//! let hash = chain.head()?;
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Encoding for hash values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    /// Binary encoding (raw bytes).
    Binary,
    /// Hex string encoding.
    Hex,
    /// Base64 encoding.
    Base64,
}

/// A SHA-256 hash chain for event sourcing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashChain {
    /// Current head of the chain.
    pub head: [u8; 32],
    /// Length of the chain.
    pub length: u64,
}

impl HashChain {
    /// Create a new chain with a genesis hash.
    pub fn new(genesis: [u8; 32]) -> Self {
        Self {
            head: genesis,
            length: 0,
        }
    }

    /// Push a new event onto the chain.
    pub fn push(&mut self, event: &[u8]) {
        self.head = Self::hash(&self.head, event);
        self.length += 1;
    }

    /// Get the current head hash.
    pub fn head(&self) -> [u8; 32] {
        self.head
    }

    /// Get the chain length.
    pub fn length(&self) -> u64 {
        self.length
    }

    /// Compute SHA-256 hash of previous_head + event.
    fn hash(previous: &[u8; 32], event: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(previous);
        hasher.update(event);
        hasher.finalize().into()
    }

    /// Verify a chain against expected head.
    pub fn verify(&self, events: &[Vec<u8>], genesis: [u8; 32]) -> bool {
        let mut expected = genesis;
        for event in events {
            let hash = Self::hash(&expected, event);
            expected = hash;
        }
        expected == self.head
    }
}

/// Builder for HashChain with encoding support.
#[derive(Debug, Clone)]
pub struct HashChainBuilder {
    encoding: Encoding,
    genesis: Option<[u8; 32]>,
}

impl HashChainBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            encoding: Encoding::Binary,
            genesis: None,
        }
    }

    /// Set the genesis hash.
    pub fn with_genesis_hash(mut self, hash: [u8; 32]) -> Self {
        self.genesis = Some(hash);
        self
    }

    /// Set the encoding.
    pub fn with_encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = encoding;
        self
    }

    /// Build the chain.
    pub fn build(self) -> HashChain {
        let genesis = self.genesis.unwrap_or([0u8; 32]);
        HashChain::new(genesis)
    }

    /// Encode a hash to the configured encoding.
    pub fn encode(&self, hash: &[u8; 32]) -> String {
        match self.encoding {
            Encoding::Binary => format!("{:02x?}", hash),
            Encoding::Hex => hex::encode(hash),
            Encoding::Base64 => base64::Engine::encode(&base64::engine::general_purpose::STANDARD, hash),
        }
    }
}

impl Default for HashChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_chain() {
        let chain = HashChain::new([0u8; 32]);
        assert_eq!(chain.length, 0);
        assert_eq!(chain.head, [0u8; 32]);
    }

    #[test]
    fn test_chain_push() {
        let mut chain = HashChain::new([0u8; 32]);
        chain.push(b"event1");
        assert_eq!(chain.length, 1);
        assert_ne!(chain.head, [0u8; 32]);
    }

    #[test]
    fn test_chain_verify() {
        let events = vec![b"event1".to_vec(), b"event2".to_vec()];
        let mut chain = HashChain::new([0u8; 32]);
        for e in &events {
            chain.push(e);
        }
        assert!(chain.verify(&events, [0u8; 32]));
    }

    #[test]
    fn test_chain_verify_fails() {
        let events = vec![b"event1".to_vec()];
        let mut chain = HashChain::new([0u8; 32]);
        chain.push(b"event1");
        assert!(chain.verify(&events, [0u8; 32]));
        assert!(!chain.verify(&[b"tampered".to_vec()], [0u8; 32]));
    }
}
