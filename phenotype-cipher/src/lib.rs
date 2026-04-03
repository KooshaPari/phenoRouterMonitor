//! Cryptographic primitives for the Phenotype ecosystem.
//!
//! Provides authenticated encryption (AES-256-GCM, ChaCha20-Poly1305),
//! digital signatures (Ed25519), and hashing (SHA-256).

pub mod core;

pub use core::encryption::{
    encrypt_with_suite, decrypt_with_suite, AesGcmCipher, ChaChaCipher,
    CipherSuite, Ciphertext, EncryptionError
};

use thiserror::Error;

/// Unified error type for all cipher operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CipherError {
    #[error("invalid key: {0}")]
    InvalidKey(String),
    #[error("encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("decryption failed: {0}")]
    DecryptionFailed(String),
}

/// Type alias for cipher results
pub type CipherResult<T> = Result<T, CipherError>;

impl From<EncryptionError> for CipherError {
    fn from(e: EncryptionError) -> Self {
        match e {
            EncryptionError::InvalidKeyLength { expected, got } => {
                CipherError::InvalidKey(format!("expected {} bytes, got {}", expected, got))
            }
            EncryptionError::EncryptionFailed(s) => CipherError::EncryptionFailed(s),
            EncryptionError::DecryptionFailed(s) => CipherError::DecryptionFailed(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_encryption() {
        let key = AesGcmCipher::generate_key();
        let cipher = AesGcmCipher::new(&key).unwrap();
        let pt = b"hello world";
        let ct = cipher.encrypt(pt).unwrap();
        assert_eq!(ct.suite, CipherSuite::Aes256Gcm);
        let decrypted = cipher.decrypt(&ct).unwrap();
        assert_eq!(decrypted, pt);
    }

    #[test]
    fn test_chacha_encryption() {
        let key = ChaChaCipher::generate_key();
        let cipher = ChaChaCipher::new(&key).unwrap();
        let pt = b"hello world";
        let ct = cipher.encrypt(pt).unwrap();
        assert_eq!(ct.suite, CipherSuite::ChaCha20Poly1305);
        let decrypted = cipher.decrypt(&ct).unwrap();
        assert_eq!(decrypted, pt);
    }

    #[test]
    fn test_encrypt_with_suite() {
        let key = AesGcmCipher::generate_key();
        let ct = encrypt_with_suite(CipherSuite::Aes256Gcm, &key, b"test").unwrap();
        assert_eq!(ct.suite, CipherSuite::Aes256Gcm);
        let decrypted = decrypt_with_suite(CipherSuite::Aes256Gcm, &key, &ct).unwrap();
        assert_eq!(decrypted, b"test");
    }
}
