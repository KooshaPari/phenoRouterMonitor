//! # Phenotype Crypto
//!
//! Comprehensive cryptographic utilities including hashing (SHA-256, Blake3),
//! symmetric encryption (AES-GCM), key derivation (PBKDF2),
//! HMAC signatures, and secure random generation.
//!
//! # FR Traceability
//! - FR-PHENO-CRYPTO-001: SHA-256 hashing functionality
//! - FR-PHENO-CRYPTO-002: Blake3 hashing functionality
//! - FR-PHENO-CRYPTO-003: HMAC signature computation and verification
//! - FR-PHENO-CRYPTO-004: Secure nonce generation
//! - FR-PHENO-CRYPTO-005: SecureRandom trait for randomness abstraction

pub mod hash;
pub mod key;
pub mod kdf;
pub mod signing;
pub mod random;

pub use hash::{blake3_hash, content_id, sha256_hash, HashAlgorithm, hash_sha256, hash_blake3};
pub use key::{KeyPair, PublicKey, KeyError};
pub use kdf::Pbkdf2Kdf;
pub use signing::{
    compute_hmac, verify_hmac, compute_hmac_hex, verify_hmac_hex,
    Ed25519Signer, Ed25519Verifier, SignatureBundle, Signer, Verifier,
    SigningError, HMAC_SIZE,
};
pub use random::{SecureRandom, DefaultSecureRandom, generate_random_bytes, RandomError};
