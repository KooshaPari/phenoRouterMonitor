//! Encryption implementations using AES-GCM and ChaCha20-Poly1305 via the aead crate.

use aes_gcm::{
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce as AesNonce,
};
use chacha20poly1305::{
    aead::Aead as ChaChaAead,
    ChaCha20Poly1305, Nonce as ChaChaNonce,
};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Cipher suite selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CipherSuite {
    Aes256Gcm,
    ChaCha20Poly1305,
}

/// Ciphertext container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ciphertext {
    pub data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub suite: CipherSuite,
}

/// Encryption error
#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("invalid key length: expected {expected}, got {got}")]
    InvalidKeyLength { expected: usize, got: usize },
    #[error("encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("decryption failed: {0}")]
    DecryptionFailed(String),
}

/// AES-256-GCM cipher
pub struct AesGcmCipher {
    key: Aes256Gcm,
}

impl AesGcmCipher {
    pub fn new(key: &[u8]) -> Result<Self, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength {
                expected: 32,
                got: key.len(),
            });
        }
        let key_arr: [u8; 32] = key.try_into().unwrap();
        let key = Aes256Gcm::new_from_slice(&key_arr)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        Ok(Self { key })
    }

    pub fn generate_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    pub fn encrypt(&self, pt: &[u8]) -> Result<Ciphertext, EncryptionError> {
        let mut nonce_bytes = vec![0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = AesNonce::from_slice(&nonce_bytes);
        let ct = self.key.encrypt(nonce, pt)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        Ok(Ciphertext {
            data: ct,
            nonce: nonce_bytes,
            suite: CipherSuite::Aes256Gcm,
        })
    }

    pub fn decrypt(&self, ct: &Ciphertext) -> Result<Vec<u8>, EncryptionError> {
        if ct.suite != CipherSuite::Aes256Gcm {
            return Err(EncryptionError::DecryptionFailed("wrong suite".to_string()));
        }
        let nonce = AesNonce::from_slice(&ct.nonce);
        self.key.decrypt(nonce, ct.data.as_ref())
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))
    }
}

/// ChaCha20-Poly1305 cipher
pub struct ChaChaCipher {
    key: ChaChaPoly1305,
}

impl ChaChaCipher {
    pub fn new(key: &[u8]) -> Result<Self, EncryptionError> {
        if key.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength {
                expected: 32,
                got: key.len(),
            });
        }
        let key_arr: [u8; 32] = key.try_into().unwrap();
        let key = ChaChaPoly1305::new_from_slice(&key_arr)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        Ok(Self { key })
    }

    pub fn generate_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    pub fn encrypt(&self, pt: &[u8]) -> Result<Ciphertext, EncryptionError> {
        let mut nonce_bytes = vec![0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = ChaChaNonce::from_slice(&nonce_bytes);
        let ct = self.key.encrypt(nonce, pt)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        Ok(Ciphertext {
            data: ct,
            nonce: nonce_bytes,
            suite: CipherSuite::ChaCha20Poly1305,
        })
    }

    pub fn decrypt(&self, ct: &Ciphertext) -> Result<Vec<u8>, EncryptionError> {
        if ct.suite != CipherSuite::ChaCha20Poly1305 {
            return Err(EncryptionError::DecryptionFailed("wrong suite".to_string()));
        }
        let nonce = ChaChaNonce::from_slice(&ct.nonce);
        self.key.decrypt(nonce, ct.data.as_ref())
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))
    }
}

/// Encrypt with selected cipher suite
pub fn encrypt_with_suite(
    suite: CipherSuite,
    key: &[u8],
    pt: &[u8],
) -> Result<Ciphertext, EncryptionError> {
    match suite {
        CipherSuite::Aes256Gcm => {
            let cipher = AesGcmCipher::new(key)?;
            cipher.encrypt(pt)
        }
        CipherSuite::ChaCha20Poly1305 => {
            let cipher = ChaChaCipher::new(key)?;
            cipher.encrypt(pt)
        }
    }
}

/// Decrypt with selected cipher suite
pub fn decrypt_with_suite(
    suite: CipherSuite,
    key: &[u8],
    ct: &Ciphertext,
) -> Result<Vec<u8>, EncryptionError> {
    match suite {
        CipherSuite::Aes256Gcm => {
            let cipher = AesGcmCipher::new(key)?;
            cipher.decrypt(ct)
        }
        CipherSuite::ChaCha20Poly1305 => {
            let cipher = ChaChaCipher::new(key)?;
            cipher.decrypt(ct)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_roundtrip() {
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
    fn test_encrypt_suite_aes() {
        let key = AesGcmCipher::generate_key();
        let pt = b"Test data";
        let ct = encrypt_with_suite(CipherSuite::Aes256Gcm, &key, pt).unwrap();
        assert_eq!(ct.suite, CipherSuite::Aes256Gcm);
        let decrypted = decrypt_with_suite(CipherSuite::Aes256Gcm, &key, &ct).unwrap();
        assert_eq!(decrypted, pt);
    }

    #[test]
    fn test_encrypt_suite_chacha() {
        let key = ChaChaCipher::generate_key();
        let pt = b"Test data";
        let ct = encrypt_with_suite(CipherSuite::ChaCha20Poly1305, &key, pt).unwrap();
        assert_eq!(ct.suite, CipherSuite::ChaCha20Poly1305);
        let decrypted = decrypt_with_suite(CipherSuite::ChaCha20Poly1305, &key, &ct).unwrap();
        assert_eq!(decrypted, pt);
    }

    #[test]
    fn test_cipher_suite_serialization() {
        let suite = CipherSuite::Aes256Gcm;
        let json = serde_json::to_string(&suite).unwrap();
        let deserialized: CipherSuite = serde_json::from_str(&json).unwrap();
        assert_eq!(suite, deserialized);
    }

    #[test]
    fn test_different_nonces() {
        let key = AesGcmCipher::generate_key();
        let cipher = AesGcmCipher::new(&key).unwrap();
        let pt = b"hello";

        let ct1 = cipher.encrypt(pt).unwrap();
        let ct2 = cipher.encrypt(pt).unwrap();
        assert_ne!(ct1.nonce, ct2.nonce);
    }
}
