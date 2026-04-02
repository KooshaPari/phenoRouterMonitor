//! General utilities for cross-platform use.

/// Returns the current platform identifier.
pub fn current_platform() -> &'static str {
    // Returns "platforms" for now - each platform can override via cfg
    "platforms"
}

/// Calculates a simple hash of the input string.
/// Uses FNV-1a algorithm for cross-platform consistency.
pub fn fnv_hash(input: &str) -> u64 {
    const PRIME: u64 = 1099511628431;
    const OFFSET: u64 = 14695981039346656037;

    input.bytes().fold(OFFSET, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(PRIME)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv_hash() {
        let h1 = fnv_hash("test");
        let h2 = fnv_hash("test");
        assert_eq!(h1, h2, "same input should produce same hash");

        let h3 = fnv_hash("other");
        assert_ne!(h1, h3, "different input should produce different hash");
    }

    #[test]
    fn test_version_display() {
        use super::Version;

        let v = Version::new(1, 2, 3);
        assert_eq!(format!("{}", v), "1.2.3");

        let pre = Version::pre(1, 0, 0, "alpha.1");
        assert_eq!(format!("{}", pre), "1.0.0-alpha.1");
    }
}
