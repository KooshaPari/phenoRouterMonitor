//! Hashing utilities — SHA-256 and Blake3.
//! @trace FR-PHENO-CRYPTO-001
//! @trace FR-PHENO-CRYPTO-002

use sha2::{Digest, Sha256};

/// Supported hash algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

/// Compute SHA-256 hash of bytes, returned as hex string.
pub fn sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Compute Blake3 hash of bytes, returned as hex string.
pub fn blake3_hash(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hash.to_hex().to_string()
}

/// Alias for `sha256_hash()`. Computes SHA-256 hash of bytes.
/// @trace FR-PHENO-CRYPTO-001
pub fn hash_sha256(data: &[u8]) -> String {
    sha256_hash(data)
}

/// Alias for `blake3_hash()`. Computes Blake3 hash of bytes.
/// @trace FR-PHENO-CRYPTO-002
pub fn hash_blake3(data: &[u8]) -> String {
    blake3_hash(data)
}

/// Compute a content-addressable ID using the specified algorithm.
/// Format: `{algorithm}:{hex_hash}`
pub fn content_id(data: &[u8], algorithm: HashAlgorithm) -> String {
    match algorithm {
        HashAlgorithm::Sha256 => format!("sha256:{}", sha256_hash(data)),
        HashAlgorithm::Blake3 => format!("blake3:{}", blake3_hash(data)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // @trace FR-PHENO-CRYPTO-001
    #[test]
    fn sha256_known_vector() {
        let hash = sha256_hash(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    // @trace FR-PHENO-CRYPTO-001
    #[test]
    fn sha256_hello() {
        let hash = sha256_hash(b"hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    // @trace FR-PHENO-CRYPTO-001
    #[test]
    fn hash_sha256_alias() {
        let data = b"test data";
        let h1 = sha256_hash(data);
        let h2 = hash_sha256(data);
        assert_eq!(h1, h2);
    }

    // @trace FR-PHENO-CRYPTO-002
    #[test]
    fn blake3_deterministic() {
        let h1 = blake3_hash(b"test data");
        let h2 = blake3_hash(b"test data");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    // @trace FR-PHENO-CRYPTO-002
    #[test]
    fn hash_blake3_alias() {
        let data = b"test data";
        let h1 = blake3_hash(data);
        let h2 = hash_blake3(data);
        assert_eq!(h1, h2);
    }

    // @trace FR-PHENO-CRYPTO-002
    #[test]
    fn blake3_empty_string() {
        let hash = blake3_hash(b"");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn content_id_format() {
        let id = content_id(b"payload", HashAlgorithm::Sha256);
        assert!(id.starts_with("sha256:"));
        assert_eq!(id.len(), 7 + 64);

        let id = content_id(b"payload", HashAlgorithm::Blake3);
        assert!(id.starts_with("blake3:"));
        assert_eq!(id.len(), 7 + 64);
    }

    #[test]
    fn different_algorithms_produce_different_hashes() {
        let s = sha256_hash(b"same input");
        let b = blake3_hash(b"same input");
        assert_ne!(s, b);
    }

    // @trace FR-PHENO-CRYPTO-001
    #[test]
    fn sha256_large_data() {
        let large_data = vec![42u8; 1_000_000];
        let hash = sha256_hash(&large_data);
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // @trace FR-PHENO-CRYPTO-002
    #[test]
    fn blake3_large_data() {
        let large_data = vec![99u8; 1_000_000];
        let hash = blake3_hash(&large_data);
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
