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
    decrypt_aes_gcm, encrypt_aes_gcm, decrypt_aes_gcm_hex, encrypt_aes_gcm_hex, CryptoError,
};
pub use keys::{generate_salt, generate_salt_hex, Pbkdf2Kdf};
pub use signatures::{compute_hmac, compute_hmac_hex, verify_hmac, verify_hmac_hex};
