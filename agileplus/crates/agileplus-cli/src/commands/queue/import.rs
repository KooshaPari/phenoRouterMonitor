//! Batch import helpers for `agileplus queue`.

use std::path::Path;

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

/// Parameters for building a backlog item (reduces argument count).
pub(crate) struct BuildItemParams {
    pub title: String,
    pub description: String,
    pub intent: Option<String>,
    pub priority: Option<String>,
    pub tags: Vec<String>,
    pub source: String,
    pub feature_slug: Option<String>,
}

pub(crate) fn build_item(
    classifier: &TriageClassifier,
    params: BuildItemParams,
) -> Result<BacklogItem> {
    let intent = if let Some(intent) = params.intent {
        parsing::parse_intent(Some(intent))?
    } else {
        classifier.classify(&params.title).intent
    };

    let mut item =
        BacklogItem::from_triage(params.title, params.description, intent, params.source)
            .with_tags(params.tags)
            .with_feature_slug(params.feature_slug);
    if let Some(priority) = params.priority {
        item.priority = parsing::parse_priority(priority)?;
    }
    Ok(item)
}

/// Parameters for building items from a file (reduces argument count).
pub(crate) struct BuildFileParams {
    pub default_description: String,
    pub default_type: Option<String>,
    pub default_priority: Option<String>,
    pub default_tags: Vec<String>,
    pub default_source: String,
    pub default_feature_slug: Option<String>,
}

pub(crate) fn build_items_from_file(
    classifier: &TriageClassifier,
    path: &Path,
    defaults: BuildFileParams,
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
            BuildItemParams {
                title: record.title,
                description: if record.description.is_empty() {
                    defaults.default_description.clone()
                } else {
                    record.description
                },
                intent: record.r#type.or_else(|| defaults.default_type.clone()),
                priority: record
                    .priority
                    .or_else(|| defaults.default_priority.clone()),
                tags: if record.tags.is_empty() {
                    defaults.default_tags.clone()
                } else {
                    record.tags
                },
                source: record
                    .source
                    .unwrap_or_else(|| defaults.default_source.clone()),
                feature_slug: record
                    .feature_slug
                    .or_else(|| defaults.default_feature_slug.clone()),
            },
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
