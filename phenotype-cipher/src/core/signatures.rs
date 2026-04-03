//! Digital signature implementations

use ed25519_dalek::{Signer, SigningKey, VerifyingKey, Signature, Verifier};
use rand::rngs::OsRng;

/// Keypair for Ed25519 signatures
#[derive(Debug, Clone)]
pub struct Keypair {
    pub secret_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

/// Generate a new Ed25519 keypair
pub fn generate_keypair() -> Keypair {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    
    Keypair {
        secret_key: signing_key.to_bytes().to_vec(),
        public_key: verifying_key.to_bytes().to_vec(),
    }
}

/// Sign a message using Ed25519
pub fn sign(message: &[u8], secret_key: &[u8]) -> Result<Vec<u8>, SignatureError> {
    let key_bytes: [u8; 32] = secret_key.try_into()
        .map_err(|_| SignatureError::InvalidKey)?;
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let signature = signing_key.sign(message);
    Ok(signature.to_bytes().to_vec())
}

/// Verify an Ed25519 signature
pub fn verify(message: &[u8], signature: &[u8], public_key: &[u8]) -> Result<(), SignatureError> {
    let key_bytes: [u8; 32] = public_key.try_into()
        .map_err(|_| SignatureError::InvalidKey)?;
    let verifying_key = VerifyingKey::from_bytes(&key_bytes)
        .map_err(|_| SignatureError::InvalidKey)?;
    
    let sig_bytes: [u8; 64] = signature.try_into()
        .map_err(|_| SignatureError::InvalidSignature)?;
    let sig = Signature::from_bytes(&sig_bytes);
    
    verifying_key.verify(message, &sig)
        .map_err(|_| SignatureError::VerificationFailed)
}

/// Signature errors
#[derive(Debug, Clone, PartialEq)]
pub enum SignatureError {
    InvalidKey,
    InvalidSignature,
    VerificationFailed,
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignatureError::InvalidKey => write!(f, "Invalid key"),
            SignatureError::InvalidSignature => write!(f, "Invalid signature"),
            SignatureError::VerificationFailed => write!(f, "Signature verification failed"),
        }
    }
}

impl std::error::Error for SignatureError {}
