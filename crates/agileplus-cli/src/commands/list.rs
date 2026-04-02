//! List features from storage (read-only).

use std::str::FromStr;

use anyhow::{Context, Result, anyhow};
use clap::Args;

use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::ports::StoragePort;

#[derive(Debug, Args)]
pub struct ListArgs {
    /// Filter by feature state (created, specified, researched, planned,
    /// implementing, validated, shipped, retrospected).
    #[arg(long)]
    pub state: Option<String>,
}

pub async fn run<S: StoragePort>(args: ListArgs, storage: &S) -> Result<()> {
    let features = if let Some(ref s) = args.state {
        let state = FeatureState::from_str(s).map_err(|e| anyhow!("{e}"))?;
        storage
            .list_features_by_state(state)
            .await
            .context("listing features by state")?
    } else {
        storage
            .list_all_features()
            .await
            .context("listing features")?
    };

    if features.is_empty() {
        println!("No features found.");
        return Ok(());
    }

    println!(
        "{:<8}  {:<36}  {:<14}  {}",
        "ID", "SLUG", "STATE", "TITLE"
    );
    println!("{}", "-".repeat(100));

    for f in features {
        println!(
            "{:<8}  {:<36}  {:<14}  {}",
            f.id,
            truncate_cell(&f.slug, 36),
            f.state.to_string(),
            f.friendly_name
        );
    }

    Ok(())
}

fn truncate_cell(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        return s.to_string();
    }
    let shortened: String = s.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{shortened}…")
}
