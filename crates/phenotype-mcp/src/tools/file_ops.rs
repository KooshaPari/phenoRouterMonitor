//! File operations tool.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Result of a file operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationResult {
    pub path: String,
    pub success: bool,
    pub message: String,
    pub bytes_processed: u64,
}

/// File operations tool for read/write/delete.
#[derive(Debug, Clone)]
pub struct FileOperator;

impl FileOperator {
    /// Create a new file operator.
    pub fn new() -> Self {
        Self
    }

    /// Read a file and return its contents.
    pub fn read_file(&self, path: &str) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))
    }

    /// Write content to a file.
    pub fn write_file(&self, path: &str, content: &str) -> Result<FileOperationResult, String> {
        std::fs::write(path, content).map_err(|e| format!("Failed to write {}: {}", path, e))?;

        Ok(FileOperationResult {
            path: path.to_string(),
            success: true,
            message: format!("Wrote {} bytes", content.len()),
            bytes_processed: content.len() as u64,
        })
    }

    /// Check if a file exists.
    pub fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    /// Get file metadata.
    pub fn get_metadata(&self, path: &str) -> Result<(u64, String), String> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| format!("Failed to get metadata for {}: {}", path, e))?;

        let is_file = if metadata.is_file() {
            "file"
        } else {
            "directory"
        };
        Ok((metadata.len(), is_file.to_string()))
    }

    /// Delete a file.
    pub fn delete_file(&self, path: &str) -> Result<FileOperationResult, String> {
        std::fs::remove_file(path).map_err(|e| format!("Failed to delete {}: {}", path, e))?;

        Ok(FileOperationResult {
            path: path.to_string(),
            success: true,
            message: "File deleted".to_string(),
            bytes_processed: 0,
        })
    }
}

impl Default for FileOperator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn file_operator_exists() {
        let op = FileOperator::new();
        // Cargo.toml should exist in repo root
        assert!(op.exists("Cargo.toml"));
        assert!(!op.exists("/nonexistent/path/to/file"));
    }

    #[test]
    fn file_operator_metadata() {
        let op = FileOperator::new();
        let (size, ftype) = op.get_metadata("Cargo.toml").unwrap();
        assert!(size > 0);
        assert_eq!(ftype, "file");
    }

    #[test]
    fn file_operator_write_and_read() {
        let op = FileOperator::new();
        let test_file = "/tmp/test_phenotype_mcp.txt";
        let content = "test content";

        // Write
        let result = op.write_file(test_file, content).unwrap();
        assert!(result.success);

        // Read
        let read_content = op.read_file(test_file).unwrap();
        assert_eq!(read_content, content);

        // Cleanup
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn file_operator_delete() {
        let op = FileOperator::new();
        let test_file = "/tmp/test_delete_phenotype.txt";
        fs::write(test_file, "delete me").ok();

        assert!(op.exists(test_file));
        op.delete_file(test_file).unwrap();
        assert!(!op.exists(test_file));
    }
}
