//! Backlog item management.
//!
//! Persistent backlog with CRUD operations backed by SQLite.
//! Stores triaged items with priority, status, and metadata.
//!
//! Traceability: WP17-T099

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::classifier::Intent;

/// Priority levels for backlog items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Status of a backlog item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BacklogStatus {
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

/// A triaged backlog item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacklogItem {
    pub id: Option<i64>,
    pub title: String,
    pub description: String,
    pub intent: Intent,
    pub priority: BacklogPriority,
    pub status: BacklogStatus,
    pub source: String,
    pub feature_slug: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BacklogItem {
    /// Create a new backlog item from triage result.
    pub fn from_triage(title: String, description: String, intent: Intent, source: String) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            title,
            description,
            intent,
            priority: default_priority(intent),
            status: BacklogStatus::New,
            source,
            feature_slug: None,
            created_at: now,
            updated_at: now,
        }
    }
}

fn default_priority(intent: Intent) -> BacklogPriority {
    match intent {
        Intent::Bug => BacklogPriority::High,
        Intent::Feature => BacklogPriority::Medium,
        Intent::Idea => BacklogPriority::Low,
        Intent::Task => BacklogPriority::Medium,
    }
}

/// In-memory backlog store for unit testing and lightweight usage.
/// Production usage goes through SQLite via StoragePort extension.
#[derive(Debug, Default)]
pub struct BacklogStore {
    items: Vec<BacklogItem>,
    next_id: i64,
}

impl BacklogStore {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add(&mut self, mut item: BacklogItem) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        item.id = Some(id);
        self.items.push(item);
        id
    }

    pub fn get(&self, id: i64) -> Option<&BacklogItem> {
        self.items.iter().find(|i| i.id == Some(id))
    }

    pub fn get_mut(&mut self, id: i64) -> Option<&mut BacklogItem> {
        self.items.iter_mut().find(|i| i.id == Some(id))
    }

    pub fn list(&self) -> &[BacklogItem] {
        &self.items
    }

    pub fn list_by_status(&self, status: BacklogStatus) -> Vec<&BacklogItem> {
        self.items.iter().filter(|i| i.status == status).collect()
    }

    pub fn list_by_intent(&self, intent: Intent) -> Vec<&BacklogItem> {
        self.items.iter().filter(|i| i.intent == intent).collect()
    }

    pub fn update_status(&mut self, id: i64, status: BacklogStatus) -> bool {
        if let Some(item) = self.get_mut(id) {
            item.status = status;
            item.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn update_priority(&mut self, id: i64, priority: BacklogPriority) -> bool {
        if let Some(item) = self.get_mut(id) {
            item.priority = priority;
            item.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Pop the next item: highest priority New item, oldest first.
    pub fn pop_next(&mut self) -> Option<&BacklogItem> {
        let priority_order = |p: &BacklogPriority| -> u8 {
            match p {
                BacklogPriority::Critical => 0,
                BacklogPriority::High => 1,
                BacklogPriority::Medium => 2,
                BacklogPriority::Low => 3,
            }
        };

        // Find the index of the best candidate
        let idx = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, i)| i.status == BacklogStatus::New)
            .min_by(|(_, a), (_, b)| {
                priority_order(&a.priority)
                    .cmp(&priority_order(&b.priority))
                    .then(a.created_at.cmp(&b.created_at))
            })
            .map(|(idx, _)| idx);

        if let Some(idx) = idx {
            self.items[idx].status = BacklogStatus::Triaged;
            self.items[idx].updated_at = Utc::now();
            Some(&self.items[idx])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backlog_crud() {
        let mut store = BacklogStore::new();
        let item = BacklogItem::from_triage(
            "Fix login".into(),
            "Login broken on mobile".into(),
            Intent::Bug,
            "user-report".into(),
        );
        let id = store.add(item);
        assert_eq!(id, 1);

        let got = store.get(id).unwrap();
        assert_eq!(got.title, "Fix login");
        assert_eq!(got.priority, BacklogPriority::High); // bug default
        assert_eq!(got.status, BacklogStatus::New);
    }

    #[test]
    fn backlog_status_update() {
        let mut store = BacklogStore::new();
        let item = BacklogItem::from_triage("Task".into(), "desc".into(), Intent::Task, "cli".into());
        let id = store.add(item);
        assert!(store.update_status(id, BacklogStatus::InProgress));
        assert_eq!(store.get(id).unwrap().status, BacklogStatus::InProgress);
        assert!(!store.update_status(999, BacklogStatus::Done));
    }

    #[test]
    fn backlog_filter_by_status() {
        let mut store = BacklogStore::new();
        store.add(BacklogItem::from_triage("a".into(), "".into(), Intent::Bug, "".into()));
        store.add(BacklogItem::from_triage("b".into(), "".into(), Intent::Feature, "".into()));
        store.update_status(1, BacklogStatus::Done);
        assert_eq!(store.list_by_status(BacklogStatus::New).len(), 1);
        assert_eq!(store.list_by_status(BacklogStatus::Done).len(), 1);
    }

    #[test]
    fn backlog_filter_by_intent() {
        let mut store = BacklogStore::new();
        store.add(BacklogItem::from_triage("a".into(), "".into(), Intent::Bug, "".into()));
        store.add(BacklogItem::from_triage("b".into(), "".into(), Intent::Bug, "".into()));
        store.add(BacklogItem::from_triage("c".into(), "".into(), Intent::Idea, "".into()));
        assert_eq!(store.list_by_intent(Intent::Bug).len(), 2);
        assert_eq!(store.list_by_intent(Intent::Idea).len(), 1);
    }

    #[test]
    fn backlog_pop_priority_order() {
        let mut store = BacklogStore::new();
        store.add(BacklogItem::from_triage("low".into(), "".into(), Intent::Idea, "".into()));
        store.add(BacklogItem::from_triage("high".into(), "".into(), Intent::Bug, "".into()));
        store.add(BacklogItem::from_triage("med".into(), "".into(), Intent::Feature, "".into()));

        let first = store.pop_next().unwrap();
        assert_eq!(first.title, "high"); // Bug = High priority
        assert_eq!(first.status, BacklogStatus::Triaged);
    }

    #[test]
    fn default_priorities() {
        assert_eq!(default_priority(Intent::Bug), BacklogPriority::High);
        assert_eq!(default_priority(Intent::Feature), BacklogPriority::Medium);
        assert_eq!(default_priority(Intent::Idea), BacklogPriority::Low);
        assert_eq!(default_priority(Intent::Task), BacklogPriority::Medium);
    }
}
