use clap::{Parser, ValueEnum};
use std::collections::HashMap;
use thegent_path_resolve::PathResolver;

#[derive(Parser)]
#[command(name = "thegent-path-resolve")]
#[command(about = "Fast PATH resolution", long_about = None)]
#[command(version)]
struct Cli {
    /// Binary name to resolve
    name: String,

    /// Additional binary names (resolves multiple at once)
    #[arg(short, long)]
    additional: Vec<String>,

    /// Directories to skip (colon-separated on Unix, semicolon on Windows)
    #[arg(short, long)]
    skip: Option<String>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "path")]
    format: OutputFormat,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Path,
    Json,
}

fn main() {
    let cli = Cli::parse();

    let skip_dirs: Vec<String> = cli
        .skip
        .map(|s| {
            #[cfg(unix)]
            return s.split(':').map(String::from).collect();
            #[cfg(windows)]
            return s.split(';').map(String::from).collect();
        })
        .unwrap_or_default();

    let resolver = if skip_dirs.is_empty() {
        PathResolver::new()
    } else {
        PathResolver::with_skip_dirs(skip_dirs)
    };

    // Resolve single or multiple binaries
    let mut names = vec![cli.name.as_str()];
    names.extend(cli.additional.iter().map(|s| s.as_str()));

    let results = resolver.resolve_many(&names);

    match cli.format {
        OutputFormat::Path => {
            let mut found_any = false;
            for name in names {
                if let Some(path) = results.get(name).and_then(|r| r.as_ref()) {
                    println!("{}", path);
                    found_any = true;
                }
            }
            if !found_any {
                eprintln!("None of the specified binaries found in PATH");
                std::process::exit(1);
            }
        }
        OutputFormat::Json => {
            let json: HashMap<String, Option<String>> = results
                .into_iter()
                .map(|(k, v)| (k, v))
                .collect();
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }
    }
}
