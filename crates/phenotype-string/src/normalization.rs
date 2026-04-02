//! String normalization utilities.
//!
//! Provides normalization for Unicode strings, whitespace handling,
//! and case normalization.

/// Normalize Unicode string to NFC form.
pub fn unicode_nfc(input: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    input.nfc().collect()
}

/// Normalize Unicode string to NFD form.
pub fn unicode_nfd(input: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    input.nfd().collect()
}

/// Normalize Unicode string to NFKC form.
pub fn unicode_nfkc(input: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    input.nfkc().collect()
}

/// Normalize Unicode string to NFKD form.
pub fn unicode_nfkd(input: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    input.nfkd().collect()
}

/// Normalize whitespace (trim leading/trailing, collapse multiple spaces).
pub fn whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Normalize to lowercase.
pub fn lowercase(input: &str) -> String {
    input.to_lowercase()
}

/// Normalize to uppercase.
pub fn uppercase(input: &str) -> String {
    input.to_uppercase()
}

/// Full normalization: Unicode NFC + whitespace + lowercase.
pub fn full(input: &str) -> String {
    lowercase(&whitespace(&unicode_nfc(input)))
}

/// Check if two strings are equal after full normalization.
pub fn eq_normalized(a: &str, b: &str) -> bool {
    full(a) == full(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_nfc() {
        // Combining characters should be combined
        let input = "e\u{0301}"; // e + combining acute
        let normalized = unicode_nfc(input);
        assert_eq!(normalized, "\u{00e9}"); // precomposed é
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(whitespace("  hello   world  "), "hello world");
        assert_eq!(whitespace("hello\tworld"), "hello world");
    }

    #[test]
    fn test_full_normalization() {
        assert_eq!(full("  HELLO   WORLD  "), "hello world");
    }

    #[test]
    fn test_eq_normalized() {
        assert!(eq_normalized("Hello", "hello"));
        assert!(eq_normalized("  Hello  ", "hello"));
        assert!(!eq_normalized("Hello", "world"));
    }
}
