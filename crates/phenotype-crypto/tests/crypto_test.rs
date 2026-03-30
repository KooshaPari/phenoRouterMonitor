//! Integration tests for phenotype-crypto
//!
//! Tests comprehensive cryptographic workflows combining:
//! - Hashing (SHA-256, Blake3)
//! - Symmetric encryption (AES-256-GCM)
//! - Key derivation (PBKDF2)
//! - Message authentication (HMAC)

use phenotype_crypto::{
    blake3_hash, content_id, decrypt_aes_gcm, encrypt_aes_gcm, sha256_hash, HashAlgorithm,
    Pbkdf2Kdf, compute_hmac, verify_hmac, compute_hmac_hex, verify_hmac_hex,
};

#[test]
fn test_complete_encryption_workflow() {
    // 1. Derive encryption key from password
    let password = b"my-secure-password";
    let kdf = Pbkdf2Kdf::new();
    let salt = [42u8; 16];

    let key = kdf.derive(password, &salt)
        .expect("Key derivation failed");

    // 2. Encrypt sensitive data
    let plaintext = b"Highly confidential information";
    let encrypted = encrypt_aes_gcm(plaintext, &key, None)
        .expect("Encryption failed");

    // 3. Verify encryption worked
    assert!(encrypted.len() > plaintext.len());

    // 4. Decrypt and verify
    let decrypted = decrypt_aes_gcm(&encrypted, &key, None)
        .expect("Decryption failed");
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_authenticated_encryption_with_aad() {
    // Demonstrate authenticated encryption with associated data
    let plaintext = b"Secret payload";
    let key = [33u8; 32];
    let aad = b"user-id:12345";

    // Encrypt with AAD
    let encrypted = encrypt_aes_gcm(plaintext, &key, Some(aad))
        .expect("Encryption with AAD failed");

    // Decrypt with same AAD
    let decrypted = decrypt_aes_gcm(&encrypted, &key, Some(aad))
        .expect("Decryption with AAD failed");
    assert_eq!(decrypted, plaintext);

    // Attempt with wrong AAD should fail
    let wrong_aad = b"user-id:99999";
    let result = decrypt_aes_gcm(&encrypted, &key, Some(wrong_aad));
    assert!(result.is_err(), "Should fail with wrong AAD");
}

#[test]
fn test_hash_content_addressing() {
    // Test content-addressable hashing
    let content1 = b"Document version 1";
    let content2 = b"Document version 2";

    let id1 = content_id(content1, HashAlgorithm::Sha256);
    let id2 = content_id(content2, HashAlgorithm::Sha256);

    assert!(id1.starts_with("sha256:"));
    assert!(id2.starts_with("sha256:"));
    assert_ne!(id1, id2);

    // Same content produces same ID
    let id1_again = content_id(content1, HashAlgorithm::Sha256);
    assert_eq!(id1, id1_again);
}

#[test]
fn test_blake3_hashing() {
    let data = b"Test data for Blake3";

    let hash1 = blake3_hash(data);
    let hash2 = blake3_hash(data);

    assert_eq!(hash1, hash2); // Deterministic
    assert_eq!(hash1.len(), 64); // 32 bytes in hex
}

#[test]
fn test_key_derivation_consistency() {
    let password = b"my-password";
    let salt = [1u8; 16];

    let kdf = Pbkdf2Kdf::new();

    let key1 = kdf.derive(password, &salt)
        .expect("Key derivation 1 failed");
    let key2 = kdf.derive(password, &salt)
        .expect("Key derivation 2 failed");

    assert_eq!(key1, key2);
}

#[test]
fn test_hmac_message_authentication() {
    // Sign a message
    let message = b"Important transaction data";
    let key = b"shared-secret-key";

    let signature = compute_hmac(message, key)
        .expect("HMAC computation failed");

    // Verify the signature
    let result = verify_hmac(message, &signature, key);
    assert!(result.is_ok());

    // Modify message and verify failure
    let modified = b"Tampered transaction data";
    let result = verify_hmac(modified, &signature, key);
    assert!(result.is_err());
}

#[test]
fn test_end_to_end_secure_storage() {
    // Comprehensive workflow: derive key, encrypt, compute MAC
    let password = b"user-password";
    let plaintext = b"Private user data";

    // Step 1: Derive encryption key
    let kdf = Pbkdf2Kdf::new();
    let salt = [99u8; 16];
    let encryption_key = kdf.derive(password, &salt)
        .expect("Key derivation failed");

    // Step 2: Encrypt data
    let encrypted = encrypt_aes_gcm(plaintext, &encryption_key, None)
        .expect("Encryption failed");

    // Step 3: Compute HMAC over encrypted data (for integrity)
    let auth_key = b"auth-secret";
    let signature = compute_hmac(&encrypted, auth_key)
        .expect("HMAC computation failed");

    // Step 4: Verify integrity
    let integrity_ok = verify_hmac(&encrypted, &signature, auth_key);
    assert!(integrity_ok.is_ok());

    // Step 5: Decrypt
    let decrypted = decrypt_aes_gcm(&encrypted, &encryption_key, None)
        .expect("Decryption failed");
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_hex_encoding_roundtrip() {
    let plaintext = b"Test message";
    let key = [77u8; 32];

    // Encrypt to hex
    let hex_encrypted = phenotype_crypto::encryption::encrypt_aes_gcm_hex(plaintext, &key, None)
        .expect("Hex encryption failed");

    // Decrypt from hex
    let decrypted = phenotype_crypto::encryption::decrypt_aes_gcm_hex(&hex_encrypted, &key, None)
        .expect("Hex decryption failed");

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_hmac_hex_workflow() {
    let message = b"Authenticate this message";
    let key = b"secret-key";

    // Compute HMAC in hex format
    let hex_signature = compute_hmac_hex(message, key)
        .expect("Hex HMAC computation failed");

    // Verify with hex signature
    let result = verify_hmac_hex(message, &hex_signature, key);
    assert!(result.is_ok());

    // Tampering should fail
    let modified = b"Tampered message";
    let result = verify_hmac_hex(modified, &hex_signature, key);
    assert!(result.is_err());
}

#[test]
fn test_various_key_sizes() {
    // Test KDF with different iteration counts
    let password = b"password";
    let salt = [1u8; 16];

    let kdf_fast = Pbkdf2Kdf::with_iterations(10_000)
        .expect("KDF creation failed");
    let kdf_slow = Pbkdf2Kdf::with_iterations(600_000)
        .expect("KDF creation failed");

    let key_fast = kdf_fast.derive(password, &salt)
        .expect("Fast KDF failed");
    let key_slow = kdf_slow.derive(password, &salt)
        .expect("Slow KDF failed");

    // Different iteration counts produce different keys
    assert_ne!(key_fast, key_slow);
}

#[test]
fn test_encryption_with_multiple_aad() {
    // Encrypt with structured AAD
    let plaintext = b"Confidential";
    let key = [55u8; 32];
    let aad = b"scope:admin,user:alice,action:read";

    let encrypted = encrypt_aes_gcm(plaintext, &key, Some(aad))
        .expect("Encryption failed");

    // Decrypt with same AAD
    let decrypted = decrypt_aes_gcm(&encrypted, &key, Some(aad))
        .expect("Decryption failed");
    assert_eq!(decrypted, plaintext);

    // Different AAD fails
    let different_aad = b"scope:user,user:bob,action:write";
    let result = decrypt_aes_gcm(&encrypted, &key, Some(different_aad));
    assert!(result.is_err());
}

#[test]
fn test_hash_algorithms_produce_different_outputs() {
    let data = b"Test data";

    let sha256 = sha256_hash(data);
    let blake3 = blake3_hash(data);

    assert_ne!(sha256, blake3);
    assert_eq!(sha256.len(), 64); // SHA-256: 32 bytes = 64 hex chars
    assert_eq!(blake3.len(), 64); // Blake3: 32 bytes = 64 hex chars
}

#[test]
fn test_large_data_encryption() {
    // Test encryption of larger payloads
    let plaintext = vec![42u8; 1_000_000]; // 1MB
    let key = [88u8; 32];

    let encrypted = encrypt_aes_gcm(&plaintext, &key, None)
        .expect("Large data encryption failed");

    let decrypted = decrypt_aes_gcm(&encrypted, &key, None)
        .expect("Large data decryption failed");

    assert_eq!(decrypted.len(), plaintext.len());
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_concurrent_encryption() {
    // Verify encryption works independently with different keys/data
    let plaintext1 = b"Message 1";
    let plaintext2 = b"Message 2";
    let key1 = [11u8; 32];
    let key2 = [22u8; 32];

    let encrypted1 = encrypt_aes_gcm(plaintext1, &key1, None)
        .expect("Encryption 1 failed");
    let encrypted2 = encrypt_aes_gcm(plaintext2, &key2, None)
        .expect("Encryption 2 failed");

    // Should not cross-decrypt
    assert!(decrypt_aes_gcm(&encrypted1, &key2, None).is_err());
    assert!(decrypt_aes_gcm(&encrypted2, &key1, None).is_err());

    // But should decrypt with correct keys
    let decrypted1 = decrypt_aes_gcm(&encrypted1, &key1, None)
        .expect("Decryption 1 failed");
    let decrypted2 = decrypt_aes_gcm(&encrypted2, &key2, None)
        .expect("Decryption 2 failed");

    assert_eq!(decrypted1, plaintext1);
    assert_eq!(decrypted2, plaintext2);
}
