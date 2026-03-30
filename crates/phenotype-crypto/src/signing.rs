//! Digital signature utilities — signing and verification using Ed25519.

use crate::key::{KeyError, KeyPair, PublicKey};
use ed25519_dalek::{Signature, Signer as _};
use thiserror::Error;

/// Errors related to signing operations.
#[derive(Debug, Error)]
pub enum SigningError {
    #[error("Signature verification failed")]
    VerificationFailed,

    #[error("Key error: {0}")]
    KeyError(#[from] KeyError),

    #[error("Invalid signature format: {0}")]
    InvalidSignatureFormat(String),

    #[error("Hex decode error: {0}")]
    DecodeError(String),
}

/// Trait for signing data.
pub trait Signer {
    /// Sign the provided data, returning the signature as hex string.
    fn sign(&self, data: &[u8]) -> Result<String, SigningError>;

    /// Sign and return raw signature bytes.
    fn sign_bytes(&self, data: &[u8]) -> Result<[u8; 64], SigningError>;
}

/// Trait for verifying signatures.
pub trait Verifier {
    /// Verify a signature (hex-encoded) against the data.
    fn verify(&self, data: &[u8], signature_hex: &str) -> Result<(), SigningError>;

    /// Verify raw signature bytes against the data.
    fn verify_bytes(&self, data: &[u8], signature: &[u8; 64]) -> Result<(), SigningError>;
}

/// Signer implementation using Ed25519 keypair.
pub struct Ed25519Signer {
    keypair: KeyPair,
}

impl Ed25519Signer {
    /// Create a new signer from a keypair.
    pub fn new(keypair: KeyPair) -> Self {
        Self { keypair }
    }

    /// Generate a new random signer.
    pub fn generate() -> Self {
        Self {
            keypair: KeyPair::generate(),
        }
    }

    /// Create from a seed.
    pub fn from_seed(seed: &[u8]) -> Result<Self, KeyError> {
        Ok(Self {
            keypair: KeyPair::from_seed(seed)?,
        })
    }

    /// Get the public key for verification.
    pub fn public_key(&self) -> PublicKey {
        PublicKey::new(*self.keypair.verifying_key())
    }

    /// Get the keypair.
    pub fn keypair(&self) -> &KeyPair {
        &self.keypair
    }
}

impl Signer for Ed25519Signer {
    fn sign(&self, data: &[u8]) -> Result<String, SigningError> {
        let signature = self.keypair.signing_key().sign(data);
        Ok(hex::encode(signature.to_bytes()))
    }

    fn sign_bytes(&self, data: &[u8]) -> Result<[u8; 64], SigningError> {
        let signature = self.keypair.signing_key().sign(data);
        Ok(signature.to_bytes())
    }
}

/// Verifier implementation using Ed25519 public key.
pub struct Ed25519Verifier {
    public_key: PublicKey,
}

impl Ed25519Verifier {
    /// Create a new verifier from a public key.
    pub fn new(public_key: PublicKey) -> Self {
        Self { public_key }
    }

    /// Create from hex-encoded public key.
    pub fn from_hex(hex: &str) -> Result<Self, KeyError> {
        Ok(Self {
            public_key: PublicKey::from_hex(hex)?,
        })
    }

    /// Get the public key.
    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }
}

impl Verifier for Ed25519Verifier {
    fn verify(&self, data: &[u8], signature_hex: &str) -> Result<(), SigningError> {
        let sig_bytes = hex::decode(signature_hex)
            .map_err(|e| SigningError::DecodeError(e.to_string()))?;

        let sig_array = <[u8; 64]>::try_from(sig_bytes.as_slice())
            .map_err(|_| SigningError::InvalidSignatureFormat("Expected 64 bytes".to_string()))?;

        self.verify_bytes(data, &sig_array)
    }

    fn verify_bytes(&self, data: &[u8], signature: &[u8; 64]) -> Result<(), SigningError> {
        let sig = Signature::from_bytes(signature);
        self.public_key
            .inner()
            .verify_strict(data, &sig)
            .map_err(|_| SigningError::VerificationFailed)
    }
}

/// HMAC signature size in bytes (SHA-256 output).
pub const HMAC_SIZE: usize = 32;

/// Sign and verify using the same keypair (for testing).
pub struct SignatureBundle {
    signer: Ed25519Signer,
    verifier: Ed25519Verifier,
}

impl SignatureBundle {
    /// Create from a keypair.
    pub fn new(keypair: KeyPair) -> Self {
        let signer = Ed25519Signer::new(keypair.clone());
        let verifier = Ed25519Verifier::new(signer.public_key());
        Self { signer, verifier }
    }

    /// Generate new keypair and bundle.
    pub fn generate() -> Self {
        Self::new(KeyPair::generate())
    }

    /// Get the signer.
    pub fn signer(&self) -> &Ed25519Signer {
        &self.signer
    }

    /// Get the verifier.
    pub fn verifier(&self) -> &Ed25519Verifier {
        &self.verifier
    }

    /// Sign data.
    pub fn sign(&self, data: &[u8]) -> Result<String, SigningError> {
        self.signer.sign(data)
    }

    /// Verify signature.
    pub fn verify(&self, data: &[u8], signature_hex: &str) -> Result<(), SigningError> {
        self.verifier.verify(data, signature_hex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_sign_verify() {
        let bundle = SignatureBundle::generate();
        let data = b"hello, world!";

        let signature = bundle.sign(data).expect("Signing failed");
        bundle
            .verify(data, &signature)
            .expect("Verification failed");
    }

    #[test]
    fn test_signature_verification_fails_on_tampered_data() {
        let bundle = SignatureBundle::generate();
        let data = b"original data";
        let tampered = b"tampered data";

        let signature = bundle.sign(data).expect("Signing failed");
        let result = bundle.verify(tampered, &signature);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SigningError::VerificationFailed));
    }

    #[test]
    fn test_signature_verification_fails_on_tampered_signature() {
        let bundle = SignatureBundle::generate();
        let data = b"hello";

        let signature = bundle.sign(data).expect("Signing failed");
        let tampered_sig = format!("{}00", &signature[..signature.len() - 2]);

        let result = bundle.verify(data, &tampered_sig);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_keys_produce_different_signatures() {
        let bundle1 = SignatureBundle::generate();
        let bundle2 = SignatureBundle::generate();
        let data = b"same data";

        let sig1 = bundle1.sign(data).expect("Signing failed");
        let sig2 = bundle2.sign(data).expect("Signing failed");

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_same_key_produces_same_signature() {
        let keypair = KeyPair::generate();
        let bundle1 = SignatureBundle::new(keypair.clone());
        let bundle2 = SignatureBundle::new(keypair.clone());
        let data = b"test data";

        let sig1 = bundle1.sign(data).expect("Signing failed");
        let sig2 = bundle2.sign(data).expect("Signing failed");

        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_sign_bytes_method() {
        let bundle = SignatureBundle::generate();
        let data = b"test";

        let sig_bytes = bundle.signer().sign_bytes(data).expect("Failed");
        assert_eq!(sig_bytes.len(), 64);

        let sig_hex = hex::encode(sig_bytes);
        bundle
            .verify(data, &sig_hex)
            .expect("Verification failed");
    }

    #[test]
    fn test_verifier_from_hex_key() {
        let signer = Ed25519Signer::generate();
        let pub_key_hex = signer.public_key().to_hex();

        let verifier =
            Ed25519Verifier::from_hex(&pub_key_hex).expect("Failed to create verifier");
        let data = b"test";
        let signature = signer.sign(data).expect("Signing failed");

        verifier
            .verify(data, &signature)
            .expect("Verification failed");
    }

    #[test]
    fn test_empty_data_signature() {
        let bundle = SignatureBundle::generate();
        let data = b"";

        let signature = bundle.sign(data).expect("Signing failed");
        bundle
            .verify(data, &signature)
            .expect("Verification failed");
    }

    #[test]
    fn test_large_data_signature() {
        let bundle = SignatureBundle::generate();
        let data = vec![42u8; 10_000];

        let signature = bundle.sign(&data).expect("Signing failed");
        bundle
            .verify(&data, &signature)
            .expect("Verification failed");
    }
}
