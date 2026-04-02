//! String normalization utilities for phenotype-string.

use thiserror::Error;

/// Errors that can occur during normalization operations.
#[derive(Debug, Error)]
pub enum NormalizationError {
    #[error("Invalid Unicode sequence: {0}")]
    InvalidUnicode(String),
}

/// Unicode normalization forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationForm {
    Composed,
    Decomposed,
    CompatibilityComposed,
    CompatibilityDecomposed,
}

/// Normalizes a string to the specified Unicode normalization form.
#[inline]
#[must_use]
pub fn normalize(s: &str, form: NormalizationForm) -> String {
    use unicode_normalization::UnicodeNormalization;
    match form {
        NormalizationForm::Composed => s.nfc().collect(),
        NormalizationForm::Decomposed => s.nfd().collect(),
        NormalizationForm::CompatibilityComposed => s.nfkc().collect(),
        NormalizationForm::CompatibilityDecomposed => s.nfkd().collect(),
    }
}

/// Normalizes a string to NFC.
#[inline]
#[must_use]
pub fn normalize_nfc(s: &str) -> String {
    normalize(s, NormalizationForm::Composed)
}

/// Case folding utilities.
pub mod case_folding {
    /// Converts a string to case-folded form.
    #[inline]
    #[must_use]
    pub fn fold(s: &str) -> String {
        s.to_lowercase()
    }

    /// Converts a string to uppercase.
    #[inline]
    #[must_use]
    pub fn uppercase(s: &str) -> String {
        s.to_uppercase()
    }

    /// Converts a string to lowercase.
    #[inline]
    #[must_use]
    pub fn lowercase(s: &str) -> String {
        s.to_lowercase()
    }

    /// Converts the first character of a string to uppercase.
    #[inline]
    #[must_use]
    pub fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => {
                let upper: String = c.to_uppercase().collect();
                let rest: String = chars.collect();
                format!("{}{}", upper, rest)
            }
        }
    }

    /// Converts the first character of each word to uppercase.
    #[inline]
    #[must_use]
    pub fn titlecase(s: &str) -> String {
        s.split_whitespace().map(capitalize).collect::<Vec<_>>().join(" ")
    }
}

/// Width normalization utilities.
pub mod width {
    /// Normalizes string width (fullwidth to halfwidth).
    #[inline]
    #[must_use]
    pub fn to_halfwidth(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '\u{FF10}'..='\u{FF19}' => {
                    char::from((u32::from(c) - 0xFF10) as u8 + b'0')
                }
                '\u{FF21}'..='\u{FF3A}' => {
                    char::from((u32::from(c) - 0xFF21) as u8 + b'A')
                }
                '\u{FF41}'..='\u{FF5A}' => {
                    char::from((u32::from(c) - 0xFF41) as u8 + b'a')
                }
                _ => c,
            })
            .collect()
    }

    /// Converts ASCII characters to their fullwidth variants.
    #[inline]
    #[must_use]
    pub fn to_fullwidth(s: &str) -> String {
        s.chars()
            .map(|c: char| {
                let cu = u32::from(c);
                match cu {
                    0x0030..=0x0039 => char::from_u32(cu - 0x0030 + 0xFF10).unwrap_or(c),
                    0x0041..=0x005A => char::from_u32(cu - 0x0041 + 0xFF21).unwrap_or(c),
                    0x0061..=0x007A => char::from_u32(cu - 0x0061 + 0xFF41).unwrap_or(c),
                    _ => c,
                }
            })
            .collect()
    }
}

/// Diacritic removal utilities.
pub mod diacritics {
    use unicode_normalization::char::is_combining_mark;

    /// Removes combining diacritical marks from a string.
    #[inline]
    #[must_use]
    pub fn remove(s: &str) -> String {
        s.chars().filter(|c| !is_combining_mark(*c)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_nfc() {
        let input = "cafe";
        let output = normalize_nfc(input);
        assert_eq!(output, "cafe");
    }

    #[test]
    fn test_case_fold() {
        assert_eq!(case_folding::fold("HELLO"), "hello");
        assert_eq!(case_folding::uppercase("hello"), "HELLO");
        assert_eq!(case_folding::capitalize("hello"), "Hello");
        assert_eq!(case_folding::titlecase("hello world"), "Hello World");
    }

    #[test]
    fn test_width_normalization() {
        let fullwidth = "\u{FF10}\u{FF11}\u{FF12}";
        assert_eq!(width::to_halfwidth(fullwidth), "012");
    }

    #[test]
    fn test_normalization_error_display() {
        let err = NormalizationError::InvalidUnicode("test".to_string());
        assert_eq!(err.to_string(), "Invalid Unicode sequence: test");
    }
}
