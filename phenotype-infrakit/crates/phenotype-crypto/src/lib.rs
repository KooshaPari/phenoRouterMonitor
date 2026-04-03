//! BKM-03: sign_artifact, verify_signature, artifact_hash for thegent.
//! Expects canonical JSON bytes (sorted keys). Python uses orjson for canonical.

use base16ct::lower;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

#[cfg(all(feature = "python", not(test)))]
use pyo3::prelude::*;

type HmacSha256 = Hmac<Sha256>;

/// Compute SHA-256 hex digest of canonical JSON bytes.
#[cfg_attr(all(feature = "python", not(test)), pyfunction)]
pub fn artifact_hash_bytes(canonical_json: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(canonical_json);
    let hash = hasher.finalize();
    let mut buf = vec![0u8; base16ct::encoded_len(&hash)];
    let encoded = lower::encode(&hash, &mut buf).unwrap();
    String::from_utf8_lossy(encoded).to_string()
}

/// HMAC-SHA256 hex signature of canonical JSON bytes.
#[cfg_attr(all(feature = "python", not(test)), pyfunction)]
pub fn sign_artifact_bytes(canonical_json: &[u8], secret_key: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret_key.as_bytes()).expect("HMAC accepts any key size");
    mac.update(canonical_json);
    let result = mac.finalize();
    let bytes = result.into_bytes();
    let mut buf = vec![0u8; base16ct::encoded_len(&bytes)];
    let encoded = lower::encode(&bytes, &mut buf).unwrap();
    String::from_utf8_lossy(encoded).to_string()
}

/// Verify HMAC-SHA256 signature. Constant-time comparison.
#[cfg_attr(all(feature = "python", not(test)), pyfunction)]
pub fn verify_signature_bytes(canonical_json: &[u8], signature: &str, secret_key: &str) -> bool {
    use subtle::ConstantTimeEq;
    let expected = sign_artifact_bytes(canonical_json, secret_key);
    if expected.len() != signature.len() {
        return false;
    }
    expected.as_bytes().ct_eq(signature.as_bytes()).into()
}

#[cfg(all(feature = "python", not(test)))]
#[pymodule]
fn thegent_crypto(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(artifact_hash_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(sign_artifact_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(verify_signature_bytes, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test data fixtures
    const TEST_KEY: &str = "my-secret-key-for-testing";
    const ANOTHER_KEY: &str = "different-secret-key";
    const TEST_JSON: &[u8] = br#"{"key":"value","number":42}"#;
    const EMPTY_JSON: &[u8] = b"";
    const LONG_JSON: &[u8] = br#"{"nested":{"deeply":{"value":"this is a longer test payload to verify hash and signing works with various input sizes"},"array":[1,2,3,4,5]}}"#;
    const BINARY_DATA: &[u8] = &[0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD, 0x7F, 0x80];

    // Test 1: Hash function returns consistent output for same input
    #[test]
    fn test_artifact_hash_consistent() {
        let hash1 = artifact_hash_bytes(TEST_JSON);
        let hash2 = artifact_hash_bytes(TEST_JSON);
        assert_eq!(
            hash1, hash2,
            "Hash of same input should be identical (deterministic)"
        );
    }

    // Test 2: Hash function returns different output for different inputs
    #[test]
    fn test_artifact_hash_different_for_different_inputs() {
        let hash1 = artifact_hash_bytes(TEST_JSON);
        let hash2 = artifact_hash_bytes(br#"{"key":"value","number":43}"#);
        assert_ne!(hash1, hash2, "Hash of different inputs should be different");
    }

    // Test 3: Hash returns valid hex string
    #[test]
    fn test_artifact_hash_is_valid_hex() {
        let hash = artifact_hash_bytes(TEST_JSON);
        // SHA-256 produces 32 bytes = 64 hex characters
        assert_eq!(hash.len(), 64, "SHA-256 hex digest should be 64 characters");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should contain only hex characters"
        );
    }

    // Test 4: Hash works with empty input
    #[test]
    fn test_artifact_hash_empty_input() {
        let hash = artifact_hash_bytes(EMPTY_JSON);
        assert_eq!(hash.len(), 64, "SHA-256 hex digest should be 64 characters");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should contain only hex characters"
        );
        // Empty input should produce a known SHA-256 hash
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(
            hash, expected,
            "Empty input should produce known SHA-256 hash"
        );
    }

    // Test 5: Hash works with binary data
    #[test]
    fn test_artifact_hash_binary_data() {
        let hash = artifact_hash_bytes(BINARY_DATA);
        assert_eq!(hash.len(), 64, "SHA-256 hex digest should be 64 characters");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should contain only hex characters"
        );
    }

    // Test 6: Hash works with very long input
    #[test]
    fn test_artifact_hash_long_input() {
        let hash = artifact_hash_bytes(LONG_JSON);
        assert_eq!(hash.len(), 64, "SHA-256 hex digest should be 64 characters");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should contain only hex characters"
        );
    }

    // Test 7: Sign and verify roundtrip - signature verifies with correct key
    #[test]
    fn test_sign_and_verify_roundtrip() {
        let signature = sign_artifact_bytes(TEST_JSON, TEST_KEY);
        let is_valid = verify_signature_bytes(TEST_JSON, &signature, TEST_KEY);
        assert!(
            is_valid,
            "Signature should verify with the correct key and original message"
        );
    }

    // Test 8: Verify fails with wrong key
    #[test]
    fn test_verify_fails_with_wrong_key() {
        let signature = sign_artifact_bytes(TEST_JSON, TEST_KEY);
        let is_valid = verify_signature_bytes(TEST_JSON, &signature, ANOTHER_KEY);
        assert!(
            !is_valid,
            "Signature verification should fail with a different key"
        );
    }

    // Test 9: Verify fails with tampered message
    #[test]
    fn test_verify_fails_with_tampered_message() {
        let signature = sign_artifact_bytes(TEST_JSON, TEST_KEY);
        let tampered_json = br#"{"key":"value","number":43}"#;
        let is_valid = verify_signature_bytes(tampered_json, &signature, TEST_KEY);
        assert!(
            !is_valid,
            "Signature verification should fail with a modified message"
        );
    }

    // Test 10: Verify fails with tampered signature
    #[test]
    fn test_verify_fails_with_tampered_signature() {
        let signature = sign_artifact_bytes(TEST_JSON, TEST_KEY);
        // Tamper with signature by flipping a bit
        let mut tampered = signature.clone();
        if let Some(first_char) = tampered.chars().next() {
            let hex_digit = first_char.to_digit(16).unwrap_or(0);
            let flipped = (hex_digit ^ 0x01) as u8;
            let tampered_first = format!("{:x}", flipped);
            tampered.replace_range(0..1, &tampered_first);
        }
        let is_valid = verify_signature_bytes(TEST_JSON, &tampered, TEST_KEY);
        assert!(
            !is_valid,
            "Signature verification should fail with a modified signature"
        );
    }

    // Test 11: Sign returns valid hex string
    #[test]
    fn test_sign_returns_valid_hex() {
        let signature = sign_artifact_bytes(TEST_JSON, TEST_KEY);
        // HMAC-SHA256 produces 32 bytes = 64 hex characters
        assert_eq!(
            signature.len(),
            64,
            "HMAC-SHA256 hex digest should be 64 characters"
        );
        assert!(
            signature.chars().all(|c| c.is_ascii_hexdigit()),
            "Signature should contain only hex characters"
        );
    }

    // Test 12: Sign is deterministic for same inputs
    #[test]
    fn test_sign_deterministic() {
        let sig1 = sign_artifact_bytes(TEST_JSON, TEST_KEY);
        let sig2 = sign_artifact_bytes(TEST_JSON, TEST_KEY);
        assert_eq!(
            sig1, sig2,
            "Signature of same message with same key should be identical (deterministic)"
        );
    }

    // Test 13: Different keys produce different signatures
    #[test]
    fn test_sign_different_keys_produce_different_signatures() {
        let sig1 = sign_artifact_bytes(TEST_JSON, TEST_KEY);
        let sig2 = sign_artifact_bytes(TEST_JSON, ANOTHER_KEY);
        assert_ne!(
            sig1, sig2,
            "Different keys should produce different signatures"
        );
    }

    // Test 14: Sign works with empty message
    #[test]
    fn test_sign_empty_message() {
        let signature = sign_artifact_bytes(EMPTY_JSON, TEST_KEY);
        assert_eq!(
            signature.len(),
            64,
            "HMAC-SHA256 hex digest should be 64 characters"
        );
        // Verify that signing empty message works and can be verified
        let is_valid = verify_signature_bytes(EMPTY_JSON, &signature, TEST_KEY);
        assert!(is_valid, "Empty message signature should verify correctly");
    }

    // Test 15: Sign works with binary data
    #[test]
    fn test_sign_binary_data() {
        let signature = sign_artifact_bytes(BINARY_DATA, TEST_KEY);
        assert_eq!(
            signature.len(),
            64,
            "HMAC-SHA256 hex digest should be 64 characters"
        );
        let is_valid = verify_signature_bytes(BINARY_DATA, &signature, TEST_KEY);
        assert!(is_valid, "Binary data signature should verify correctly");
    }

    // Test 16: Sign works with very long message
    #[test]
    fn test_sign_long_message() {
        let signature = sign_artifact_bytes(LONG_JSON, TEST_KEY);
        assert_eq!(
            signature.len(),
            64,
            "HMAC-SHA256 hex digest should be 64 characters"
        );
        let is_valid = verify_signature_bytes(LONG_JSON, &signature, TEST_KEY);
        assert!(is_valid, "Long message signature should verify correctly");
    }

    // Test 17: Verify with wrong signature format (too short)
    #[test]
    fn test_verify_fails_with_short_signature() {
        let signature = sign_artifact_bytes(TEST_JSON, TEST_KEY);
        let short_sig = &signature[0..32]; // Half the length
        let is_valid = verify_signature_bytes(TEST_JSON, short_sig, TEST_KEY);
        assert!(
            !is_valid,
            "Verification should fail with signature of wrong length"
        );
    }

    // Test 18: Verify with wrong signature format (too long)
    #[test]
    fn test_verify_fails_with_long_signature() {
        let signature = sign_artifact_bytes(TEST_JSON, TEST_KEY);
        let long_sig = format!("{}00", signature); // Add extra characters
        let is_valid = verify_signature_bytes(TEST_JSON, &long_sig, TEST_KEY);
        assert!(
            !is_valid,
            "Verification should fail with signature of wrong length"
        );
    }

    // Test 19: Constant-time comparison (verify should work regardless of where bit differs)
    #[test]
    fn test_verify_constant_time_comparison() {
        let signature = sign_artifact_bytes(TEST_JSON, TEST_KEY);

        // Test early bit difference
        let mut early_tampered = signature.clone();
        if let Some(first_char) = early_tampered.chars().next() {
            if first_char != 'f' {
                early_tampered.replace_range(0..1, "f");
            } else {
                early_tampered.replace_range(0..1, "0");
            }
        }

        // Test late bit difference
        let mut late_tampered = signature.clone();
        let len = late_tampered.len();
        if let Some(last_char) = late_tampered.chars().last() {
            if last_char != 'f' {
                late_tampered.replace_range(len - 1..len, "f");
            } else {
                late_tampered.replace_range(len - 1..len, "0");
            }
        }

        // Both should fail verification
        assert!(
            !verify_signature_bytes(TEST_JSON, &early_tampered, TEST_KEY),
            "Early bit tamper should fail"
        );
        assert!(
            !verify_signature_bytes(TEST_JSON, &late_tampered, TEST_KEY),
            "Late bit tamper should fail"
        );
    }

    // Test 20: Empty key is accepted (HMAC accepts any key size including empty)
    #[test]
    fn test_sign_with_empty_key() {
        let empty_key = "";
        let signature = sign_artifact_bytes(TEST_JSON, empty_key);
        assert_eq!(
            signature.len(),
            64,
            "HMAC-SHA256 hex digest should be 64 characters"
        );
        // Verify that signing with empty key works and is verifiable
        let is_valid = verify_signature_bytes(TEST_JSON, &signature, empty_key);
        assert!(is_valid, "Empty key signature should verify with empty key");
        // But should not verify with different key
        let is_invalid = verify_signature_bytes(TEST_JSON, &signature, "non-empty-key");
        assert!(
            !is_invalid,
            "Empty key signature should not verify with different key"
        );
    }

    // Test 21: Very long key
    #[test]
    fn test_sign_with_very_long_key() {
        let long_key = "k".repeat(1000);
        let signature = sign_artifact_bytes(TEST_JSON, &long_key);
        assert_eq!(
            signature.len(),
            64,
            "HMAC-SHA256 hex digest should be 64 characters"
        );
        let is_valid = verify_signature_bytes(TEST_JSON, &signature, &long_key);
        assert!(is_valid, "Long key signature should verify correctly");
    }

    // Test 22: Integration test - complete workflow
    #[test]
    fn test_complete_workflow() {
        // Sign a message
        let message = br#"{"action":"transfer","amount":100,"to":"address"}"#;
        let key = "production-secret-key";

        let signature = sign_artifact_bytes(message, key);
        let hash = artifact_hash_bytes(message);

        // Verify signature is valid
        assert!(
            verify_signature_bytes(message, &signature, key),
            "Signature should be valid for original message and key"
        );

        // Hash should be consistent
        let hash2 = artifact_hash_bytes(message);
        assert_eq!(hash, hash2, "Hash should be deterministic");

        // If message is tampered, signature should be invalid
        let tampered_message = br#"{"action":"transfer","amount":1000,"to":"address"}"#;
        assert!(
            !verify_signature_bytes(tampered_message, &signature, key),
            "Tampered message signature should fail"
        );

        // If key is tampered, signature should be invalid
        assert!(
            !verify_signature_bytes(message, &signature, "wrong-key"),
            "Wrong key signature verification should fail"
        );
    }
}
