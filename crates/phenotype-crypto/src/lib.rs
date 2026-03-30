//! Cryptographic utilities for Phenotype.
//!
//! Provides hashing, UUID generation, HMAC, and constant-time comparison.

use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Compute SHA-256 hash of data, returning hex-encoded result.
pub fn hash_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Compute BLAKE3 hash of data, returning hex-encoded result.
pub fn hash_blake3(data: &[u8]) -> String {
    hex::encode(blake3::hash(data).as_bytes())
}

/// Generate a UUID v4 (random).
pub fn generate_uuid_v4() -> String {
    Uuid::new_v4().to_string()
}

/// Generate a UUID v7 (timestamp-based, sortable).
pub fn generate_uuid_v7() -> String {
    Uuid::now_v7().to_string()
}

/// Compute HMAC-SHA256 of data using the provided key.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC can take any size key");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// Constant-time equality comparison.
///
/// Uses subtle's ConstantTimeEq to prevent timing attacks.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    use subtle::ConstantTimeEq;
    a.ct_eq(b).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_sha256() {
        let input = b"hello world";
        let result = hash_sha256(input);
        assert_eq!(
            result,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_hash_blake3() {
        let input = b"hello world";
        let result = hash_blake3(input);
        assert_eq!(
            result,
            "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"
        );
    }

    #[test]
    fn test_generate_uuid_v4() {
        let uuid = generate_uuid_v4();
        assert_eq!(uuid.len(), 36); // standard UUID string length
        assert!(uuid.contains('-'));
    }

    #[test]
    fn test_generate_uuid_v7() {
        let uuid = generate_uuid_v7();
        assert_eq!(uuid.len(), 36);
        assert!(uuid.contains('-'));
    }

    #[test]
    fn test_hmac_sha256() {
        let key = b"secret";
        let data = b"message";
        let result = hmac_sha256(key, data);
        assert_eq!(result.len(), 32); // SHA256 is 32 bytes
    }

    #[test]
    fn test_hmac_sha256_deterministic() {
        let key = b"secret";
        let data = b"message";
        let result1 = hmac_sha256(key, data);
        let result2 = hmac_sha256(key, data);
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_constant_time_eq_equal() {
        let a = b"hello";
        let b = b"hello";
        assert!(constant_time_eq(a, b));
    }

    #[test]
    fn test_constant_time_eq_not_equal() {
        let a = b"hello";
        let b = b"world";
        assert!(!constant_time_eq(a, b));
    }

    #[test]
    fn test_constant_time_eq_different_lengths() {
        let a = b"hello";
        let b = b"hi";
        assert!(!constant_time_eq(a, b));
    }
}
