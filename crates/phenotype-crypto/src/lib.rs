//! # Phenotype Crypto
//!
//! Comprehensive cryptographic utilities for Phenotype:
//!
//! - **Hashing**: SHA-256 and Blake3 hashing with content-addressable identifiers
//! - **Key Management**: Ed25519 keypair generation and secure key handling
//! - **Signing & Verification**: Sign and verify data using Ed25519 digital signatures
//! - **Key Derivation**: PBKDF2-SHA256 key derivation from passwords

pub mod hash;
pub mod key;
pub mod kdf;
pub mod signing;

pub use hash::{blake3_hash, content_id, sha256_hash, HashAlgorithm};
pub use key::{KeyError, KeyPair, PublicKey};
pub use kdf::{generate_salt, generate_salt_hex, Pbkdf2Kdf, KdfError, DEFAULT_ITERATIONS};
pub use signing::{Ed25519Signer, Ed25519Verifier, SignatureBundle, Signer, Verifier, SigningError};
