//! `agileplus queue` command implementation.
//!
//! Manages the triage backlog: add, list, show, pop items.
//!
//! Traceability: FR-049 / WP21-T122

use anyhow::Result;

use agileplus_triage::{BacklogItem, BacklogPriority, BacklogStatus, Intent, TriageClassifier};

/// Arguments for the `queue` subcommand.
#[derive(Debug, clap::Args)]
pub struct QueueArgs {
    #[command(subcommand)]
    pub action: QueueAction,
}

#[derive(Debug, clap::Subcommand)]
pub enum QueueAction {
    /// Add an item to the backlog queue.
    Add {
        /// Item title.
        #[arg(long)]
        title: String,
        /// Item description.
        #[arg(long, default_value = "")]
        description: String,
        /// Item type (bug, feature, idea, task). Auto-classified if omitted.
        #[arg(long)]
        r#type: Option<String>,
    },
    /// List items in the backlog.
    List {
        /// Filter by status (new, triaged, in_progress, done, dismissed).
        #[arg(long)]
        status: Option<String>,
        /// Filter by type (bug, feature, idea, task).
        #[arg(long)]
        r#type: Option<String>,
        /// Output format: table (default) or json.
        #[arg(long, default_value = "table")]
        output: String,
    },
    /// Show details for a specific backlog item.
    Show {
        /// Item ID.
        id: i64,
    },
    /// Pop the next highest-priority item from the queue.
    Pop,
}

/// Run the `queue` command.
pub async fn run_queue(args: QueueArgs) -> Result<()> {
    match args.action {
        QueueAction::Add {
            title,
            description,
            r#type,
        } => {
            let intent = if let Some(ref t) = r#type {
                parse_intent(t)?
            } else {
                let classifier = TriageClassifier::new();
                classifier.classify(&title).intent
            };

            let item = BacklogItem::from_triage(
                title.clone(),
                description,
                intent,
                "cli".to_string(),
            );

            println!("Added to queue: \"{}\" ({})", title, intent);
            println!("Priority: {}", item.priority);
        }
        QueueAction::List {
            status,
            r#type,
            output,
        } => {
            // In full implementation, this reads from SQLite.
            // For now, print placeholder.
            println!("Backlog queue (filters: status={}, type={})",
                status.as_deref().unwrap_or("all"),
                r#type.as_deref().unwrap_or("all"),
            );
            println!("  (no items — connect to storage for persistence)");
        }
        QueueAction::Show { id } => {
            println!("Item #{id}: (connect to storage for persistence)");
        }
        QueueAction::Pop => {
            println!("Popping next item from queue...");
            println!("  (no items — connect to storage for persistence)");
        }
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
        assert_eq!(parse_intent("FEATURE").unwrap(), Intent::Feature);
    }

    #[test]
    fn parse_intent_invalid() {
        assert!(parse_intent("xxx").is_err());
    }
}
