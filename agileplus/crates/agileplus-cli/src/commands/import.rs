//! Import command family for populating AgilePlus from a bundle manifest.

use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use agileplus_domain::ports::{StoragePort, VcsPort};
use agileplus_import::{ImportBundle, import_bundle};

/// Top-level import command.
#[derive(Debug, Args)]
pub struct ImportArgs {
    #[command(subcommand)]
    pub command: ImportCommand,
}

#[derive(Debug, Subcommand)]
pub enum ImportCommand {
    /// Import a JSON bundle from a file or stdin.
    Bundle(BundleArgs),
}

#[derive(Debug, Args)]
pub struct BundleArgs {
    /// Read the bundle from this file. Omit to read from stdin.
    #[arg(long)]
    pub file: Option<PathBuf>,
}

pub async fn run_import<S, V>(args: ImportArgs, storage: &S, vcs: &V) -> Result<()>
where
    S: StoragePort,
    V: VcsPort,
{
    match args.command {
        ImportCommand::Bundle(bundle) => run_bundle(bundle, storage, vcs).await,
    }
}

async fn run_bundle<S, V>(args: BundleArgs, storage: &S, vcs: &V) -> Result<()>
where
    S: StoragePort,
    V: VcsPort,
{
    let raw = if let Some(path) = args.file {
        tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("reading bundle from {}", path.display()))?
    } else {
        let mut raw = String::new();
        std::io::stdin()
            .read_to_string(&mut raw)
            .context("reading bundle from stdin")?;
        raw
    };

    let bundle: ImportBundle = parse_bundle(&raw)?;
    let report = import_bundle(bundle, storage, vcs).await?;

    println!("Import completed.");
    println!("  Modules created:   {}", report.modules_created);
    println!("  Modules updated:   {}", report.modules_updated);
    println!("  Features created:  {}", report.features_created);
    println!("  Features updated:  {}", report.features_updated);
    println!("  Cycles created:    {}", report.cycles_created);
    println!("  Cycles updated:    {}", report.cycles_updated);
    println!("  WPs created:       {}", report.work_packages_created);
    println!("  WPs updated:       {}", report.work_packages_updated);
    println!(
        "  Links created:     {}",
        report.module_links_created + report.cycle_links_created
    );
    println!("  Artifacts written: {}", report.artifacts_written);
    println!("  Audit entries:     {}", report.audits_written);
    Ok(())
}

fn parse_bundle(raw: &str) -> Result<ImportBundle> {
    serde_json::from_str(raw)
        .or_else(|json_err| serde_yaml::from_str(raw).map_err(|yaml_err| (json_err, yaml_err)))
        .map_err(|(json_err, yaml_err)| {
            anyhow::anyhow!(
                "parsing import bundle as JSON or YAML failed: JSON error: {}; YAML error: {}",
                json_err,
                yaml_err
            )
        })
}
