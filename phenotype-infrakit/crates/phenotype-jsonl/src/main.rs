//! BKM-10: thegent-jsonl CLI binary.
//!
//! Provides subcommands for counting, streaming, filtering, and sampling JSONL
//! files from the command line.  Each subcommand writes to stdout so output can
//! be piped to other tools.
//!
//! # Usage
//!
//! ```text
//! thegent-jsonl count <file>
//! thegent-jsonl stream <file>
//! thegent-jsonl filter <file> --key KEY --value VALUE
//! thegent-jsonl sample <file> --n N
//! ```

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use thegent_jsonl::{count_file, filter_file, parse_file, sample_file};

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(
    name = "thegent-jsonl",
    about = "BKM-10: Streaming JSONL parser for thegent audit logs",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Count the number of records in a JSONL file
    Count {
        /// Path to the JSONL file
        file: PathBuf,
    },
    /// Stream records from a JSONL file, one JSON object per line to stdout
    Stream {
        /// Path to the JSONL file
        file: PathBuf,
    },
    /// Filter records in a JSONL file by key=value
    Filter {
        /// Path to the JSONL file
        file: PathBuf,
        /// Key to filter on (top-level JSON field name)
        #[arg(long)]
        key: String,
        /// Value to match (string comparison)
        #[arg(long)]
        value: String,
    },
    /// Sample N records from the beginning of a JSONL file
    Sample {
        /// Path to the JSONL file
        file: PathBuf,
        /// Number of records to return
        #[arg(long, default_value_t = 10)]
        n: usize,
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
        Command::Count { file } => {
            let n = count_file(&file)?;
            println!("{n}");
        }
        Command::Stream { file } => {
            for record in parse_file(&file)? {
                match record {
                    Ok(v) => println!("{v}"),
                    Err(e) => eprintln!("warning: {e}"),
                }
            }
        }
        Command::Filter { file, key, value } => {
            for record in filter_file(&file, &key, &value)? {
                match record {
                    Ok(v) => println!("{v}"),
                    Err(e) => eprintln!("warning: {e}"),
                }
            }
        }
        Command::Sample { file, n } => {
            for record in sample_file(&file, n)? {
                match record {
                    Ok(v) => println!("{v}"),
                    Err(e) => eprintln!("warning: {e}"),
                }
            }
        }
    }
    Ok(())
}
