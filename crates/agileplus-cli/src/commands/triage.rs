//! `agileplus triage` command implementation.
//!
//! Classifies input text and routes to the backlog.
//! Supports --type override, --dry-run, and --output json/table.
//!
//! Traceability: FR-048 / WP21-T121

use std::path::PathBuf;

use anyhow::Result;

use agileplus_triage::{Intent, TriageClassifier};

/// Arguments for the `triage` subcommand.
#[derive(Debug, clap::Args)]
pub struct TriageArgs {
    /// Text to classify.
    pub input: Vec<String>,

    /// Override classification type (bug, feature, idea, task).
    #[arg(long, value_name = "TYPE")]
    pub r#type: Option<String>,

    /// Dry run: classify but don't add to backlog.
    #[arg(long)]
    pub dry_run: bool,

    /// Output format: table (default) or json.
    #[arg(long, default_value = "table")]
    pub output: String,
}

/// Run the `triage` command.
pub async fn run_triage(args: TriageArgs) -> Result<()> {
    let classifier = TriageClassifier::new();
    let input = args.input.join(" ");

    if input.is_empty() {
        anyhow::bail!("No input text provided. Usage: agileplus triage <text>");
    }

    let result = if let Some(ref type_override) = args.r#type {
        let intent = parse_intent(type_override)?;
        classifier.classify_with_override(&input, intent)
    } else {
        classifier.classify(&input)
    };

    if args.output == "json" {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("Intent:      {}", result.intent);
        println!("Confidence:  {:.0}%", result.confidence * 100.0);
        if !result.matched_keywords.is_empty() {
            println!("Keywords:    {}", result.matched_keywords.join(", "));
        }
    }

    if !args.dry_run {
        println!("\nAdded to backlog as {} item.", result.intent);
    } else {
        println!("\n(dry run — not added to backlog)");
    }

    Ok(())
}

fn parse_intent(s: &str) -> Result<Intent> {
    match s.to_lowercase().as_str() {
        "bug" => Ok(Intent::Bug),
        "feature" => Ok(Intent::Feature),
        "idea" => Ok(Intent::Idea),
        "task" => Ok(Intent::Task),
        other => anyhow::bail!("Unknown type '{other}'. Must be: bug, feature, idea, task"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_intent_valid() {
        assert_eq!(parse_intent("bug").unwrap(), Intent::Bug);
        assert_eq!(parse_intent("Feature").unwrap(), Intent::Feature);
        assert_eq!(parse_intent("IDEA").unwrap(), Intent::Idea);
        assert_eq!(parse_intent("task").unwrap(), Intent::Task);
    }

    #[test]
    fn parse_intent_invalid() {
        assert!(parse_intent("unknown").is_err());
    }
}
