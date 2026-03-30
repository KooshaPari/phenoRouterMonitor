//! Domain aggregates (aggregate roots) for AgilePlus.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{Project, WorkItem, Sprint};
use crate::events::DomainEvent;
use crate::values::{Priority, Status};

/// A ProjectAggregate is the aggregate root containing a Project and all related work items.
/// Changes to the project and its work items are tracked as domain events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAggregate {
    pub project: Project,
    pub work_items: Vec<WorkItem>,
    pub sprints: Vec<Sprint>,
    pub events: Vec<DomainEvent>,
}

impl ProjectAggregate {
    /// Create a new project aggregate.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let project = Project::new(name, description);

        let now = Utc::now();
        let event = DomainEvent::ProjectCreated {
            project_id: project.id,
            name: project.name.clone(),
            description: project.description.clone(),
            timestamp: now,
        };

        Self {
            project,
            work_items: Vec::new(),
            sprints: Vec::new(),
            events: vec![event],
        }
    }

    /// Add a work item to the project.
    pub fn add_work_item(
        &mut self,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Uuid {
        let work_item = WorkItem::new(self.project.id, title, description);
        let work_item_id = work_item.id;

        let now = Utc::now();
        self.events.push(DomainEvent::WorkItemCreated {
            work_item_id,
            project_id: self.project.id,
            title: work_item.title.clone(),
            description: work_item.description.clone(),
            status: work_item.status.clone(),
            priority: work_item.priority.clone(),
            timestamp: now,
        });

        self.work_items.push(work_item);
        work_item_id
    }

    /// Update the status of a work item.
    pub fn update_work_item_status(&mut self, work_item_id: Uuid, new_status: Status) -> bool {
        if let Some(work_item) = self.work_items.iter_mut().find(|w| w.id == work_item_id) {
            let old_status = work_item.status.clone();
            if old_status != new_status {
                work_item.status = new_status.clone();
                work_item.updated_at = Utc::now();

                self.events.push(DomainEvent::WorkItemStatusChanged {
                    work_item_id,
                    old_status,
                    new_status,
                    timestamp: work_item.updated_at,
                });

                return true;
            }
        }
        false
    }

    /// Update the priority of a work item.
    pub fn update_work_item_priority(&mut self, work_item_id: Uuid, new_priority: Priority) -> bool {
        if let Some(work_item) = self.work_items.iter_mut().find(|w| w.id == work_item_id) {
            let old_priority = work_item.priority.clone();
            if old_priority != new_priority {
                work_item.priority = new_priority.clone();
                work_item.updated_at = Utc::now();

                self.events.push(DomainEvent::WorkItemPriorityChanged {
                    work_item_id,
                    old_priority,
                    new_priority,
                    timestamp: work_item.updated_at,
                });

                return true;
            }
        }
        false
    }

    /// Assign a work item to a person.
    pub fn assign_work_item(&mut self, work_item_id: Uuid, assignee: impl Into<String>) -> bool {
        if let Some(work_item) = self.work_items.iter_mut().find(|w| w.id == work_item_id) {
            let assignee_str = assignee.into();
            work_item.assignee = Some(assignee_str.clone());
            work_item.updated_at = Utc::now();

            self.events.push(DomainEvent::WorkItemAssigned {
                work_item_id,
                assignee: assignee_str,
                timestamp: work_item.updated_at,
            });

            return true;
        }
        false
    }

    /// Unassign a work item.
    pub fn unassign_work_item(&mut self, work_item_id: Uuid) -> bool {
        if let Some(work_item) = self.work_items.iter_mut().find(|w| w.id == work_item_id) {
            if work_item.assignee.is_some() {
                work_item.assignee = None;
                work_item.updated_at = Utc::now();

                self.events.push(DomainEvent::WorkItemUnassigned {
                    work_item_id,
                    timestamp: work_item.updated_at,
                });

                return true;
            }
        }
        false
    }

    /// Add a sprint to the project.
    pub fn add_sprint(
        &mut self,
        name: impl Into<String>,
        goal: impl Into<String>,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Uuid {
        let sprint = Sprint {
            id: Uuid::new_v4(),
            project_id: self.project.id,
            name: name.into(),
            goal: goal.into(),
            start_date,
            end_date,
        };

        let sprint_id = sprint.id;
        let now = Utc::now();

        self.events.push(DomainEvent::SprintCreated {
            sprint_id,
            project_id: self.project.id,
            name: sprint.name.clone(),
            goal: sprint.goal.clone(),
            start_date: sprint.start_date,
            end_date: sprint.end_date,
            timestamp: now,
        });

        self.sprints.push(sprint);
        sprint_id
    }

    /// Get all uncommitted events.
    pub fn events(&self) -> &[DomainEvent] {
        &self.events
    }

    /// Clear all uncommitted events.
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// Get work item count.
    pub fn work_item_count(&self) -> usize {
        self.work_items.len()
    }

    /// Get sprint count.
    pub fn sprint_count(&self) -> usize {
        self.sprints.len()
    }

    /// Get work items in a specific status.
    pub fn work_items_with_status(&self, status: &Status) -> Vec<&WorkItem> {
        self.work_items.iter().filter(|w| &w.status == status).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_project_aggregate() {
        let agg = ProjectAggregate::new("Test Project", "A test project");
        assert_eq!(agg.work_item_count(), 0);
        assert_eq!(agg.sprint_count(), 0);
        assert_eq!(agg.events.len(), 1);
    }

    #[test]
    fn add_work_item_to_aggregate() {
        let mut agg = ProjectAggregate::new("Test", "desc");
        let wi_id = agg.add_work_item("Task 1", "Do something");

        assert_eq!(agg.work_item_count(), 1);
        assert_eq!(agg.events.len(), 2);
        assert!(!wi_id.is_nil());
    }

    #[test]
    fn update_work_item_status() {
        let mut agg = ProjectAggregate::new("Test", "desc");
        let wi_id = agg.add_work_item("Task", "desc");

        let result = agg.update_work_item_status(wi_id, Status::InProgress);
        assert!(result);
        assert_eq!(agg.events.len(), 3);

        let work_item = agg.work_items.iter().find(|w| w.id == wi_id).unwrap();
        assert_eq!(work_item.status, Status::InProgress);
    }

    #[test]
    fn update_work_item_priority() {
        let mut agg = ProjectAggregate::new("Test", "desc");
        let wi_id = agg.add_work_item("Task", "desc");

        let result = agg.update_work_item_priority(wi_id, Priority::High);
        assert!(result);

        let work_item = agg.work_items.iter().find(|w| w.id == wi_id).unwrap();
        assert_eq!(work_item.priority, Priority::High);
    }

    #[test]
    fn assign_and_unassign() {
        let mut agg = ProjectAggregate::new("Test", "desc");
        let wi_id = agg.add_work_item("Task", "desc");

        assert!(agg.assign_work_item(wi_id, "alice@example.com"));
        let work_item = agg.work_items.iter().find(|w| w.id == wi_id).unwrap();
        assert_eq!(work_item.assignee.as_ref().unwrap(), "alice@example.com");

        assert!(agg.unassign_work_item(wi_id));
        let work_item = agg.work_items.iter().find(|w| w.id == wi_id).unwrap();
        assert!(work_item.assignee.is_none());
    }

    #[test]
    fn add_sprint() {
        let mut agg = ProjectAggregate::new("Test", "desc");
        let now = Utc::now();
        let sprint_id = agg.add_sprint("Sprint 1", "Build feature X", now, now);

        assert_eq!(agg.sprint_count(), 1);
        assert!(!sprint_id.is_nil());
    }

    #[test]
    fn work_items_with_status() {
        let mut agg = ProjectAggregate::new("Test", "desc");
        let wi1 = agg.add_work_item("Task 1", "desc");
        let _wi2 = agg.add_work_item("Task 2", "desc");

        agg.update_work_item_status(wi1, Status::InProgress);

        let in_progress = agg.work_items_with_status(&Status::InProgress);
        assert_eq!(in_progress.len(), 1);

        let backlog = agg.work_items_with_status(&Status::Backlog);
        assert_eq!(backlog.len(), 1);
    }

    #[test]
    fn events_are_recorded() {
        let mut agg = ProjectAggregate::new("Test", "desc");
        agg.add_work_item("Task", "desc");
        agg.add_work_item("Task 2", "desc");

        assert_eq!(agg.events.len(), 3); // 1 project + 2 work items

        agg.clear_events();
        assert_eq!(agg.events.len(), 0);
    }
}
