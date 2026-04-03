//! BKM-??: thegent-fs CLI binary.
//!
//! Provides command-line file operations:
//! - copy <src> <dst> [--preserve-metadata]
//! - copy-tree <src> <dst> [--ignore PATTERN]
//! - move <src> <dst>
//! - remove <path> [--recursive]
//! - size <path>
//! - glob <pattern>
//! - ls <dir>

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use thegent_fs::{
    copy_file, copy_tree, ensure_dir, get_size, glob_files, list_dir, move_path, remove_path,
};

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(
    name = "thegent-fs",
    about = "BKM-??: High-performance file operations for thegent",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Copy a file
    Copy {
        /// Source file path
        src: PathBuf,
        /// Destination file path
        dst: PathBuf,
        /// Preserve file metadata (permissions, timestamps)
        #[arg(long, short)]
        preserve_metadata: bool,
    },
    /// Copy a directory tree
    CopyTree {
        /// Source directory path
        src: PathBuf,
        /// Destination directory path
        dst: PathBuf,
        /// Patterns to ignore (can be specified multiple times)
        #[arg(long)]
        ignore: Vec<String>,
    },
    /// Move a file or directory
    Move {
        /// Source path
        src: PathBuf,
        /// Destination path
        dst: PathBuf,
    },
    /// Remove a file or directory
    Remove {
        /// Path to remove
        path: PathBuf,
        /// Remove recursively (for directories)
        #[arg(long, short)]
        recursive: bool,
    },
    /// Get size of file or directory
    Size {
        /// Path to measure
        path: PathBuf,
    },
    /// Find files matching glob pattern
    Glob {
        /// Glob pattern (e.g., "src/**/*.rs")
        pattern: String,
    },
    /// List directory contents
    Ls {
        /// Directory path
        path: PathBuf,
    },
    /// Ensure directory exists
    Mkdir {
        /// Directory path
        path: PathBuf,
        /// Directory permissions (octal)
        #[arg(long, default_value_t = 0o755)]
        mode: u32,
    },
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Copy {
            src,
            dst,
            preserve_metadata,
        } => {
            let bytes = copy_file(&src, &dst, preserve_metadata)?;
            println!("Copied {} bytes", bytes);
        }
        Command::CopyTree { src, dst, ignore } => {
            let ignore_refs: Vec<&str> = ignore.iter().map(|s| s.as_str()).collect();
            let bytes = copy_tree(
                &src,
                &dst,
                if ignore_refs.is_empty() {
                    None
                } else {
                    Some(&ignore_refs)
                },
            )?;
            println!("Copied {} bytes", bytes);
        }
        Command::Move { src, dst } => {
            move_path(&src, &dst)?;
            println!("Moved {:?} to {:?}", src, dst);
        }
        Command::Remove { path, recursive } => {
            remove_path(&path, recursive)?;
            println!("Removed {:?}", path);
        }
        Command::Size { path } => {
            let size = get_size(&path)?;
            println!("{}", size);
        }
        Command::Glob { pattern } => {
            let matches = glob_files(&pattern)?;
            for m in matches {
                println!("{}", m.display());
            }
        }
        Command::Ls { path } => {
            let entries = list_dir(&path)?;
            for e in entries {
                println!("{}", e.display());
            }
        }
        Command::Mkdir { path, mode } => {
            ensure_dir(&path, mode)?;
            println!("Created directory: {:?}", path);
        }
    }
    Ok(())
}
