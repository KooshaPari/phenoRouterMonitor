use clap::Parser;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "phenotype-path")]
#[command(about = "Fast PATH resolution with shim filtering")]
struct Cli {
    /// Binary name(s) to resolve
    #[arg(value_name = "NAME")]
    names: Vec<String>,
    
    /// Output format
    #[arg(short, long, value_enum, default_value = "path")]
    format: OutputFormat,
}

#[derive(Clone, Debug, Default, Serialize, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Path,
    Json,
}

fn main() {
    let cli = Cli::parse();
    
    if cli.names.is_empty() {
        eprintln!("Usage: phenotype-path <NAME> [--format <FORMAT>]");
        std::process::exit(1);
    }
    
    // Create owned strings for names
    let names: Vec<String> = cli.names.iter().map(|s| s.to_string()).collect();
    
    // Get references for resolve_many
    let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    
    let resolver = PathResolver::with_skip_dirs(ShimFilter::default_skip_dirs());
    let results = resolver.resolve_many(&name_refs);

    match cli.format {
        OutputFormat::Path => {
            let mut found_any = false;
            for name in &names {
                if let Some(Some(path)) = results.get(name.as_str()) {
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
            println!("{}", serde_json::to_string_pretty(&results).unwrap());
        }
    }
}
