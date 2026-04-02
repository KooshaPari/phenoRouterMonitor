//! String compression utilities for phenotype-string.
//!
//! This module provides compression utilities for reducing string size
//! while preserving data integrity for phenotype ecosystem use cases.

use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during string compression operations.
#[derive(Debug, Error)]
pub enum CompressionError {
    #[error("Compression failed: {0}")]
    CompressionFailed(String),

    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    #[error("Invalid compression data: {0}")]
    InvalidData(String),
}

/// Result type alias for compression operations.
pub type CompressionResult<T> = Result<T, CompressionError>;

/// Dictionary-based compression for repeated string patterns.
///
/// Uses a simple dictionary encoding scheme optimized for
/// code-like strings commonly found in configuration and DSL contexts.
#[derive(Debug, Clone)]
pub struct DictionaryCompressor {
    dictionary: HashMap<String, usize>,
    next_id: usize,
}

impl Default for DictionaryCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl DictionaryCompressor {
    /// Creates a new dictionary compressor with default capacity.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(256)
    }

    /// Creates a new dictionary compressor with specified initial capacity.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            dictionary: HashMap::with_capacity(capacity),
            next_id: 0,
        }
    }

    /// Adds a string to the dictionary and returns its ID.
    #[inline]
    pub fn add(&mut self, s: &str) -> usize {
        if let Some(&id) = self.dictionary.get(s) {
            return id;
        }
        let id = self.next_id;
        self.dictionary.insert(s.to_string(), id);
        self.next_id += 1;
        id
    }

    /// Gets the ID for a string if it exists in the dictionary.
    #[inline]
    #[must_use]
    pub fn get(&self, s: &str) -> Option<usize> {
        self.dictionary.get(s).copied()
    }

    /// Returns the number of entries in the dictionary.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.dictionary.len()
    }

    /// Returns true if the dictionary is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dictionary.is_empty()
    }
}

/// Whitespace compression utilities.
pub mod whitespace {
    /// Compresses consecutive whitespace to single spaces.
    #[inline]
    #[must_use]
    pub fn compress(s: &str) -> String {
        compress_impl(s.trim(), false)
    }

    /// Compresses whitespace and trims leading/trailing whitespace.
    #[inline]
    #[must_use]
    pub fn compress_trim(s: &str) -> String {
        compress_impl(s.trim(), false)
    }

    fn compress_impl(s: &str, _trim: bool) -> String {
        let mut result = String::with_capacity(s.len());
        let mut last_was_space = false;

        for c in s.chars() {
            if c.is_whitespace() {
                if !last_was_space {
                    result.push(' ');
                    last_was_space = true;
                }
            } else {
                result.push(c);
                last_was_space = false;
            }
        }

        result
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_compress_basic() {
            assert_eq!(compress("hello    world"), "hello world");
        }

        #[test]
        fn test_compress_mixed() {
            assert_eq!(compress("foo\t\n  bar"), "foo bar");
        }

        #[test]
        fn test_compress_trim() {
            assert_eq!(compress_trim("  hello world  "), "hello world");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_basic() {
        let mut dict = DictionaryCompressor::new();
        assert_eq!(dict.add("hello"), 0);
        assert_eq!(dict.add("world"), 1);
        assert_eq!(dict.add("hello"), 0);
        assert_eq!(dict.len(), 2);
    }

    #[test]
    fn test_compression_error_display() {
        let err = CompressionError::CompressionFailed("test".to_string());
        assert_eq!(err.to_string(), "Compression failed: test");
    }
}
