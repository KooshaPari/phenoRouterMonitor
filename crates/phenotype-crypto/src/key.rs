//! Key management utilities — Ed25519 key pair generation, export, and import.

use ed25519_dalek::{SigningKey, VerifyingKey, SECRET_KEY_LENGTH};
use rand::rngs::OsRng;
use rand::RngCore;
use thiserror::Error;
use zeroize::Zeroize;

/// Errors related to key operations.
#[derive(Debug, Error)]
pub enum KeyError {
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    #[error("Key decode error: {0}")]
    DecodeError(String),

    #[error("Invalid seed length (expected 32 bytes, got {0})")]
    InvalidSeedLength(usize),
}

/// Ed25519 keypair wrapper with secure key handling.
#[derive(Clone, Debug)]
pub struct KeyPair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl KeyPair {
    /// Generate a new random Ed25519 keypair.
    pub fn generate() -> Self {
        let mut seed = [0u8; SECRET_KEY_LENGTH];
        OsRng.fill_bytes(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = VerifyingKey::from(&signing_key);
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Create a keypair from a 32-byte seed.
    ///
    /// # Errors
    /// Returns `KeyError::InvalidSeedLength` if seed is not exactly 32 bytes.
    pub fn from_seed(seed: &[u8]) -> Result<Self, KeyError> {
        if seed.len() != 32 {
            return Err(KeyError::InvalidSeedLength(seed.len()));
        }

        let mut seed_bytes = [0u8; 32];
        seed_bytes.copy_from_slice(seed);

        let signing_key = SigningKey::from_bytes(&seed_bytes);
        let verifying_key = VerifyingKey::from(&signing_key);

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Export the signing key as bytes (32 bytes for Ed25519).
    pub fn signing_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Export the verifying key as bytes (32 bytes for Ed25519).
    pub fn verifying_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Export the signing key as a hex string.
    pub fn signing_key_hex(&self) -> String {
        hex::encode(self.signing_key_bytes())
    }

    /// Export the verifying key as a hex string.
    pub fn verifying_key_hex(&self) -> String {
        hex::encode(self.verifying_key_bytes())
    }

    /// Import a keypair from hex-encoded signing key.
    ///
    /// # Errors
    /// Returns `KeyError::DecodeError` if hex decoding fails or key format is invalid.
    pub fn from_signing_key_hex(hex: &str) -> Result<Self, KeyError> {
        let bytes = hex::decode(hex).map_err(|e| KeyError::DecodeError(e.to_string()))?;
        Self::from_seed(&bytes)
    }

    /// Get reference to the signing key for direct operations.
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    /// Get reference to the verifying key for direct operations.
    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }
}

impl Drop for KeyPair {
    fn drop(&mut self) {
        // Zero out sensitive data on drop
        let mut seed = self.signing_key.to_bytes();
        seed.zeroize();
    }
}

/// Verifying key wrapper for signature verification.
#[derive(Clone, Copy)]
pub struct PublicKey(VerifyingKey);

impl PublicKey {
    /// Create a public key from a verifying key.
    pub fn new(verifying_key: VerifyingKey) -> Self {
        Self(verifying_key)
    }

    /// Create a public key from 32 bytes.
    ///
    /// # Errors
    /// Returns `KeyError::InvalidKeyFormat` if bytes don't form a valid Ed25519 key.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KeyError> {
        let key_bytes = <[u8; 32]>::try_from(bytes)
            .map_err(|_| KeyError::InvalidKeyFormat("Expected 32 bytes".to_string()))?;
        let verifying_key = VerifyingKey::from_bytes(&key_bytes)
            .map_err(|_| KeyError::InvalidKeyFormat("Invalid Ed25519 key bytes".to_string()))?;
        Ok(Self(verifying_key))
    }

    /// Create a public key from hex string.
    ///
    /// # Errors
    /// Returns `KeyError::DecodeError` if hex decoding fails.
    pub fn from_hex(hex: &str) -> Result<Self, KeyError> {
        let bytes = hex::decode(hex).map_err(|e| KeyError::DecodeError(e.to_string()))?;
        Self::from_bytes(&bytes)
    }

    /// Export as bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Export as hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }

    /// Get reference to the inner verifying key.
    pub fn inner(&self) -> &VerifyingKey {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generate() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();

        assert_ne!(kp1.signing_key_hex(), kp2.signing_key_hex());
        assert_ne!(kp1.verifying_key_hex(), kp2.verifying_key_hex());
        assert_eq!(kp1.signing_key_hex().len(), 64);
        assert_eq!(kp1.verifying_key_hex().len(), 64);
    }

    #[test]
    fn test_keypair_from_seed() {
        let seed = [1u8; 32];
        let kp1 = KeyPair::from_seed(&seed).expect("Valid seed");
        let kp2 = KeyPair::from_seed(&seed).expect("Valid seed");

        assert_eq!(kp1.signing_key_hex(), kp2.signing_key_hex());
        assert_eq!(kp1.verifying_key_hex(), kp2.verifying_key_hex());
    }

    #[test]
    fn test_keypair_from_seed_invalid_length() {
        let short_seed = [1u8; 16];
        let result = KeyPair::from_seed(&short_seed);
        assert!(result.is_err());
    }

    #[test]
    fn test_keypair_roundtrip_hex() {
        let kp1 = KeyPair::generate();
        let hex = kp1.signing_key_hex();
        let kp2 = KeyPair::from_signing_key_hex(&hex).expect("Valid hex");

        assert_eq!(kp1.signing_key_hex(), kp2.signing_key_hex());
        assert_eq!(kp1.verifying_key_hex(), kp2.verifying_key_hex());
    }

    #[test]
    fn test_public_key_from_bytes() {
        let kp = KeyPair::generate();
        let bytes = kp.verifying_key_bytes();
        let pub_key = PublicKey::from_bytes(&bytes).expect("Valid bytes");

        assert_eq!(pub_key.to_bytes(), bytes);
    }

    #[test]
    fn test_public_key_from_hex() {
        let kp = KeyPair::generate();
        let hex = kp.verifying_key_hex();
        let pub_key = PublicKey::from_hex(&hex).expect("Valid hex");

        assert_eq!(pub_key.to_hex(), hex);
    }

    #[test]
    fn test_public_key_invalid_format() {
        let result = PublicKey::from_hex("deadbeef");
        assert!(result.is_err());
    }
}
