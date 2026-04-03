//! High-performance file operations

use std::path::{Path, PathBuf};

/// Copy a file
pub fn copy_file(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<()> {
    std::fs::copy(src, dst)?;
    Ok(())
}

/// Move a file
pub fn move_file(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<()> {
    std::fs::rename(src, dst)?;
    Ok(())
}

/// Glob pattern matching
pub fn glob(pattern: &str) -> anyhow::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in glob::glob(pattern)? {
        if let Ok(path) = entry {
            paths.push(path);
        }
    }
    Ok(paths)
}

/// Walk directory
pub fn walk_dir(path: impl AsRef<Path>) -> anyhow::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in walkdir::WalkDir::new(path) {
        if let Ok(entry) = entry {
            paths.push(entry.path().to_path_buf());
        }
    }
    Ok(paths)
}
