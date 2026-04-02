//! String compression utilities.
//!
//! Provides compression algorithms for string data.

use crate::{Error, Result};

/// Compress a string using gzip.
pub fn gzip_compress(input: &str) -> Result<Vec<u8>> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(input.as_bytes())
        .map_err(|e| Error::Compression(e.to_string()))?;
    encoder
        .finish()
        .map_err(|e| Error::Compression(e.to_string()))
}

/// Decompress gzip-compressed data to a string.
pub fn gzip_decompress(input: &[u8]) -> Result<String> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let mut decoder = GzDecoder::new(input);
    let mut result = String::new();
    decoder
        .read_to_string(&mut result)
        .map_err(|e| Error::Decompression(e.to_string()))?;
    Ok(result)
}

/// Compress a string using zlib.
pub fn zlib_compress(input: &str) -> Result<Vec<u8>> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(input.as_bytes())
        .map_err(|e| Error::Compression(e.to_string()))?;
    encoder
        .finish()
        .map_err(|e| Error::Compression(e.to_string()))
}

/// Decompress zlib-compressed data to a string.
pub fn zlib_decompress(input: &[u8]) -> Result<String> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;

    let mut decoder = ZlibDecoder::new(input);
    let mut result = String::new();
    decoder
        .read_to_string(&mut result)
        .map_err(|e| Error::Decompression(e.to_string()))?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gzip_roundtrip() {
        let input = "Hello, World!";
        let compressed = gzip_compress(input).unwrap();
        let decompressed = gzip_decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }

    #[test]
    fn test_zlib_roundtrip() {
        let input = "Hello, World!";
        let compressed = zlib_compress(input).unwrap();
        let decompressed = zlib_decompress(&compressed).unwrap();
        assert_eq!(decompressed, input);
    }
}
