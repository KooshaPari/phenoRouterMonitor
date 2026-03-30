//! Key derivation using PBKDF2 (Password-Based Key Derivation Function 2).

use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use thiserror::Error;

/// Default iteration count for PBKDF2 (time-cost).
pub const DEFAULT_ITERATIONS: u32 = 600_000;

/// Salt length in bytes.
pub const SALT_LEN: usize = 16;

/// Derived key length in bytes.
pub const KEY_LEN: usize = 32;

/// Errors related to key derivation.
#[derive(Debug, Error)]
pub enum KdfError {
    #[error("Invalid salt length (expected {}, got {})", SALT_LEN, .0)]
    InvalidSaltLength(usize),

    #[error("Hex decode error: {0}")]
    DecodeError(String),

    #[error("Invalid iteration count (must be > 0)")]
    InvalidIterationCount,
}

/// Key derivation function using PBKDF2-SHA256.
pub struct Pbkdf2Kdf {
    iterations: u32,
}

impl Pbkdf2Kdf {
    /// Create a new KDF with the default iteration count.
    pub fn new() -> Self {
        Self {
            iterations: DEFAULT_ITERATIONS,
        }
    }

    /// Create a new KDF with a custom iteration count.
    ///
    /// # Errors
    /// Returns `KdfError::InvalidIterationCount` if iterations is 0.
    pub fn with_iterations(iterations: u32) -> Result<Self, KdfError> {
        if iterations == 0 {
            return Err(KdfError::InvalidIterationCount);
        }
        Ok(Self { iterations })
    }

    /// Derive a key from a password and salt.
    ///
    /// # Arguments
    /// - `password`: The password to derive from
    /// - `salt`: A 16-byte salt (usually random)
    ///
    /// # Errors
    /// Returns `KdfError::InvalidSaltLength` if salt is not exactly 16 bytes.
    ///
    /// # Returns
    /// A 32-byte derived key
    pub fn derive(&self, password: &[u8], salt: &[u8]) -> Result<[u8; KEY_LEN], KdfError> {
        if salt.len() != SALT_LEN {
            return Err(KdfError::InvalidSaltLength(salt.len()));
        }

        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(password, salt, self.iterations, &mut key);
        Ok(key)
    }

    /// Derive a key and return as hex string.
    pub fn derive_hex(&self, password: &[u8], salt: &[u8]) -> Result<String, KdfError> {
        let key = self.derive(password, salt)?;
        Ok(hex::encode(key))
    }

    /// Derive a key from a password and hex-encoded salt.
    pub fn derive_from_hex_salt(
        &self,
        password: &[u8],
        salt_hex: &str,
    ) -> Result<[u8; KEY_LEN], KdfError> {
        let salt = hex::decode(salt_hex).map_err(|e| KdfError::DecodeError(e.to_string()))?;
        self.derive(password, &salt)
    }

    /// Get the iteration count.
    pub fn iterations(&self) -> u32 {
        self.iterations
    }
}

impl Default for Pbkdf2Kdf {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a random salt suitable for key derivation.
pub fn generate_salt() -> [u8; SALT_LEN] {
    use rand::RngCore;
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Generate a random salt and return as hex string.
pub fn generate_salt_hex() -> String {
    hex::encode(generate_salt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbkdf2_deterministic() {
        let kdf = Pbkdf2Kdf::new();
        let password = b"my-password";
        let salt = [1u8; SALT_LEN];

        let key1 = kdf.derive(password, &salt).expect("Failed");
        let key2 = kdf.derive(password, &salt).expect("Failed");

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_pbkdf2_different_salts() {
        let kdf = Pbkdf2Kdf::new();
        let password = b"password";
        let salt1 = [1u8; SALT_LEN];
        let salt2 = [2u8; SALT_LEN];

        let key1 = kdf.derive(password, &salt1).expect("Failed");
        let key2 = kdf.derive(password, &salt2).expect("Failed");

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_pbkdf2_different_passwords() {
        let kdf = Pbkdf2Kdf::new();
        let salt = [1u8; SALT_LEN];

        let key1 = kdf.derive(b"password1", &salt).expect("Failed");
        let key2 = kdf.derive(b"password2", &salt).expect("Failed");

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_pbkdf2_invalid_salt_length() {
        let kdf = Pbkdf2Kdf::new();
        let short_salt = [1u8; 8];

        let result = kdf.derive(b"password", &short_salt);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid salt length"));
    }

    #[test]
    fn test_pbkdf2_hex_encoding() {
        let kdf = Pbkdf2Kdf::new();
        let password = b"test-password";
        let salt = [42u8; SALT_LEN];

        let key = kdf.derive(password, &salt).expect("Failed");
        let hex = hex::encode(key);

        assert_eq!(hex.len(), KEY_LEN * 2); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_pbkdf2_derive_hex() {
        let kdf = Pbkdf2Kdf::new();
        let password = b"password";
        let salt = [1u8; SALT_LEN];

        let key = kdf.derive(password, &salt).expect("Failed");
        let key_hex = kdf.derive_hex(password, &salt).expect("Failed");

        assert_eq!(key_hex, hex::encode(key));
    }

    #[test]
    fn test_pbkdf2_from_hex_salt() {
        let kdf = Pbkdf2Kdf::new();
        let password = b"password";
        let salt = [1u8; SALT_LEN];
        let salt_hex = hex::encode(salt);

        let key1 = kdf.derive(password, &salt).expect("Failed");
        let key2 = kdf
            .derive_from_hex_salt(password, &salt_hex)
            .expect("Failed");

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_custom_iterations() {
        let kdf = Pbkdf2Kdf::with_iterations(100_000).expect("Failed");
        let password = b"password";
        let salt = [1u8; SALT_LEN];

        let key = kdf.derive(password, &salt).expect("Failed");
        assert_eq!(key.len(), KEY_LEN);
    }

    #[test]
    fn test_zero_iterations_fails() {
        let result = Pbkdf2Kdf::with_iterations(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_iterations_produce_different_keys() {
        let kdf1 = Pbkdf2Kdf::with_iterations(100_000).expect("Failed");
        let kdf2 = Pbkdf2Kdf::with_iterations(200_000).expect("Failed");
        let password = b"password";
        let salt = [1u8; SALT_LEN];

        let key1 = kdf1.derive(password, &salt).expect("Failed");
        let key2 = kdf2.derive(password, &salt).expect("Failed");

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();

        assert_eq!(salt1.len(), SALT_LEN);
        assert_eq!(salt2.len(), SALT_LEN);
        // Random salts should be different
        assert_ne!(salt1, salt2);
    }

    #[test]
    fn test_generate_salt_hex() {
        let salt_hex = generate_salt_hex();
        assert_eq!(salt_hex.len(), SALT_LEN * 2); // 16 bytes = 32 hex chars
    }

    #[test]
    fn test_empty_password() {
        let kdf = Pbkdf2Kdf::new();
        let salt = [1u8; SALT_LEN];

        let key = kdf.derive(b"", &salt).expect("Failed");
        assert_eq!(key.len(), KEY_LEN);
    }

    #[test]
    fn test_long_password() {
        let kdf = Pbkdf2Kdf::new();
        let password = vec![42u8; 1000];
        let salt = [1u8; SALT_LEN];

        let key = kdf.derive(&password, &salt).expect("Failed");
        assert_eq!(key.len(), KEY_LEN);
    }

    #[test]
    fn test_iterations_getter() {
        let kdf = Pbkdf2Kdf::with_iterations(100_000).expect("Failed");
        assert_eq!(kdf.iterations(), 100_000);
    }
}
