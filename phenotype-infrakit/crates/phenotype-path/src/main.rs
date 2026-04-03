use std::env;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                println!("phenotype-path: Cross-platform path utilities");
                println!("Usage: phenotype-path [OPTIONS]");
                println!("\nOptions:");
                println!("  -h, --help    Show this help message");
                println!("  -c, --current Print current working directory");
                println!("  --home        Print home directory");
                return;
            }
            "--current" | "-c" => {
                match std::env::current_dir() {
                    Ok(path) => println!("{}", path.display()),
                    Err(e) => eprintln!("Error: {}", e),
                }
                return;
            }
            "--home" => {
                match dirs::home_dir() {
                    Some(path) => println!("{}", path.display()),
                    None => eprintln!("Error: Could not determine home directory"),
                }
                return;
            }
            _ => {}
        }
    }
    
    // Default: print current directory
    match std::env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        Err(e) => eprintln!("Error: {}", e),
    }
}
