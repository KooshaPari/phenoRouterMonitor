//! Secure random number generation abstraction.
//!
//! Provides the `SecureRandom` trait for abstraction over randomness sources
//! and a default implementation using the `rand` crate.
//! @trace FR-PHENO-CRYPTO-005

use rand::RngCore;
use thiserror::Error;

/// Errors related to random generation.
#[derive(Debug, Error)]
pub enum RandomError {
    #[error("Random generation failed: {0}")]
    GenerationFailed(String),
}

/// Trait for generating cryptographically secure random bytes.
/// Allows abstraction and testing with deterministic sources.
/// @trace FR-PHENO-CRYPTO-005
pub trait SecureRandom {
    /// Generate n random bytes.
    /// 
    /// # Errors
    /// Returns `RandomError` if generation fails.
    fn generate_bytes(&self, n: usize) -> Result<Vec<u8>, RandomError>;

    /// Fill a slice with random bytes.
    /// 
    /// # Errors
    /// Returns `RandomError` if generation fails.
    fn fill_bytes(&self, dest: &mut [u8]) -> Result<(), RandomError>;

    /// Generate a nonce of the specified size (typically 12 bytes for AES-GCM).
    fn generate_nonce(&self, size: usize) -> Result<Vec<u8>, RandomError> {
        self.generate_bytes(size)
    }
}

/// Default secure random implementation using the system RNG.
/// @trace FR-PHENO-CRYPTO-005
#[derive(Clone, Debug, Default)]
pub struct DefaultSecureRandom;

impl DefaultSecureRandom {
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl SecureRandom for DefaultSecureRandom {
    fn generate_bytes(&self, n: usize) -> Result<Vec<u8>, RandomError> {
        let mut bytes = vec![0u8; n];
        rand::thread_rng().fill_bytes(&mut bytes);
        Ok(bytes)
    }

    fn fill_bytes(&self, dest: &mut [u8]) -> Result<(), RandomError> {
        rand::thread_rng().fill_bytes(dest);
        Ok(())
    }
}

/// Generate random bytes using the default secure random implementation.
/// 
/// # Arguments
/// - `n`: Number of bytes to generate
/// 
/// # Returns
/// A vector of `n` cryptographically secure random bytes
/// 
/// # Example
/// ```ignore
/// let bytes = generate_random_bytes(32).expect("Failed to generate random bytes");
/// assert_eq!(bytes.len(), 32);
/// ```
/// @trace FR-PHENO-CRYPTO-005
pub fn generate_random_bytes(n: usize) -> Result<Vec<u8>, RandomError> {
    let rng = DefaultSecureRandom::new();
    rng.generate_bytes(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    // @trace FR-PHENO-CRYPTO-005
    #[test]
    fn test_default_secure_random_generate_bytes() {
        let rng = DefaultSecureRandom::new();
        let bytes = rng.generate_bytes(32).expect("Failed to generate bytes");
        assert_eq!(bytes.len(), 32);
    }

    // @trace FR-PHENO-CRYPTO-005
    #[test]
    fn test_default_secure_random_fill_bytes() {
        let rng = DefaultSecureRandom::new();
        let mut dest = [0u8; 16];
        rng.fill_bytes(&mut dest).expect("Failed to fill bytes");
        // At least one byte should be non-zero (extremely unlikely to be all zeros)
        assert!(dest.iter().any(|&b| b != 0), "Random bytes should not be all zeros");
    }

    // @trace FR-PHENO-CRYPTO-005
    #[test]
    fn test_generate_random_bytes() {
        let bytes = generate_random_bytes(64).expect("Failed to generate random bytes");
        assert_eq!(bytes.len(), 64);
        assert!(bytes.iter().any(|&b| b != 0), "Random bytes should not be all zeros");
    }

    // @trace FR-PHENO-CRYPTO-005
    #[test]
    fn test_random_bytes_are_different() {
        let bytes1 = generate_random_bytes(32).expect("Failed to generate bytes 1");
        let bytes2 = generate_random_bytes(32).expect("Failed to generate bytes 2");
        assert_ne!(bytes1, bytes2, "Two random generations should produce different results");
    }

    // @trace FR-PHENO-CRYPTO-005
    #[test]
    fn test_generate_nonce() {
        let rng = DefaultSecureRandom::new();
        let nonce = rng.generate_nonce(12).expect("Failed to generate nonce");
        assert_eq!(nonce.len(), 12);
    }

    // @trace FR-PHENO-CRYPTO-005
    #[test]
    fn test_large_random_bytes() {
        let bytes = generate_random_bytes(100_000).expect("Failed to generate large buffer");
        assert_eq!(bytes.len(), 100_000);
    }

    // @trace FR-PHENO-CRYPTO-005
    #[test]
    fn test_secure_random_trait() {
        let rng = DefaultSecureRandom::new();
        let bytes: Vec<u8> = (0..5)
            .map(|_| rng.generate_bytes(16).expect("Failed to generate bytes"))
            .collect();
        assert_eq!(bytes.len(), 5);
        assert!(bytes.iter().all(|b| b.len() == 16));
    }

    // @trace FR-PHENO-CRYPTO-005
    #[test]
    fn test_zero_byte_generation() {
        let rng = DefaultSecureRandom::new();
        let bytes = rng.generate_bytes(0).expect("Failed to generate 0 bytes");
        assert_eq!(bytes.len(), 0);
    }
}
