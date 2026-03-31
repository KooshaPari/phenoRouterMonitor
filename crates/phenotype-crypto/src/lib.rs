//! Cryptographic utilities for the Phenotype ecosystem.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("hash error: {0}")]
    Hash(String),
    #[error("encryption error: {0}")]
    Encryption(String),
    #[error("decryption error: {0}")]
    Decryption(String),
    #[error("key derivation error: {0}")]
    KeyDerivation(String),
    #[error("signature error: {0}")]
    Signature(String),
}

pub type Result<T> = std::result::Result<T, CryptoError>;

/// SHA-256 hash of the given data.
pub fn sha256(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// BLAKE3 hash of the given data.
pub fn blake3_hash(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hash.to_hex().to_string()
}

/// Derive a key from a password using PBKDF2.
pub fn derive_key(password: &[u8], salt: &[u8], iterations: u32, key_len: usize) -> Result<Vec<u8>> {
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;
    let mut key = vec![0u8; key_len];
    pbkdf2_hmac::<Sha256>(password, salt, iterations, &mut key);
    Ok(key)
}

/// Generate a random key of the specified length.
pub fn generate_key(len: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut key = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Compute HMAC-SHA256 of the given data with the specified key.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> String {
    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<sha2::Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    hex::encode(mac.finalize().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let hash = sha256(b"hello");
        assert_eq!(hash.len(), 64);
        let hash2 = sha256(b"hello");
        assert_eq!(hash, hash2);
        let hash3 = sha256(b"world");
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_blake3_hash() {
        let hash = blake3_hash(b"hello");
        assert_eq!(hash.len(), 64);
        let hash2 = blake3_hash(b"hello");
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_derive_key() {
        let key = derive_key(b"password", b"salt", 1000, 32).unwrap();
        assert_eq!(key.len(), 32);
        let key2 = derive_key(b"password", b"salt", 1000, 32).unwrap();
        assert_eq!(key, key2);
    }

    #[test]
    fn test_generate_key() {
        let key = generate_key(32);
        assert_eq!(key.len(), 32);
        let key2 = generate_key(32);
        assert_ne!(key, key2);
    }

    #[test]
    fn test_hmac_sha256() {
        let mac = hmac_sha256(b"key", b"message");
        assert_eq!(mac.len(), 64);
        let mac2 = hmac_sha256(b"key", b"message");
        assert_eq!(mac, mac2);
        let mac3 = hmac_sha256(b"key", b"other");
        assert_ne!(mac, mac3);
    }
}
