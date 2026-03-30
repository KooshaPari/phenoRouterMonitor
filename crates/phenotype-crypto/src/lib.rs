//! Phenotype crypto utilities: BLAKE3 and SHA-256 hashing.
//!
//! Provides [`ContentHash`] (BLAKE3) for internal content addressing and
//! [`Sha256Hash`] for interoperability with external systems (OCI digests, etc.).

use std::fmt;
use std::path::Path;
use std::str::FromStr;
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors produced by crypto operations.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    /// An I/O error occurred (e.g. reading a file for hashing).
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    /// The provided string is not a valid hex-encoded hash.
    #[error("invalid hash: {0}")]
    InvalidHash(String),

    /// A hex encoding/decoding error.
    #[error("encoding error: {0}")]
    EncodingError(#[from] hex::FromHexError),
}

// ---------------------------------------------------------------------------
// ContentHash (BLAKE3, 32 bytes)
// ---------------------------------------------------------------------------

/// A BLAKE3 content hash (32 bytes).
///
/// Displays and serializes as a lowercase hex string.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    /// Create a [`ContentHash`] from raw bytes.
    #[inline]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self { Self(bytes) }

    /// Return the underlying byte array.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 32] { &self.0 }

    /// Return the hash as a lowercase hex string.
    pub fn to_hex(&self) -> String { hex::encode(self.0) }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl fmt::Debug for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentHash({})", self.to_hex())
    }
}

impl FromStr for ContentHash {
    type Err = CryptoError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        let arr: [u8; 32] = bytes.try_into().map_err(|v: Vec<u8>| {
            CryptoError::InvalidHash(format!("expected 32 bytes, got {}", v.len()))
        })?;
        Ok(Self(arr))
    }
}

impl serde::Serialize for ContentHash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> serde::Deserialize<'de> for ContentHash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Sha256Hash (SHA-256, 32 bytes)
// ---------------------------------------------------------------------------

/// A SHA-256 hash (32 bytes) for interoperability with external systems.
///
/// Displays and serializes as a lowercase hex string.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sha256Hash([u8; 32]);

impl Sha256Hash {
    /// Create a [`Sha256Hash`] from raw bytes.
    #[inline]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self { Self(bytes) }

    /// Return the underlying byte array.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 32] { &self.0 }

    /// Return the hash as a lowercase hex string.
    pub fn to_hex(&self) -> String { hex::encode(self.0) }
}

impl fmt::Display for Sha256Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl fmt::Debug for Sha256Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sha256Hash({})", self.to_hex())
    }
}

impl FromStr for Sha256Hash {
    type Err = CryptoError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        let arr: [u8; 32] = bytes.try_into().map_err(|v: Vec<u8>| {
            CryptoError::InvalidHash(format!("expected 32 bytes, got {}", v.len()))
        })?;
        Ok(Self(arr))
    }
}

impl serde::Serialize for Sha256Hash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> serde::Deserialize<'de> for Sha256Hash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Hashing functions
// ---------------------------------------------------------------------------

/// Hash arbitrary bytes with BLAKE3 and return a [`ContentHash`].
#[inline]
pub fn hash_bytes(data: &[u8]) -> ContentHash {
    ContentHash(*blake3::hash(data).as_bytes())
}

/// Hash a string slice with BLAKE3.
#[inline]
pub fn hash_str(s: &str) -> ContentHash {
    hash_bytes(s.as_bytes())
}

/// Asynchronously read a file and return its BLAKE3 [`ContentHash`].
pub async fn hash_file(path: &Path) -> Result<ContentHash, CryptoError> {
    let data = tokio::fs::read(path).await?;
    Ok(hash_bytes(&data))
}

/// Verify that `data` hashes to the `expected` BLAKE3 [`ContentHash`].
#[inline]
pub fn verify_hash(data: &[u8], expected: &ContentHash) -> bool {
    hash_bytes(data) == *expected
}

/// Compute the SHA-256 digest of `data`.
#[inline]
pub fn sha256(data: &[u8]) -> Sha256Hash {
    Sha256Hash(Sha256::digest(data).into())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    #[test]
    fn blake3_roundtrip() {
        let h = hash_str("hello world");
        let hex_str = h.to_hex();
        let parsed: ContentHash = hex_str.parse().unwrap();
        assert_eq!(h, parsed);
    }

    #[test]
    fn blake3_deterministic() {
        assert_eq!(hash_bytes(b"abc"), hash_bytes(b"abc"));
        assert_ne!(hash_bytes(b"abc"), hash_bytes(b"def"));
    }

    #[test]
    fn verify_hash_works() {
        let data = b"test data";
        let h = hash_bytes(data);
        assert!(verify_hash(data, &h));
        assert!(!verify_hash(b"wrong", &h));
    }

    #[test]
    fn sha256_known_vector() {
        let h = sha256(b"");
        assert_eq!(
            h.to_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_roundtrip() {
        let h = sha256(b"hello");
        let parsed: Sha256Hash = h.to_hex().parse().unwrap();
        assert_eq!(h, parsed);
    }

    #[test]
    fn serde_content_hash() {
        let h = hash_str("serde test");
        let json = serde_json::to_string(&h).unwrap();
        let back: ContentHash = serde_json::from_str(&json).unwrap();
        assert_eq!(h, back);
    }

    #[test]
    fn serde_sha256_hash() {
        let h = sha256(b"serde test");
        let json = serde_json::to_string(&h).unwrap();
        let back: Sha256Hash = serde_json::from_str(&json).unwrap();
        assert_eq!(h, back);
    }

    #[test]
    fn invalid_hex_errors() {
        assert!("not_hex_zz".parse::<ContentHash>().is_err());
    }

    #[test]
    fn wrong_length_errors() {
        let short = hex::encode([0u8; 16]);
        assert!(short.parse::<ContentHash>().is_err());
        assert!("aabb".parse::<ContentHash>().is_err());
    }

    #[tokio::test]
    async fn hash_file_works() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"file content").unwrap();
        tmp.flush().unwrap();
        let h = hash_file(tmp.path()).await.unwrap();
        assert_eq!(h, hash_bytes(b"file content"));
    }

    #[tokio::test]
    async fn hash_file_not_found() {
        let result = hash_file(Path::new("/nonexistent/file")).await;
        assert!(matches!(result.unwrap_err(), CryptoError::IoError(_)));
    }

    #[test]
    fn display_and_debug() {
        let h = hash_str("display");
        let display = format!("{h}");
        let debug = format!("{h:?}");
        assert_eq!(display, h.to_hex());
        assert!(debug.starts_with("ContentHash("));
    }
}
