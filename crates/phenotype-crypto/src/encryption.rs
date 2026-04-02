//! Symmetric encryption using AES-256-GCM.
//!
//! Provides authenticated encryption with associated data (AEAD)
//! using AES in Galois/Counter Mode. All encrypted data includes
//! a 12-byte nonce and 16-byte authentication tag.

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use thiserror::Error;

/// AES-256-GCM nonce size in bytes.
pub const NONCE_SIZE: usize = 12;

/// AES-256 key size in bytes.
pub const KEY_SIZE: usize = 32;

/// Errors related to encryption and decryption.
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid key size (expected {}, got {})", KEY_SIZE, .0)]
    InvalidKeySize(usize),

    #[error("Invalid nonce size (expected {}, got {})", NONCE_SIZE, .0)]
    InvalidNonceSize(usize),

    #[error("Hex decode error: {0}")]
    HexDecodeError(String),
}

/// Generate a random nonce suitable for AES-GCM encryption.
fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce
}

/// Encrypt data using AES-256-GCM with a random nonce.
///
/// The returned ciphertext includes the nonce prepended to the encrypted data.
///
/// # Arguments
/// - `plaintext`: Data to encrypt
/// - `key`: 32-byte encryption key
/// - `aad`: Optional associated data (authenticated but not encrypted)
///
/// # Returns
/// A vector containing: nonce (12 bytes) + ciphertext + tag (16 bytes)
///
/// # Errors
/// Returns `CryptoError` if the key size is invalid or encryption fails.
pub fn encrypt_aes_gcm(
    plaintext: &[u8],
    key: &[u8],
    aad: Option<&[u8]>,
) -> Result<Vec<u8>, CryptoError> {
    if key.len() != KEY_SIZE {
        return Err(CryptoError::InvalidKeySize(key.len()));
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let payload = match aad {
        Some(aad_data) => Payload {
            msg: plaintext,
            aad: aad_data,
        },
        None => Payload {
            msg: plaintext,
            aad: b"",
        },
    };

    let ciphertext = cipher
        .encrypt(nonce, payload)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    // Return nonce + ciphertext + tag
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt data encrypted with AES-256-GCM.
///
/// The input must be in the format produced by `encrypt_aes_gcm`:
/// nonce (12 bytes) + ciphertext + tag (16 bytes).
///
/// # Arguments
/// - `ciphertext`: Encrypted data with prepended nonce
/// - `key`: 32-byte decryption key
/// - `aad`: Optional associated data (must match what was used during encryption)
///
/// # Returns
/// The decrypted plaintext
///
/// # Errors
/// Returns `CryptoError` if:
/// - Key size is invalid
/// - Ciphertext is too short to contain a nonce
/// - Decryption or authentication fails
pub fn decrypt_aes_gcm(
    ciphertext: &[u8],
    key: &[u8],
    aad: Option<&[u8]>,
) -> Result<Vec<u8>, CryptoError> {
    if key.len() != KEY_SIZE {
        return Err(CryptoError::InvalidKeySize(key.len()));
    }

    if ciphertext.len() < NONCE_SIZE {
        return Err(CryptoError::DecryptionFailed(
            "Ciphertext too short to contain nonce".to_string(),
        ));
    }

    let (nonce_bytes, encrypted) = ciphertext.split_at(NONCE_SIZE);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    let nonce = Nonce::from_slice(nonce_bytes);

    let payload = match aad {
        Some(aad_data) => Payload {
            msg: encrypted,
            aad: aad_data,
        },
        None => Payload {
            msg: encrypted,
            aad: b"",
        },
    };

    cipher
        .decrypt(nonce, payload)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
}

/// Encrypt data and return as hex string (nonce + ciphertext + tag).
pub fn encrypt_aes_gcm_hex(
    plaintext: &[u8],
    key: &[u8],
    aad: Option<&[u8]>,
) -> Result<String, CryptoError> {
    let encrypted = encrypt_aes_gcm(plaintext, key, aad)?;
    Ok(hex::encode(encrypted))
}

/// Decrypt hex-encoded data encrypted with AES-256-GCM.
pub fn decrypt_aes_gcm_hex(
    hex_ciphertext: &str,
    key: &[u8],
    aad: Option<&[u8]>,
) -> Result<Vec<u8>, CryptoError> {
    let ciphertext = hex::decode(hex_ciphertext)
        .map_err(|e| CryptoError::HexDecodeError(e.to_string()))?;
    decrypt_aes_gcm(&ciphertext, key, aad)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = b"Hello, World!";
        let key = [42u8; KEY_SIZE];

        let encrypted = encrypt_aes_gcm(plaintext, &key, None).expect("Encryption failed");
        assert!(encrypted.len() > plaintext.len()); // Includes nonce and tag

        let decrypted = decrypt_aes_gcm(&encrypted, &key, None).expect("Decryption failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_with_aad() {
        let plaintext = b"Secret message";
        let key = [99u8; KEY_SIZE];
        let aad = b"Header data";

        let encrypted =
            encrypt_aes_gcm(plaintext, &key, Some(aad)).expect("Encryption with AAD failed");
        let decrypted =
            decrypt_aes_gcm(&encrypted, &key, Some(aad)).expect("Decryption with AAD failed");

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_fails_with_wrong_aad() {
        let plaintext = b"Secret";
        let key = [11u8; KEY_SIZE];
        let aad = b"Correct AAD";
        let wrong_aad = b"Wrong AAD";

        let encrypted = encrypt_aes_gcm(plaintext, &key, Some(aad)).expect("Encryption failed");

        let result = decrypt_aes_gcm(&encrypted, &key, Some(wrong_aad));
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_fails_with_wrong_key() {
        let plaintext = b"Secret";
        let key = [11u8; KEY_SIZE];
        let wrong_key = [22u8; KEY_SIZE];

        let encrypted = encrypt_aes_gcm(plaintext, &key, None).expect("Encryption failed");

        let result = decrypt_aes_gcm(&encrypted, &wrong_key, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_fails_with_invalid_key_size() {
        let plaintext = b"Test";
        let short_key = [42u8; 16]; // Wrong size

        let result = encrypt_aes_gcm(plaintext, &short_key, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_fails_with_invalid_key_size() {
        let ciphertext = [42u8; 50];
        let short_key = [42u8; 16];

        let result = decrypt_aes_gcm(&ciphertext, &short_key, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_fails_with_short_ciphertext() {
        let short_ciphertext = [42u8; 5]; // Too short for nonce
        let key = [11u8; KEY_SIZE];

        let result = decrypt_aes_gcm(&short_ciphertext, &key, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_corrupted_ciphertext() {
        let plaintext = b"Secret message";
        let key = [77u8; KEY_SIZE];

        let mut encrypted = encrypt_aes_gcm(plaintext, &key, None).expect("Encryption failed");

        // Corrupt the ciphertext (keep nonce, corrupt encrypted data)
        if encrypted.len() > NONCE_SIZE + 1 {
            encrypted[NONCE_SIZE + 1] ^= 0xFF;
        }

        let result = decrypt_aes_gcm(&encrypted, &key, None);
        assert!(result.is_err(), "Corrupted ciphertext should fail authentication");
    }

    #[test]
    fn test_encrypt_decrypt_hex_roundtrip() {
        let plaintext = b"Test data";
        let key = [55u8; KEY_SIZE];

        let hex_encrypted =
            encrypt_aes_gcm_hex(plaintext, &key, None).expect("Hex encryption failed");
        let decrypted =
            decrypt_aes_gcm_hex(&hex_encrypted, &key, None).expect("Hex decryption failed");

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_nonce_is_random() {
        let plaintext = b"Same message";
        let key = [33u8; KEY_SIZE];

        let encrypted1 = encrypt_aes_gcm(plaintext, &key, None).expect("Encryption 1 failed");
        let encrypted2 = encrypt_aes_gcm(plaintext, &key, None).expect("Encryption 2 failed");

        // First NONCE_SIZE bytes should be different nonces
        let nonce1 = &encrypted1[..NONCE_SIZE];
        let nonce2 = &encrypted2[..NONCE_SIZE];

        assert_ne!(nonce1, nonce2, "Nonces should be random");
    }

    #[test]
    fn test_empty_plaintext() {
        let plaintext = b"";
        let key = [88u8; KEY_SIZE];

        let encrypted = encrypt_aes_gcm(plaintext, &key, None).expect("Encryption failed");
        let decrypted = decrypt_aes_gcm(&encrypted, &key, None).expect("Decryption failed");

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_large_plaintext() {
        let plaintext = vec![42u8; 100_000];
        let key = [99u8; KEY_SIZE];

        let encrypted =
            encrypt_aes_gcm(&plaintext, &key, None).expect("Encryption of large data failed");
        let decrypted = decrypt_aes_gcm(&encrypted, &key, None).expect("Decryption failed");

        assert_eq!(decrypted, plaintext);
    }
}
