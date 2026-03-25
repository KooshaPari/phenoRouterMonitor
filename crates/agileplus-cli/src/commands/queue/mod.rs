//! `agileplus queue` command implementation.
//!
//! Manages the triage backlog: add, list, show, pop items.
//!
//! Traceability: FR-049 / WP21-T122

use std::path::PathBuf;

use anyhow::Result;

use agileplus_domain::domain::backlog::BacklogFilters;
use agileplus_domain::ports::ContentStoragePort;
use agileplus_triage::TriageClassifier;

mod import;
mod output;
mod parsing;

/// Arguments for the `queue` subcommand.
#[derive(Debug, clap::Args)]
pub struct QueueArgs {
    #[command(subcommand)]
    pub action: QueueAction,
}

#[derive(Debug, clap::Subcommand)]
pub enum QueueAction {
    /// Add one item to the backlog queue.
    Add {
        /// Item text to classify and queue.
        #[arg(value_name = "TEXT", required_unless_present = "from_file")]
        text: Option<String>,
        /// Item description or body text.
        #[arg(long, default_value = "")]
        description: String,
        /// Item type (bug, feature, idea, task). Auto-classified if omitted.
        #[arg(long)]
        r#type: Option<String>,
        /// Priority override (critical, high, medium, low).
        #[arg(long)]
        priority: Option<String>,
        /// Comma-separated tags to attach to every added item.
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        /// Source identifier recorded on created backlog items.
        #[arg(long, default_value = "cli")]
        source: String,
        /// Optional feature slug to associate with the item.
        #[arg(long)]
        feature_slug: Option<String>,
        /// Add multiple items from a newline-delimited JSON file.
        #[arg(long)]
        from_file: Option<PathBuf>,
    },
    /// Import multiple backlog items from a newline-delimited JSON file.
    Import {
        /// Path to the newline-delimited JSON file.
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Item description or body text used as a fallback when missing from the file.
        #[arg(long, default_value = "")]
        description: String,
        /// Item type (bug, feature, idea, task). Auto-classified if omitted.
        #[arg(long)]
        r#type: Option<String>,
        /// Priority override (critical, high, medium, low).
        #[arg(long)]
        priority: Option<String>,
        /// Comma-separated tags to attach to every imported item.
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        /// Source identifier recorded on imported backlog items.
        #[arg(long, default_value = "cli")]
        source: String,
        /// Optional feature slug to associate with every imported item.
        #[arg(long)]
        feature_slug: Option<String>,
    },
    /// List items in the backlog.
    List {
        /// Filter by type (bug, feature, idea, task).
        #[arg(long)]
        r#type: Option<String>,
        /// Filter by status (new, triaged, in_progress, done, dismissed).
        #[arg(long)]
        status: Option<String>,
        /// Filter by priority (critical, high, medium, low).
        #[arg(long)]
        priority: Option<String>,
        /// Filter by feature slug.
        #[arg(long)]
        feature_slug: Option<String>,
        /// Filter by source.
        #[arg(long)]
        source: Option<String>,
        /// Sorting mode: priority (default), age, impact.
        #[arg(long, default_value = "priority")]
        sort: String,
        /// Output format: plain text (default) or json.
        #[arg(long, default_value = "table")]
        output: String,
        /// Limit the number of items returned.
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Show details for a specific backlog item.
    Show {
        /// Item ID.
        id: i64,
        /// Output format: plain text (default) or json.
        #[arg(long, default_value = "table")]
        output: String,
    },
    /// Pop the next highest-priority item from the queue.
    Pop {
        /// Number of items to pop.
        #[arg(long, default_value_t = 1)]
        count: usize,
        /// Output format: plain text (default) or json.
        #[arg(long, default_value = "table")]
        output: String,
    },
}

/// Run the `queue` command.
pub async fn run_queue<S>(args: QueueArgs, storage: &S) -> Result<()>
where
    S: ContentStoragePort + Send + Sync,
{
    let classifier = TriageClassifier::new();

    match args.action {
        QueueAction::Add {
            text,
            description,
            r#type,
            priority,
            tags,
            source,
            feature_slug,
            from_file,
        } => {
            let items = if let Some(path) = from_file {
                import::build_items_from_file(
                    &classifier,
                    &path,
                    import::BuildFileParams {
                        default_description: description,
                        default_type: r#type,
                        default_priority: priority,
                        default_tags: tags,
                        default_source: source,
                        default_feature_slug: feature_slug,
                    },
                )?
            } else {
                let text = text.unwrap_or_default();
                if text.is_empty() {
                    anyhow::bail!("No item text provided");
                }
                vec![import::build_item(
                    &classifier,
                    import::BuildItemParams {
                        title: text,
                        description,
                        intent: r#type,
                        priority,
                        tags,
                        source,
                        feature_slug,
                    },
                )?]
            };

            let created = import::persist_items(storage, items).await?;
            if created.is_empty() {
                println!("No backlog items created");
            } else if created.len() == 1 {
                let item = &created[0];
                println!(
                    "Added backlog item #{:>4}: [{}] {} ({}, {})",
                    item.id.unwrap_or_default(),
                    item.intent,
                    item.title,
                    item.priority,
                    item.status,
                );
            } else {
                println!("Imported {} backlog items:", created.len());
                for item in created {
                    println!(
                        "  #{:>4} [{}] {} ({}, {})",
                        item.id.unwrap_or_default(),
                        item.intent,
                        item.title,
                        item.priority,
                        item.status,
                    );
                }
            }
        }
        QueueAction::Import {
            file,
            description,
            r#type,
            priority,
            tags,
            source,
            feature_slug,
        } => {
            let items = import::build_items_from_file(
                &classifier,
                &file,
                import::BuildFileParams {
                    default_description: description,
                    default_type: r#type,
                    default_priority: priority,
                    default_tags: tags,
                    default_source: source,
                    default_feature_slug: feature_slug,
                },
            )?;
            let created = import::persist_items(storage, items).await?;
            println!("Imported {} backlog items:", created.len());
            for item in created {
                println!(
                    "  #{:>4} [{}] {} ({}, {})",
                    item.id.unwrap_or_default(),
                    item.intent,
                    item.title,
                    item.priority,
                    item.status,
                );
            }
        }
        QueueAction::List {
            r#type,
            status,
            priority,
            feature_slug,
            source,
            sort,
            output,
            limit,
        } => {
            let filters = BacklogFilters {
                intent: parsing::parse_intent_opt(r#type)?,
                status: parsing::parse_status_opt(status)?,
                priority: parsing::parse_priority_opt(priority)?,
                feature_slug,
                source,
                limit: Some(limit),
                sort: parsing::parse_sort(&sort)?,
            };
            let items = storage.list_backlog_items(&filters).await?;
            output::print_backlog_items(&items, &output)?;
        }
        QueueAction::Show { id, output } => {
            let item = storage
                .get_backlog_item(id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Backlog item #{id} not found"))?;
            output::print_backlog_item(&item, &output)?;
        }
        QueueAction::Pop { count, output } => {
            let mut popped = Vec::new();
            for _ in 0..count {
                if let Some(item) = storage.pop_next_backlog_item().await? {
                    popped.push(item);
                } else {
                    break;
                }
            }
            output::print_backlog_items(&popped, &output)?;
        }
    }

    Ok(())
}
