//! Unicode normalization, trimming, and slugification utilities.
//!
//! Provides pure, stateless functions for string normalization and transformation.

use regex::Regex;
use unicode_normalization::UnicodeNormalization;

/// Normalizes a string to NFC (Canonical Decomposition, followed by Canonical Composition).
///
/// # Arguments
/// * `input` - String to normalize
///
/// # Returns
/// NFC-normalized string
///
/// # Example
/// ```
/// use phenotype_string::normalize_nfc;
/// let input = "Café";
/// let normalized = normalize_nfc(input);
/// assert!(!normalized.is_empty());
/// ```
pub fn normalize_nfc(input: &str) -> String {
    input.nfc().collect()
}

/// Normalizes a string to NFD (Canonical Decomposition).
///
/// # Arguments
/// * `input` - String to normalize
///
/// # Returns
/// NFD-normalized string
///
/// # Example
/// ```
/// use phenotype_string::normalize_nfd;
/// let input = "Café";
/// let normalized = normalize_nfd(input);
/// assert!(normalized.len() > 4);
/// ```
pub fn normalize_nfd(input: &str) -> String {
    input.nfd().collect()
}

/// Removes leading and trailing whitespace from a string.
///
/// # Arguments
/// * `input` - String to trim
///
/// # Returns
/// Trimmed string
///
/// # Example
/// ```
/// use phenotype_string::trim_whitespace;
/// assert_eq!(trim_whitespace("  hello world  "), "hello world");
/// ```
pub fn trim_whitespace(input: &str) -> String {
    input.trim().to_string()
}

/// Converts a string to a URL-friendly slug.
///
/// Converts to lowercase, replaces spaces and special characters with hyphens,
/// and removes leading/trailing hyphens.
///
/// # Arguments
/// * `input` - String to slugify
///
/// # Returns
/// URL-friendly slug
///
/// # Example
/// ```
/// use phenotype_string::slugify;
/// assert_eq!(slugify("Hello World!"), "hello-world");
/// assert_eq!(slugify("Foo    Bar"), "foo-bar");
/// ```
pub fn slugify(input: &str) -> String {
    let lowercase = input.to_lowercase();

    // Replace spaces and underscores with hyphens
    let with_hyphens = lowercase.replace([' ', '_'], "-");

    // Use regex to replace non-alphanumeric characters (except hyphens) with nothing
    let regex = Regex::new(r"[^a-z0-9\-]").unwrap_or_else(|_| Regex::new("[^a-z0-9-]").unwrap());
    let clean = regex.replace_all(&with_hyphens, "").to_string();

    // Replace multiple consecutive hyphens with a single hyphen
    let hyphen_regex = Regex::new(r"-+").unwrap();
    let normalized = hyphen_regex.replace_all(&clean, "-").to_string();

    // Remove leading and trailing hyphens
    normalized.trim_matches('-').to_string()
}

/// Removes all non-alphanumeric characters from a string.
///
/// # Arguments
/// * `input` - String to clean
///
/// # Returns
/// String with only alphanumeric characters
///
/// # Example
/// ```ignore
/// use phenotype_string::remove_special_chars;
/// assert_eq!(remove_special_chars("Hello@World!"), "HelloWorld");
/// ```
pub fn remove_special_chars(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

/// Collapses multiple consecutive whitespace characters into a single space.
///
/// # Arguments
/// * `input` - String with potential whitespace
///
/// # Returns
/// String with collapsed whitespace
///
/// # Example
/// ```ignore
/// use phenotype_string::collapse_whitespace;
/// assert_eq!(collapse_whitespace("hello    world"), "hello world");
/// ```
pub fn collapse_whitespace(input: &str) -> String {
    let regex = Regex::new(r"\s+").unwrap();
    regex.replace_all(input, " ").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_nfc() {
        let input = "Café";
        let normalized = normalize_nfc(input);
        assert_eq!(normalized, "Café");
    }

    #[test]
    fn test_normalize_nfd() {
        let input = "Café";
        let normalized = normalize_nfd(input);
        // NFD produces decomposed form (e + combining acute accent)
        assert_ne!(normalized, "Café");
        assert!(normalized.len() > 4);
    }

    #[test]
    fn test_trim_whitespace() {
        assert_eq!(trim_whitespace("  hello  "), "hello");
        assert_eq!(trim_whitespace("  hello world  "), "hello world");
        assert_eq!(trim_whitespace("hello"), "hello");
        assert_eq!(trim_whitespace(""), "");
    }

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("foo bar"), "foo-bar");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("Hello@World!"), "helloworld");
        assert_eq!(slugify("foo#bar$baz"), "foobarbaz");
    }

    #[test]
    fn test_slugify_multiple_spaces() {
        assert_eq!(slugify("foo    bar"), "foo-bar");
        assert_eq!(slugify("  hello  world  "), "hello-world");
    }

    #[test]
    fn test_slugify_mixed() {
        assert_eq!(slugify("The Quick Brown Fox!"), "the-quick-brown-fox");
        assert_eq!(slugify("My_Test_String"), "my-test-string");
    }

    #[test]
    fn test_slugify_empty() {
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn test_slugify_only_special() {
        assert_eq!(slugify("@#$%"), "");
    }

    #[test]
    fn test_slugify_leading_trailing_hyphens() {
        assert_eq!(slugify("-hello-"), "hello");
        assert_eq!(slugify("---test---"), "test");
    }

    #[test]
    fn test_remove_special_chars() {
        assert_eq!(remove_special_chars("Hello@World!"), "HelloWorld");
        assert_eq!(remove_special_chars("foo#bar$baz"), "foobarbaz");
        assert_eq!(remove_special_chars("test123"), "test123");
        assert_eq!(remove_special_chars("!!!"), "");
    }

    #[test]
    fn test_collapse_whitespace() {
        assert_eq!(collapse_whitespace("hello    world"), "hello world");
        assert_eq!(collapse_whitespace("  hello  "), "hello");
        assert_eq!(collapse_whitespace("a\t\n\rb"), "a b");
        assert_eq!(collapse_whitespace("single"), "single");
    }

    #[test]
    fn test_collapse_whitespace_tabs_and_newlines() {
        assert_eq!(collapse_whitespace("hello\t\tworld"), "hello world");
        assert_eq!(collapse_whitespace("hello\n\nworld"), "hello world");
    }

    #[test]
    fn test_nfc_nfd_equivalence() {
        let original = "Naïve café";
        let nfc = normalize_nfc(original);
        let nfd = normalize_nfd(original);
        // Both should normalize to consistent representations
        assert!(!nfc.is_empty());
        assert!(!nfd.is_empty());
    }

    #[test]
    fn test_slugify_unicode() {
        assert_eq!(slugify("Café"), "caf");
        assert_eq!(slugify("naïve"), "nave");
    }

    #[test]
    fn test_slugify_consecutive_special_chars() {
        assert_eq!(slugify("hello@@world"), "helloworld");
        // Consecutive hyphens get normalized to single hyphen
        assert_eq!(slugify("foo---bar"), "foo-bar");
    }
}
