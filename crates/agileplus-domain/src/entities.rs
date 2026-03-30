//! Domain entities for AgilePlus.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::values::{Priority, Status};

/// A project in AgilePlus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A work item (epic, story, task) in a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: String,
    pub status: Status,
    pub priority: Priority,
    pub assignee: Option<String>,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A sprint / iteration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprint {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub goal: String,
}

impl Project {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl WorkItem {
    pub fn new(project_id: Uuid, title: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            project_id,
            title: title.into(),
            description: description.into(),
            status: Status::Backlog,
            priority: Priority::Medium,
            assignee: None,
            parent_id: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_project() {
        let p = Project::new("Test", "A test project");
        assert_eq!(p.name, "Test");
        assert!(!p.id.is_nil());
    }

    #[test]
    fn create_work_item() {
        let p = Project::new("P", "desc");
        let wi = WorkItem::new(p.id, "Task 1", "Do something");
        assert_eq!(wi.project_id, p.id);
        assert_eq!(wi.status, Status::Backlog);
        assert_eq!(wi.priority, Priority::Medium);
    }
}
