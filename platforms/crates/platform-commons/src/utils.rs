//! Utility functions

/// Returns the current platform identifier.
pub fn current_platform() -> &'static str {
    "platforms"
}

/// Calculates a simple hash of the input string.
pub fn simple_hash(input: &str) -> u64 {
    input.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
}
