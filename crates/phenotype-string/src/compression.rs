//! String compression and decompression utilities using zstd.
//!
//! Provides pure, stateless functions for compressing and decompressing strings.

use crate::{Error, Result};

/// Compresses a string using zstd compression.
///
/// # Arguments
/// * `input` - String to compress
/// * `level` - Compression level (1-22, where 22 is maximum compression)
///
/// # Returns
/// Compressed bytes as a Vec<u8>
///
/// # Example
/// ```
/// use phenotype_string::compress;
/// let data = "Hello, World!".as_bytes();
/// let compressed = compress(data, 3).expect("compression failed");
/// assert!(compressed.len() < data.len() || data.len() < 50);
/// ```
pub fn compress(input: &[u8], level: i32) -> Result<Vec<u8>> {
    let level = if level < 1 {
        1
    } else if level > 22 {
        22
    } else {
        level
    };

    zstd::encode_all(input, level)
        .map_err(|e| Error::CompressionError(e.to_string()))
}

/// Decompresses a zstd-compressed byte slice.
///
/// # Arguments
/// * `compressed` - Compressed bytes
///
/// # Returns
/// Decompressed bytes as a Vec<u8>
///
/// # Example
/// ```
/// use phenotype_string::{compress, decompress};
/// let data = "Hello, World!".as_bytes();
/// let compressed = compress(data, 3).expect("compression failed");
/// let decompressed = decompress(&compressed).expect("decompression failed");
/// assert_eq!(data, &decompressed[..]);
/// ```
pub fn decompress(compressed: &[u8]) -> Result<Vec<u8>> {
    zstd::decode_all(compressed)
        .map_err(|e| Error::DecompressionError(e.to_string()))
}

/// Compresses a string with default compression level (3).
///
/// # Arguments
/// * `input` - String to compress
///
/// # Returns
/// Compressed bytes as a Vec<u8>
pub fn compress_string(input: &str) -> Result<Vec<u8>> {
    compress(input.as_bytes(), 3)
}

/// Decompresses bytes and converts to a UTF-8 string.
///
/// # Arguments
/// * `compressed` - Compressed bytes
///
/// # Returns
/// Decompressed string
pub fn decompress_string(compressed: &[u8]) -> Result<String> {
    let bytes = decompress(compressed)?;
    String::from_utf8(bytes).map_err(|e| Error::Invalid(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_roundtrip() {
        let original = b"Hello, World! This is a test string.";
        let compressed = compress(original, 3).expect("compression failed");
        let decompressed = decompress(&compressed).expect("decompression failed");
        assert_eq!(original, &decompressed[..]);
    }

    #[test]
    fn test_compress_string() {
        let original = "Hello, World!";
        let compressed = compress_string(original).expect("compression failed");
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_decompress_string() {
        let original = "Hello, World!";
        let compressed = compress_string(original).expect("compression failed");
        let decompressed = decompress_string(&compressed).expect("decompression failed");
        assert_eq!(original, &decompressed);
    }

    #[test]
    fn test_compress_with_invalid_level_too_low() {
        let data = b"test";
        let compressed = compress(data, -10).expect("compression failed");
        let decompressed = decompress(&compressed).expect("decompression failed");
        assert_eq!(data, &decompressed[..]);
    }

    #[test]
    fn test_compress_with_invalid_level_too_high() {
        let data = b"test";
        let compressed = compress(data, 100).expect("compression failed");
        let decompressed = decompress(&compressed).expect("decompression failed");
        assert_eq!(data, &decompressed[..]);
    }

    #[test]
    fn test_compress_empty_string() {
        let data = b"";
        let compressed = compress(data, 3).expect("compression failed");
        let decompressed = decompress(&compressed).expect("decompression failed");
        assert_eq!(data, &decompressed[..]);
    }

    #[test]
    fn test_compress_large_string() {
        let original = "A".repeat(10000);
        let compressed = compress_string(&original).expect("compression failed");
        let decompressed = decompress_string(&compressed).expect("decompression failed");
        assert_eq!(&original, &decompressed);
    }

    #[test]
    fn test_decompress_invalid_data() {
        let invalid = b"not compressed data";
        let result = decompress(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_compress_various_levels() {
        let data = b"The quick brown fox jumps over the lazy dog";
        for level in 1..=22 {
            let compressed = compress(data, level).expect("compression failed");
            let decompressed = decompress(&compressed).expect("decompression failed");
            assert_eq!(data, &decompressed[..]);
        }
    }
}
