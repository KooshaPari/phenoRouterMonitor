//! Batch import helpers for `agileplus queue`.

use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

use agileplus_domain::domain::backlog::BacklogItem;
use agileplus_domain::ports::ContentStoragePort;
use agileplus_triage::TriageClassifier;

use super::parsing;

#[derive(Debug, Deserialize)]
pub(crate) struct QueueImportRecord {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub feature_slug: Option<String>,
}

pub(crate) fn build_item(
    classifier: &TriageClassifier,
    title: String,
    description: String,
    intent: Option<String>,
    priority: Option<String>,
    tags: Vec<String>,
    source: String,
    feature_slug: Option<String>,
) -> Result<BacklogItem> {
    let intent = if let Some(intent) = intent {
        parsing::parse_intent(Some(intent))?
    } else {
        classifier.classify(&title).intent
    };

    let mut item = BacklogItem::from_triage(title, description, intent, source)
        .with_tags(tags)
        .with_feature_slug(feature_slug);
    if let Some(priority) = priority {
        item.priority = parsing::parse_priority(priority)?;
    }
    Ok(item)
}

pub(crate) fn build_items_from_file(
    classifier: &TriageClassifier,
    path: &PathBuf,
    default_description: String,
    default_type: Option<String>,
    default_priority: Option<String>,
    default_tags: Vec<String>,
    default_source: String,
    default_feature_slug: Option<String>,
) -> Result<Vec<BacklogItem>> {
    let content = std::fs::read_to_string(path)?;
    let mut items = Vec::new();

    for line in content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let record = parse_import_record(line)?;
        items.push(build_item(
            classifier,
            record.title,
            if record.description.is_empty() {
                default_description.clone()
            } else {
                record.description
            },
            record.r#type.or_else(|| default_type.clone()),
            record.priority.or_else(|| default_priority.clone()),
            if record.tags.is_empty() {
                default_tags.clone()
            } else {
                record.tags
            },
            record.source.unwrap_or_else(|| default_source.clone()),
            record.feature_slug.or_else(|| default_feature_slug.clone()),
        )?);
    }

    Ok(items)
}

pub(crate) async fn persist_items<S>(
    storage: &S,
    items: Vec<BacklogItem>,
) -> Result<Vec<BacklogItem>>
where
    S: ContentStoragePort + Send + Sync,
{
    let mut created = Vec::with_capacity(items.len());
    for item in items {
        let id = storage.create_backlog_item(&item).await?;
        created.push(BacklogItem {
            id: Some(id),
            ..item
        });
    }
    Ok(created)
}

fn parse_import_record(line: &str) -> Result<QueueImportRecord> {
    serde_json::from_str(line).map_err(|e| anyhow::anyhow!("Invalid backlog import record: {e}"))
}
