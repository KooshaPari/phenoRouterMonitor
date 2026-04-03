use clap::{Parser, Subcommand};
use phenotype_compliance_scanner::ComplianceScanner;
use phenotype_project_registry::{Project, discover_projects};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pheno-guard")]
#[command(about = "Phenotype repository governance and compliance enforcer", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan repositories for compliance and governance
    Scan {
        /// Directory containing repositories to scan
        #[arg(long, default_value = ".")]
        repos_dir: PathBuf,
        
        /// Output format (json, table)
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// List discovered projects in the workspace
    List {
        /// Root directory to search for projects
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { repos_dir, format } => {
            let scanner = ComplianceScanner::new();
            let findings = scanner.scan(repos_dir.to_str().unwrap()).await?;
            
            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&findings)?);
            } else {
                if findings.is_empty() {
                    println!("✅ No compliance issues found.");
                } else {
                    println!("❌ Found {} compliance issues:", findings.len());
                    for finding in findings {
                        println!("  - [{}] {}: {}", finding.rule_id, finding.file_path, finding.message);
                    }
                }
            }
        }
        Commands::List { root } => {
            let projects = discover_projects(&root);
            println!("Discovered {} projects:", projects.len());
            for project in projects {
                println!("  - {} ({:?}) at {}", project.name, project.project_type, project.path.display());
            }
        }
    }

    Ok(())
}
