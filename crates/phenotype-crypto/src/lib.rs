//! # Phenotype Crypto
//!
//! Cryptographic utilities: hashing (SHA-256, Blake3), hex encoding,
//! and content-addressable identifiers.

pub mod hash;

pub use hash::{blake3_hash, content_id, sha256_hash, HashAlgorithm};
