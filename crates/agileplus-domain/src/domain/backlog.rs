use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Intent classification for queued work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Intent {
    Bug,
    Feature,
    Idea,
    Task,
}

impl std::fmt::Display for Intent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bug => write!(f, "bug"),
            Self::Feature => write!(f, "feature"),
            Self::Idea => write!(f, "idea"),
            Self::Task => write!(f, "task"),
        }
    }
}

impl FromStr for Intent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bug" => Ok(Self::Bug),
            "feature" => Ok(Self::Feature),
            "idea" => Ok(Self::Idea),
            "task" => Ok(Self::Task),
            other => Err(format!("unknown intent '{other}'")),
        }
    }
}

impl Intent {
    pub fn default_priority(self) -> BacklogPriority {
        match self {
            Self::Bug => BacklogPriority::High,
            Self::Feature => BacklogPriority::Medium,
            Self::Idea => BacklogPriority::Low,
            Self::Task => BacklogPriority::Medium,
        }
    }
}

/// Priority levels for backlog items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BacklogPriority {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for BacklogPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "critical"),
            Self::High => write!(f, "high"),
            Self::Medium => write!(f, "medium"),
            Self::Low => write!(f, "low"),
        }
    }
}

impl FromStr for BacklogPriority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "critical" => Ok(Self::Critical),
            "high" => Ok(Self::High),
            "medium" => Ok(Self::Medium),
            "low" => Ok(Self::Low),
            other => Err(format!("unknown backlog priority '{other}'")),
        }
    }
}

impl BacklogPriority {
    pub fn rank(self) -> u8 {
        match self {
            Self::Critical => 0,
            Self::High => 1,
            Self::Medium => 2,
            Self::Low => 3,
        }
    }
}

/// Queue lifecycle state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BacklogStatus {
    #[default]
    New,
    Triaged,
    InProgress,
    Done,
    Dismissed,
}

impl std::fmt::Display for BacklogStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "new"),
            Self::Triaged => write!(f, "triaged"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Done => write!(f, "done"),
            Self::Dismissed => write!(f, "dismissed"),
        }
    }
}

impl FromStr for BacklogStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "new" => Ok(Self::New),
            "triaged" => Ok(Self::Triaged),
            "in_progress" | "in-progress" => Ok(Self::InProgress),
            "done" => Ok(Self::Done),
            "dismissed" => Ok(Self::Dismissed),
            other => Err(format!("unknown backlog status '{other}'")),
        }
    }
}

impl BacklogStatus {
    pub fn is_open(self) -> bool {
        matches!(self, Self::New | Self::Triaged | Self::InProgress)
    }
}

/// Sorting modes for backlog queries.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BacklogSort {
    #[default]
    Priority,
    Age,
    Impact,
}

impl std::fmt::Display for BacklogSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Priority => write!(f, "priority"),
            Self::Age => write!(f, "age"),
            Self::Impact => write!(f, "impact"),
        }
    }
}

impl FromStr for BacklogSort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "priority" => Ok(Self::Priority),
            "age" => Ok(Self::Age),
            "impact" => Ok(Self::Impact),
            other => Err(format!("unknown backlog sort '{other}'")),
        }
    }
}

/// Backlog query filters shared by CLI, API, and storage adapters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BacklogFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<Intent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<BacklogStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<BacklogPriority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(default)]
    pub sort: BacklogSort,
}

/// A queued backlog item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklogItem {
    pub id: Option<i64>,
    pub title: String,
    pub description: String,
    pub intent: Intent,
    pub priority: BacklogPriority,
    pub status: BacklogStatus,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_slug: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BacklogItem {
    pub fn from_triage(title: String, description: String, intent: Intent, source: String) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            title,
            description,
            intent,
            priority: intent.default_priority(),
            status: BacklogStatus::New,
            source,
            feature_slug: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_priority(mut self, priority: BacklogPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_status(mut self, status: BacklogStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_feature_slug(mut self, feature_slug: Option<String>) -> Self {
        self.feature_slug = feature_slug;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}
