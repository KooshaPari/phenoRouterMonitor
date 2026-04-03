//! Encryption implementations using AES-GCM and ChaCha20-Poly1305 via the aead crate.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce as AesNonce,
};
use chacha20poly1305::{
    aead::Aead as _,
    ChaCha20Poly1305, Nonce as ChaChaNonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Cipher suite selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CipherSuite {
    Aes256Gcm,
    ChaCha20Poly1305,
}

/// Encryption errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum EncryptionError {
    #[error("invalid key length: expected {expected} bytes, got {got}")]
    InvalidKeyLength { expected: usize, got: usize },
    #[error("encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("wrong cipher suite")]
    WrongSuite,
}

/// Output of an encryption operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ciphertext {
    /// Nonce/counter (12 bytes for both AES-GCM and ChaCha20-Poly1305)
    pub nonce: Vec<u8>,
    /// Encrypted data (includes auth tag)
    pub data: Vec<u8>,
    /// Selected cipher suite
    pub suite: CipherSuite,
}

/// AES-256-GCM implementation
#[derive(Clone)]
pub struct AesGcmCipher {
    key: Aes256Gcm,
}

impl AesGcmCipher {
    /// Create a new AES-256-GCM cipher from 32-byte key
    pub fn new(key: &[u8]) -> Result<Self, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength {
                expected: 32,
                got: key.len(),
            });
        }
        let key_arr: [u8; 32] = key.try_into().map_err(|_| EncryptionError::InvalidKeyLength {
            expected: 32,
            got: key.len(),
        })?;
        let cipher = Aes256Gcm::new_from_slice(&key_arr)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        Ok(Self { key: cipher })
    }

    /// Generate a random 32-byte key
    pub fn generate_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Encrypt plaintext using AES-256-GCM
    pub fn encrypt(&self, pt: &[u8]) -> Result<Ciphertext, EncryptionError> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = AesNonce::from_slice(&nonce_bytes);

        let ct = self.key
            .encrypt(nonce, pt)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        Ok(Ciphertext {
            nonce: nonce_bytes.to_vec(),
            data: ct,
            suite: CipherSuite::Aes256Gcm,
        })
    }

    /// Decrypt ciphertext using AES-256-GCM
    pub fn decrypt(&self, ct: &Ciphertext) -> Result<Vec<u8>, EncryptionError> {
        if ct.suite != CipherSuite::Aes256Gcm {
            return Err(EncryptionError::WrongSuite);
        }
        let nonce = AesNonce::from_slice(&ct.nonce);
        self.key
            .decrypt(nonce, ct.data.as_ref())
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))
    }
}

/// ChaCha20-Poly1305 implementation
#[derive(Clone)]
pub struct ChaChaCipher {
    key: ChaCha20Poly1305,
}

impl ChaChaCipher {
    /// Create a new ChaCha20-Poly1305 cipher from 32-byte key
    pub fn new(key: &[u8]) -> Result<Self, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength {
                expected: 32,
                got: key.len(),
            });
        }
        let key_arr: [u8; 32] = key.try_into().map_err(|_| EncryptionError::InvalidKeyLength {
            expected: 32,
            got: key.len(),
        })?;
        let cipher = ChaCha20Poly1305::new_from_slice(&key_arr)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        Ok(Self { key: cipher })
    }

    /// Generate a random 32-byte key
    pub fn generate_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Encrypt plaintext using ChaCha20-Poly1305
    pub fn encrypt(&self, pt: &[u8]) -> Result<Ciphertext, EncryptionError> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = ChaChaNonce::from_slice(&nonce_bytes);

        let ct = self.key
            .encrypt(nonce, pt)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        Ok(Ciphertext {
            nonce: nonce_bytes.to_vec(),
            data: ct,
            suite: CipherSuite::ChaCha20Poly1305,
        })
    }

    /// Decrypt ciphertext using ChaCha20-Poly1305
    pub fn decrypt(&self, ct: &Ciphertext) -> Result<Vec<u8>, EncryptionError> {
        if ct.suite != CipherSuite::ChaCha20Poly1305 {
            return Err(EncryptionError::WrongSuite);
        }
        let nonce = ChaChaNonce::from_slice(&ct.nonce);
        self.key
            .decrypt(nonce, ct.data.as_ref())
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))
    }
}

/// Encrypt with selected cipher suite (convenience function)
pub fn encrypt_with_suite(
    suite: CipherSuite,
    key: &[u8],
    plaintext: &[u8],
) -> Result<Ciphertext, EncryptionError> {
    match suite {
        CipherSuite::Aes256Gcm => {
            let cipher = AesGcmCipher::new(key)?;
            cipher.encrypt(plaintext)
        }
        CipherSuite::ChaCha20Poly1305 => {
            let cipher = ChaChaCipher::new(key)?;
            cipher.encrypt(plaintext)
        }
    }
}

/// Decrypt with selected cipher suite (convenience function)
pub fn decrypt_with_suite(
    suite: CipherSuite,
    key: &[u8],
    ciphertext: &Ciphertext,
) -> Result<Vec<u8>, EncryptionError> {
    match suite {
        CipherSuite::Aes256Gcm => {
            let cipher = AesGcmCipher::new(key)?;
            cipher.decrypt(ciphertext)
        }
        CipherSuite::ChaCha20Poly1305 => {
            let cipher = ChaChaCipher::new(key)?;
            cipher.decrypt(ciphertext)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_roundtrip() {
        let key = AesGcmCipher::generate_key();
        let cipher = AesGcmCipher::new(&key).unwrap();
        let pt = b"hello world";
        let ct = cipher.encrypt(pt).unwrap();
        assert_eq!(ct.suite, CipherSuite::Aes256Gcm);
        let decrypted = cipher.decrypt(&ct).unwrap();
        assert_eq!(decrypted, pt);
    }

    #[test]
    fn test_chacha_roundtrip() {
        let key = ChaChaCipher::generate_key();
        let cipher = ChaChaCipher::new(&key).unwrap();
        let pt = b"hello world";
        let ct = cipher.encrypt(pt).unwrap();
        assert_eq!(ct.suite, CipherSuite::ChaCha20Poly1305);
        let decrypted = cipher.decrypt(&ct).unwrap();
        assert_eq!(decrypted, pt);
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = vec![0u8; 16];
        assert!(AesGcmCipher::new(&short_key).is_err());
        assert!(ChaChaCipher::new(&short_key).is_err());
    }

    #[test]
    fn test_different_nonces() {
        let key = AesGcmCipher::generate_key();
        let cipher = AesGcmCipher::new(&key).unwrap();
        let ct1 = cipher.encrypt(b"hello").unwrap();
        let ct2 = cipher.encrypt(b"hello").unwrap();
        assert_ne!(ct1.nonce, ct2.nonce);
    }

    #[test]
    fn test_cipher_suite_serialization() {
        let suite = CipherSuite::Aes256Gcm;
        let json = serde_json::to_string(&suite).unwrap();
        let deserialized: CipherSuite = serde_json::from_str(&json).unwrap();
        assert_eq!(suite, deserialized);
    }

    #[test]
    fn test_encrypt_with_suite_aes() {
        let key = AesGcmCipher::generate_key();
        let pt = b"Test data";
        let ct = encrypt_with_suite(CipherSuite::Aes256Gcm, &key, pt).unwrap();
        assert_eq!(ct.suite, CipherSuite::Aes256Gcm);
        let decrypted = decrypt_with_suite(CipherSuite::Aes256Gcm, &key, &ct).unwrap();
        assert_eq!(decrypted, pt);
    }

    #[test]
    fn test_encrypt_with_suite_chacha() {
        let key = ChaChaCipher::generate_key();
        let pt = b"Test data";
        let ct = encrypt_with_suite(CipherSuite::ChaCha20Poly1305, &key, pt).unwrap();
        assert_eq!(ct.suite, CipherSuite::ChaCha20Poly1305);
        let decrypted = decrypt_with_suite(CipherSuite::ChaCha20Poly1305, &key, &ct).unwrap();
        assert_eq!(decrypted, pt);
    }
}
