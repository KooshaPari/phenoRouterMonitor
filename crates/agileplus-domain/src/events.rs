//! Domain events for AgilePlus.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::values::{Priority, Status};

/// A domain event representing a change in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum DomainEvent {
    /// A project was created.
    #[serde(rename = "project_created")]
    ProjectCreated {
        project_id: Uuid,
        name: String,
        description: String,
        timestamp: DateTime<Utc>,
    },

    /// A project was updated.
    #[serde(rename = "project_updated")]
    ProjectUpdated {
        project_id: Uuid,
        name: String,
        description: String,
        timestamp: DateTime<Utc>,
    },

    /// A work item was created.
    #[serde(rename = "work_item_created")]
    WorkItemCreated {
        work_item_id: Uuid,
        project_id: Uuid,
        title: String,
        description: String,
        status: Status,
        priority: Priority,
        timestamp: DateTime<Utc>,
    },

    /// A work item status changed.
    #[serde(rename = "work_item_status_changed")]
    WorkItemStatusChanged {
        work_item_id: Uuid,
        old_status: Status,
        new_status: Status,
        timestamp: DateTime<Utc>,
    },

    /// A work item priority changed.
    #[serde(rename = "work_item_priority_changed")]
    WorkItemPriorityChanged {
        work_item_id: Uuid,
        old_priority: Priority,
        new_priority: Priority,
        timestamp: DateTime<Utc>,
    },

    /// A work item was assigned.
    #[serde(rename = "work_item_assigned")]
    WorkItemAssigned {
        work_item_id: Uuid,
        assignee: String,
        timestamp: DateTime<Utc>,
    },

    /// A work item was unassigned.
    #[serde(rename = "work_item_unassigned")]
    WorkItemUnassigned {
        work_item_id: Uuid,
        timestamp: DateTime<Utc>,
    },

    /// A sprint was created.
    #[serde(rename = "sprint_created")]
    SprintCreated {
        sprint_id: Uuid,
        project_id: Uuid,
        name: String,
        goal: String,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        timestamp: DateTime<Utc>,
    },

    /// A sprint was started.
    #[serde(rename = "sprint_started")]
    SprintStarted {
        sprint_id: Uuid,
        timestamp: DateTime<Utc>,
    },

    /// A sprint was completed.
    #[serde(rename = "sprint_completed")]
    SprintCompleted {
        sprint_id: Uuid,
        timestamp: DateTime<Utc>,
    },
}

impl DomainEvent {
    /// Get the timestamp of this event.
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::ProjectCreated { timestamp, .. }
            | Self::ProjectUpdated { timestamp, .. }
            | Self::WorkItemCreated { timestamp, .. }
            | Self::WorkItemStatusChanged { timestamp, .. }
            | Self::WorkItemPriorityChanged { timestamp, .. }
            | Self::WorkItemAssigned { timestamp, .. }
            | Self::WorkItemUnassigned { timestamp, .. }
            | Self::SprintCreated { timestamp, .. }
            | Self::SprintStarted { timestamp, .. }
            | Self::SprintCompleted { timestamp, .. } => *timestamp,
        }
    }

    /// Get a human-readable event name.
    pub fn event_name(&self) -> &'static str {
        match self {
            Self::ProjectCreated { .. } => "ProjectCreated",
            Self::ProjectUpdated { .. } => "ProjectUpdated",
            Self::WorkItemCreated { .. } => "WorkItemCreated",
            Self::WorkItemStatusChanged { .. } => "WorkItemStatusChanged",
            Self::WorkItemPriorityChanged { .. } => "WorkItemPriorityChanged",
            Self::WorkItemAssigned { .. } => "WorkItemAssigned",
            Self::WorkItemUnassigned { .. } => "WorkItemUnassigned",
            Self::SprintCreated { .. } => "SprintCreated",
            Self::SprintStarted { .. } => "SprintStarted",
            Self::SprintCompleted { .. } => "SprintCompleted",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_created_event() {
        let event = DomainEvent::ProjectCreated {
            project_id: Uuid::new_v4(),
            name: "Test Project".into(),
            description: "A test project".into(),
            timestamp: Utc::now(),
        };

        assert_eq!(event.event_name(), "ProjectCreated");
    }

    #[test]
    fn work_item_status_changed_event() {
        let event = DomainEvent::WorkItemStatusChanged {
            work_item_id: Uuid::new_v4(),
            old_status: Status::Backlog,
            new_status: Status::InProgress,
            timestamp: Utc::now(),
        };

        assert_eq!(event.event_name(), "WorkItemStatusChanged");
    }

    #[test]
    fn work_item_assigned_event() {
        let event = DomainEvent::WorkItemAssigned {
            work_item_id: Uuid::new_v4(),
            assignee: "alice@example.com".into(),
            timestamp: Utc::now(),
        };

        assert_eq!(event.event_name(), "WorkItemAssigned");
    }

    #[test]
    fn events_serialize() {
        let event = DomainEvent::ProjectCreated {
            project_id: Uuid::new_v4(),
            name: "Test".into(),
            description: "desc".into(),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("project_created"));
    }
}
