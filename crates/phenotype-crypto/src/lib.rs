//\! Phenotype crypto utilities: BLAKE3 and SHA-256 hashing.

use std::fmt;
use std::path::Path;
use std::str::FromStr;
use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("invalid hash: {0}")]
    InvalidHash(String),
    #[error("encoding error: {0}")]
    EncodingError(#[from] hex::FromHexError),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash([u8; 32]);
impl ContentHash {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self { Self(bytes) }
    pub const fn as_bytes(&self) -> &[u8; 32] { &self.0 }
    pub fn to_hex(&self) -> String { hex::encode(self.0) }
}
impl fmt::Display for ContentHash { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.to_hex()) } }
impl fmt::Debug for ContentHash { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write\!(f, "ContentHash({})", self.to_hex()) } }
impl FromStr for ContentHash {
    type Err = CryptoError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        let arr: [u8; 32] = bytes.try_into().map_err(|v: Vec<u8>| CryptoError::InvalidHash(format\!("expected 32 bytes, got {}", v.len())))?;
        Ok(Self(arr))
    }
}
impl serde::Serialize for ContentHash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { serializer.serialize_str(&self.to_hex()) }
}
impl<'de> serde::Deserialize<'de> for ContentHash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sha256Hash([u8; 32]);
impl Sha256Hash {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self { Self(bytes) }
    pub const fn as_bytes(&self) -> &[u8; 32] { &self.0 }
    pub fn to_hex(&self) -> String { hex::encode(self.0) }
}
impl fmt::Display for Sha256Hash { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.to_hex()) } }
impl fmt::Debug for Sha256Hash { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write\!(f, "Sha256Hash({})", self.to_hex()) } }
impl FromStr for Sha256Hash {
    type Err = CryptoError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)?;
        let arr: [u8; 32] = bytes.try_into().map_err(|v: Vec<u8>| CryptoError::InvalidHash(format\!("expected 32 bytes, got {}", v.len())))?;
        Ok(Self(arr))
    }
}
impl serde::Serialize for Sha256Hash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { serializer.serialize_str(&self.to_hex()) }
}
impl<'de> serde::Deserialize<'de> for Sha256Hash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

pub fn hash_bytes(data: &[u8]) -> ContentHash { ContentHash(*blake3::hash(data).as_bytes()) }
pub fn hash_str(s: &str) -> ContentHash { hash_bytes(s.as_bytes()) }
pub async fn hash_file(path: &Path) -> Result<ContentHash, CryptoError> { let data = tokio::fs::read(path).await?; Ok(hash_bytes(&data)) }
pub fn verify_hash(data: &[u8], expected: &ContentHash) -> bool { hash_bytes(data) == *expected }
pub fn sha256(data: &[u8]) -> Sha256Hash { Sha256Hash(Sha256::digest(data).into()) }

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;
    #[test] fn blake3_roundtrip() { let h = hash_str("hello"); let p: ContentHash = h.to_hex().parse().unwrap(); assert_eq\!(h, p); }
    #[test] fn blake3_deterministic() { assert_eq\!(hash_bytes(b"abc"), hash_bytes(b"abc")); assert_ne\!(hash_bytes(b"abc"), hash_bytes(b"def")); }
    #[test] fn verify_works() { let h = hash_bytes(b"test"); assert\!(verify_hash(b"test", &h)); assert\!(\!verify_hash(b"wrong", &h)); }
    #[test] fn sha256_empty() { assert_eq\!(sha256(b"").to_hex(), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"); }
    #[test] fn sha256_roundtrip() { let h = sha256(b"hello"); let p: Sha256Hash = h.to_hex().parse().unwrap(); assert_eq\!(h, p); }
    #[test] fn serde_content() { let h = hash_str("s"); let j = serde_json::to_string(&h).unwrap(); let b: ContentHash = serde_json::from_str(&j).unwrap(); assert_eq\!(h, b); }
    #[test] fn serde_sha() { let h = sha256(b"s"); let j = serde_json::to_string(&h).unwrap(); let b: Sha256Hash = serde_json::from_str(&j).unwrap(); assert_eq\!(h, b); }
    #[test] fn invalid_hex() { assert\!("zz".parse::<ContentHash>().is_err()); }
    #[test] fn wrong_len() { assert\!(hex::encode([0u8; 16]).parse::<ContentHash>().is_err()); }
    #[tokio::test] async fn file_hash() { let mut t = tempfile::NamedTempFile::new().unwrap(); t.write_all(b"fc").unwrap(); t.flush().unwrap(); assert_eq\!(hash_file(t.path()).await.unwrap(), hash_bytes(b"fc")); }
    #[tokio::test] async fn file_not_found() { assert\!(matches\!(hash_file(Path::new("/no")).await.unwrap_err(), CryptoError::IoError(_))); }
    #[test] fn display_debug() { let h = hash_str("d"); assert_eq\!(format\!("{h}"), h.to_hex()); assert\!(format\!("{h:?}").starts_with("ContentHash(")); }
}
