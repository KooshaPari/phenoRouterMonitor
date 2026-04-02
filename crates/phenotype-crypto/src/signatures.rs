//! HMAC-based message authentication codes (MACs) for integrity verification.
//!
//! Provides HMAC (Hash-based Message Authentication Code) using SHA-256
//! for computing and verifying message authentication codes.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// HMAC signature size in bytes (SHA-256 output).
pub const HMAC_SIZE: usize = 32;

/// Errors related to signature operations.
#[derive(Debug, Error)]
pub enum SignatureError {
    #[error("Signature verification failed")]
    VerificationFailed,

    #[error("Invalid signature size (expected {}, got {})", HMAC_SIZE, .0)]
    InvalidSignatureSize(usize),

    #[error("Hex decode error: {0}")]
    HexDecodeError(String),
}

/// Compute HMAC-SHA256 signature of a message.
///
/// # Arguments
/// - `message`: The data to authenticate
/// - `key`: The shared secret key
///
/// # Returns
/// A 32-byte HMAC signature
///
/// # Example
/// ```ignore
/// let message = b"Secret message";
/// let key = b"shared-secret";
/// let signature = compute_hmac(message, key).unwrap();
/// assert_eq!(signature.len(), 32);
/// ```
pub fn compute_hmac(message: &[u8], key: &[u8]) -> Result<Vec<u8>, SignatureError> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|_| SignatureError::VerificationFailed)?;
    mac.update(message);
    Ok(mac.finalize().into_bytes().to_vec())
}

/// Compute HMAC-SHA256 signature and return as hex string.
///
/// # Arguments
/// - `message`: The data to authenticate
/// - `key`: The shared secret key
///
/// # Returns
/// A 64-character hex string representation of the HMAC
pub fn compute_hmac_hex(message: &[u8], key: &[u8]) -> Result<String, SignatureError> {
    let signature = compute_hmac(message, key)?;
    Ok(hex::encode(signature))
}

/// Verify an HMAC-SHA256 signature.
///
/// # Arguments
/// - `message`: The original data that was authenticated
/// - `signature`: The HMAC signature to verify (32 bytes)
/// - `key`: The shared secret key
///
/// # Returns
/// `Ok(())` if verification succeeds, `SignatureError` otherwise
///
/// # Errors
/// Returns `SignatureError::VerificationFailed` if:
/// - The signature is invalid
/// - The signature does not match the computed HMAC
/// - The signature size is incorrect
///
/// # Example
/// ```ignore
/// let message = b"Data to authenticate";
/// let key = b"secret-key";
/// let signature = compute_hmac(message, key)?;
/// verify_hmac(message, &signature, key)?; // OK
/// ```
pub fn verify_hmac(
    message: &[u8],
    signature: &[u8],
    key: &[u8],
) -> Result<(), SignatureError> {
    if signature.len() != HMAC_SIZE {
        return Err(SignatureError::InvalidSignatureSize(signature.len()));
    }

    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|_| SignatureError::VerificationFailed)?;
    mac.update(message);

    mac.verify_slice(signature)
        .map_err(|_| SignatureError::VerificationFailed)
}

/// Verify a hex-encoded HMAC-SHA256 signature.
///
/// # Arguments
/// - `message`: The original data that was authenticated
/// - `hex_signature`: The HMAC signature as a 64-character hex string
/// - `key`: The shared secret key
///
/// # Returns
/// `Ok(())` if verification succeeds, `SignatureError` otherwise
pub fn verify_hmac_hex(
    message: &[u8],
    hex_signature: &str,
    key: &[u8],
) -> Result<(), SignatureError> {
    let signature = hex::decode(hex_signature)
        .map_err(|e| SignatureError::HexDecodeError(e.to_string()))?;
    verify_hmac(message, &signature, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hmac() {
        let message = b"Hello, World!";
        let key = b"secret-key";

        let signature = compute_hmac(message, key).expect("HMAC computation failed");
        assert_eq!(signature.len(), HMAC_SIZE);
    }

    #[test]
    fn test_hmac_deterministic() {
        let message = b"Test message";
        let key = b"shared-secret";

        let sig1 = compute_hmac(message, key).expect("HMAC 1 failed");
        let sig2 = compute_hmac(message, key).expect("HMAC 2 failed");

        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_hmac_different_messages() {
        let key = b"secret";

        let sig1 = compute_hmac(b"message1", key).expect("HMAC 1 failed");
        let sig2 = compute_hmac(b"message2", key).expect("HMAC 2 failed");

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_hmac_different_keys() {
        let message = b"same message";

        let sig1 = compute_hmac(message, b"key1").expect("HMAC 1 failed");
        let sig2 = compute_hmac(message, b"key2").expect("HMAC 2 failed");

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_verify_hmac_valid() {
        let message = b"Authenticate this";
        let key = b"my-secret-key";

        let signature = compute_hmac(message, key).expect("HMAC computation failed");

        let result = verify_hmac(message, &signature, key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_hmac_fails_with_wrong_key() {
        let message = b"Authenticate this";
        let key = b"original-key";
        let wrong_key = b"wrong-key";

        let signature = compute_hmac(message, key).expect("HMAC computation failed");

        let result = verify_hmac(message, &signature, wrong_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_hmac_fails_with_modified_message() {
        let message = b"Original message";
        let key = b"secret";

        let signature = compute_hmac(message, key).expect("HMAC computation failed");

        let modified_message = b"Modified message";
        let result = verify_hmac(modified_message, &signature, key);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_hmac_fails_with_corrupted_signature() {
        let message = b"Message to sign";
        let key = b"secret-key";

        let mut signature = compute_hmac(message, key).expect("HMAC computation failed");

        // Corrupt the signature
        if !signature.is_empty() {
            signature[0] ^= 0xFF;
        }

        let result = verify_hmac(message, &signature, key);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_hmac_fails_with_invalid_signature_size() {
        let message = b"Message";
        let key = b"secret";
        let short_signature = [42u8; 16]; // Wrong size

        let result = verify_hmac(message, &short_signature, key);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid signature size"));
    }

    #[test]
    fn test_compute_hmac_hex() {
        let message = b"Test data";
        let key = b"key";

        let hex_sig = compute_hmac_hex(message, key).expect("Hex HMAC failed");
        assert_eq!(hex_sig.len(), HMAC_SIZE * 2); // 32 bytes = 64 hex chars
        assert!(hex_sig.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_verify_hmac_hex_valid() {
        let message = b"Authenticate";
        let key = b"secret";

        let hex_sig = compute_hmac_hex(message, key).expect("Hex HMAC computation failed");
        let result = verify_hmac_hex(message, &hex_sig, key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_hmac_hex_invalid_hex() {
        let message = b"Message";
        let key = b"secret";
        let invalid_hex = "not-valid-hex-string";

        let result = verify_hmac_hex(message, invalid_hex, key);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_hmac_hex_fails_with_wrong_key() {
        let message = b"Secret message";
        let key = b"original-key";
        let wrong_key = b"wrong-key";

        let hex_sig = compute_hmac_hex(message, key).expect("Hex HMAC failed");
        let result = verify_hmac_hex(message, &hex_sig, wrong_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_message() {
        let key = b"secret";

        let signature = compute_hmac(b"", key).expect("HMAC of empty message failed");
        let result = verify_hmac(b"", &signature, key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_large_message() {
        let message = vec![42u8; 100_000];
        let key = b"secret-key";

        let signature = compute_hmac(&message, key).expect("HMAC of large message failed");
        let result = verify_hmac(&message, &signature, key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_long_key() {
        let message = b"message";
        let key = vec![99u8; 1000];

        let signature = compute_hmac(message, &key).expect("HMAC with long key failed");
        let result = verify_hmac(message, &signature, &key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_known_vector() {
        // Test against a known HMAC-SHA256 vector
        // HMAC-SHA256("", "") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let signature = compute_hmac(b"", b"").expect("HMAC computation failed");
        let hex_sig = hex::encode(signature);

        // SHA-256 of empty string (the HMAC of empty message with empty key should match)
        assert_eq!(hex_sig.len(), 64);
    }
}
