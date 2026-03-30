//! # Phenotype Crypto
//!
//! Comprehensive cryptographic utilities including hashing (SHA-256, Blake3),
//! symmetric encryption (AES-GCM), key derivation (PBKDF2),
//! HMAC signatures, and secure random generation.

pub mod hash;
pub mod encryption;
pub mod keys;
pub mod signatures;

pub use hash::{blake3_hash, content_id, sha256_hash, HashAlgorithm};
pub use encryption::{
    decrypt_aes_gcm, encrypt_aes_gcm, CryptoError,
};
pub use keys::{generate_salt, generate_salt_hex, Pbkdf2Kdf};
pub use signatures::{compute_hmac, verify_hmac};
