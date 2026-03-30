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

/// A work package is a collection of related tasks that implement part of a spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPackage {
    pub id: Uuid,
    pub spec_id: Uuid,
    pub title: String,
    pub description: String,
    pub status: Status,
    pub priority: Priority,
    pub assignee: Option<String>,
    pub tasks: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A task is an atomic unit of work within a work package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub work_package_id: Uuid,
    pub title: String,
    pub description: String,
    pub status: Status,
    pub priority: Priority,
    pub assignee: Option<String>,
    pub estimated_hours: Option<f32>,
    pub actual_hours: Option<f32>,
    pub dependencies: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A spec (specification) document defining requirements and acceptance criteria.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: String,
    pub version: String,
    pub status: Status,
    pub acceptance_criteria: Vec<String>,
    pub work_packages: Vec<Uuid>,
    pub author: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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

impl WorkPackage {
    pub fn new(spec_id: Uuid, title: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            spec_id,
            title: title.into(),
            description: description.into(),
            status: Status::Backlog,
            priority: Priority::Medium,
            assignee: None,
            tasks: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a task to this work package.
    pub fn add_task(&mut self, task_id: Uuid) {
        if !self.tasks.contains(&task_id) {
            self.tasks.push(task_id);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a task from this work package.
    pub fn remove_task(&mut self, task_id: Uuid) -> bool {
        let initial_len = self.tasks.len();
        self.tasks.retain(|&id| id != task_id);
        if self.tasks.len() < initial_len {
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Validate state transitions: ensure status transitions are valid.
    pub fn can_transition_to(&self, new_status: Status) -> bool {
        match (self.status, new_status) {
            (Status::Backlog, Status::Todo) => true,
            (Status::Todo, Status::InProgress) => true,
            (Status::InProgress, Status::InReview) => true,
            (Status::InReview, Status::Done) => true,
            (s, Status::Cancelled) => s != Status::Done && s != Status::Cancelled,
            (Status::Done, _) => false,
            (Status::Cancelled, _) => false,
            _ => false,
        }
    }

    /// Transition to a new status if allowed.
    pub fn transition_to(&mut self, new_status: Status) -> Result<(), String> {
        if self.can_transition_to(new_status) {
            self.status = new_status;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!("Cannot transition from {} to {}", self.status, new_status))
        }
    }
}

impl Task {
    pub fn new(work_package_id: Uuid, title: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            work_package_id,
            title: title.into(),
            description: description.into(),
            status: Status::Backlog,
            priority: Priority::Medium,
            assignee: None,
            estimated_hours: None,
            actual_hours: None,
            dependencies: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a dependency on another task.
    pub fn add_dependency(&mut self, task_id: Uuid) {
        if !self.dependencies.contains(&task_id) && task_id != self.id {
            self.dependencies.push(task_id);
            self.updated_at = Utc::now();
        }
    }

    /// Check if this task is blocked (has unmet dependencies).
    pub fn is_blocked(&self, completed_tasks: &[Uuid]) -> bool {
        self.dependencies.iter().any(|dep_id| !completed_tasks.contains(dep_id))
    }

    /// Validate state transitions for tasks.
    pub fn can_transition_to(&self, new_status: Status) -> bool {
        match (self.status, new_status) {
            (Status::Backlog, Status::Todo) => true,
            (Status::Todo, Status::InProgress) => true,
            (Status::InProgress, Status::InReview) => true,
            (Status::InReview, Status::Done) => true,
            (s, Status::Cancelled) => s != Status::Done && s != Status::Cancelled,
            (Status::Done, _) => false,
            (Status::Cancelled, _) => false,
            _ => false,
        }
    }

    /// Transition to a new status if allowed.
    pub fn transition_to(&mut self, new_status: Status) -> Result<(), String> {
        if self.can_transition_to(new_status) {
            self.status = new_status;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!("Cannot transition from {} to {}", self.status, new_status))
        }
    }

    /// Record time worked on this task.
    pub fn log_time(&mut self, hours: f32) -> Result<(), String> {
        if hours <= 0.0 {
            return Err("Hours must be positive".to_string());
        }
        match self.actual_hours {
            Some(current) => self.actual_hours = Some(current + hours),
            None => self.actual_hours = Some(hours),
        }
        self.updated_at = Utc::now();
        Ok(())
    }
}

impl Spec {
    pub fn new(project_id: Uuid, title: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            project_id,
            title: title.into(),
            description: description.into(),
            version: "0.1.0".to_string(),
            status: Status::Backlog,
            acceptance_criteria: Vec::new(),
            work_packages: Vec::new(),
            author: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add an acceptance criterion to this spec.
    pub fn add_acceptance_criterion(&mut self, criterion: impl Into<String>) {
        self.acceptance_criteria.push(criterion.into());
        self.updated_at = Utc::now();
    }

    /// Add a work package to this spec.
    pub fn add_work_package(&mut self, wp_id: Uuid) {
        if !self.work_packages.contains(&wp_id) {
            self.work_packages.push(wp_id);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a work package from this spec.
    pub fn remove_work_package(&mut self, wp_id: Uuid) -> bool {
        let initial_len = self.work_packages.len();
        self.work_packages.retain(|&id| id != wp_id);
        if self.work_packages.len() < initial_len {
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Check if spec is valid (has acceptance criteria and work packages).
    pub fn is_valid(&self) -> bool {
        !self.acceptance_criteria.is_empty() && !self.work_packages.is_empty()
    }

    /// Validate state transitions for specs.
    pub fn can_transition_to(&self, new_status: Status) -> bool {
        match (self.status, new_status) {
            (Status::Backlog, Status::Todo) => true,
            (Status::Todo, Status::InProgress) => true,
            (Status::InProgress, Status::InReview) => true,
            (Status::InReview, Status::Done) => self.is_valid(),
            (s, Status::Cancelled) => s != Status::Done && s != Status::Cancelled,
            (Status::Done, _) => false,
            (Status::Cancelled, _) => false,
            _ => false,
        }
    }

    /// Transition to a new status if allowed.
    pub fn transition_to(&mut self, new_status: Status) -> Result<(), String> {
        if self.can_transition_to(new_status) {
            self.status = new_status;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!("Cannot transition from {} to {}", self.status, new_status))
        }
    }

    /// Increment the version (e.g., from 0.1.0 to 0.2.0).
    pub fn increment_version(&mut self) {
        let parts: Vec<&str> = self.version.split('.').collect();
        if parts.len() == 3 {
            if let (Ok(major), Ok(minor), Ok(_patch)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                self.version = format!("{}.{}.{}", major, minor + 1, 0);
                self.updated_at = Utc::now();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== FR-PHENO-DOMAIN-001: Project Creation =====
    #[test]
    fn test_create_project() {
        let p = Project::new("Test", "A test project");
        assert_eq!(p.name, "Test");
        assert_eq!(p.description, "A test project");
        assert!(!p.id.is_nil());
    }

    // ===== FR-PHENO-DOMAIN-002: WorkItem Creation =====
    #[test]
    fn test_create_work_item() {
        let p = Project::new("P", "desc");
        let wi = WorkItem::new(p.id, "Task 1", "Do something");
        assert_eq!(wi.project_id, p.id);
        assert_eq!(wi.status, Status::Backlog);
        assert_eq!(wi.priority, Priority::Medium);
        assert!(wi.assignee.is_none());
    }

    // ===== FR-PHENO-DOMAIN-003: WorkPackage Creation =====
    #[test]
    fn test_create_work_package() {
        let spec_id = Uuid::new_v4();
        let wp = WorkPackage::new(spec_id, "WP-001", "Implement feature");
        assert_eq!(wp.spec_id, spec_id);
        assert_eq!(wp.title, "WP-001");
        assert_eq!(wp.status, Status::Backlog);
        assert!(wp.tasks.is_empty());
    }

    // ===== FR-PHENO-DOMAIN-004: Task Creation =====
    #[test]
    fn test_create_task() {
        let wp_id = Uuid::new_v4();
        let task = Task::new(wp_id, "Task 1", "Complete feature");
        assert_eq!(task.work_package_id, wp_id);
        assert_eq!(task.status, Status::Backlog);
        assert!(task.dependencies.is_empty());
        assert!(task.actual_hours.is_none());
    }

    // ===== FR-PHENO-DOMAIN-005: Spec Creation =====
    #[test]
    fn test_create_spec() {
        let project_id = Uuid::new_v4();
        let spec = Spec::new(project_id, "Authentication", "User login spec");
        assert_eq!(spec.project_id, project_id);
        assert_eq!(spec.version, "0.1.0");
        assert_eq!(spec.status, Status::Backlog);
        assert!(spec.acceptance_criteria.is_empty());
    }

    // ===== FR-PHENO-DOMAIN-006: WorkPackage Task Management =====
    #[test]
    fn test_work_package_add_task() {
        let spec_id = Uuid::new_v4();
        let mut wp = WorkPackage::new(spec_id, "WP", "desc");
        let task_id = Uuid::new_v4();

        wp.add_task(task_id);
        assert!(wp.tasks.contains(&task_id));
        assert_eq!(wp.tasks.len(), 1);

        // Adding same task twice should not duplicate
        wp.add_task(task_id);
        assert_eq!(wp.tasks.len(), 1);
    }

    // ===== FR-PHENO-DOMAIN-007: WorkPackage Task Removal =====
    #[test]
    fn test_work_package_remove_task() {
        let spec_id = Uuid::new_v4();
        let mut wp = WorkPackage::new(spec_id, "WP", "desc");
        let task_id = Uuid::new_v4();

        wp.add_task(task_id);
        assert!(wp.remove_task(task_id));
        assert!(!wp.tasks.contains(&task_id));

        // Removing non-existent task returns false
        assert!(!wp.remove_task(task_id));
    }

    // ===== FR-PHENO-DOMAIN-008: Task Dependencies =====
    #[test]
    fn test_task_add_dependency() {
        let wp_id = Uuid::new_v4();
        let mut task = Task::new(wp_id, "Task 1", "desc");
        let dep_id = Uuid::new_v4();

        task.add_dependency(dep_id);
        assert!(task.dependencies.contains(&dep_id));

        // Adding same dependency twice should not duplicate
        task.add_dependency(dep_id);
        assert_eq!(task.dependencies.len(), 1);
    }

    // ===== FR-PHENO-DOMAIN-009: Task Self-Dependency Prevention =====
    #[test]
    fn test_task_cannot_depend_on_self() {
        let wp_id = Uuid::new_v4();
        let mut task = Task::new(wp_id, "Task 1", "desc");

        // Should not add self as dependency
        task.add_dependency(task.id);
        assert!(task.dependencies.is_empty());
    }

    // ===== FR-PHENO-DOMAIN-010: Task Blocking Detection =====
    #[test]
    fn test_task_blocking_detection() {
        let wp_id = Uuid::new_v4();
        let mut task = Task::new(wp_id, "Task 2", "desc");
        let dep_id = Uuid::new_v4();

        task.add_dependency(dep_id);

        // Task is blocked when dependency not in completed list
        assert!(task.is_blocked(&[]));
        assert!(task.is_blocked(&[Uuid::new_v4()]));

        // Task is not blocked when dependency is completed
        assert!(!task.is_blocked(&[dep_id]));
    }

    // ===== FR-PHENO-DOMAIN-011: Task Time Logging =====
    #[test]
    fn test_task_log_time() {
        let wp_id = Uuid::new_v4();
        let mut task = Task::new(wp_id, "Task", "desc");

        assert!(task.log_time(2.5).is_ok());
        assert_eq!(task.actual_hours, Some(2.5));

        // Accumulate time
        assert!(task.log_time(1.5).is_ok());
        assert_eq!(task.actual_hours, Some(4.0));
    }

    // ===== FR-PHENO-DOMAIN-012: Task Time Logging Validation =====
    #[test]
    fn test_task_log_time_invalid() {
        let wp_id = Uuid::new_v4();
        let mut task = Task::new(wp_id, "Task", "desc");

        assert!(task.log_time(0.0).is_err());
        assert!(task.log_time(-1.0).is_err());
    }

    // ===== FR-PHENO-DOMAIN-013: WorkPackage State Transition =====
    #[test]
    fn test_work_package_state_transition() {
        let spec_id = Uuid::new_v4();
        let mut wp = WorkPackage::new(spec_id, "WP", "desc");

        assert_eq!(wp.status, Status::Backlog);
        assert!(wp.transition_to(Status::Todo).is_ok());
        assert_eq!(wp.status, Status::Todo);

        assert!(wp.transition_to(Status::InProgress).is_ok());
        assert_eq!(wp.status, Status::InProgress);

        assert!(wp.transition_to(Status::InReview).is_ok());
        assert_eq!(wp.status, Status::InReview);

        assert!(wp.transition_to(Status::Done).is_ok());
        assert_eq!(wp.status, Status::Done);

        // Cannot transition from Done
        assert!(wp.transition_to(Status::Backlog).is_err());
    }

    // ===== FR-PHENO-DOMAIN-014: Task State Transition =====
    #[test]
    fn test_task_state_transition() {
        let wp_id = Uuid::new_v4();
        let mut task = Task::new(wp_id, "Task", "desc");

        assert_eq!(task.status, Status::Backlog);
        assert!(task.transition_to(Status::Todo).is_ok());
        assert_eq!(task.status, Status::Todo);

        assert!(task.transition_to(Status::InProgress).is_ok());
        assert!(task.transition_to(Status::InReview).is_ok());
        assert!(task.transition_to(Status::Done).is_ok());

        // Cannot transition from Done
        assert!(task.transition_to(Status::Backlog).is_err());
    }

    // ===== FR-PHENO-DOMAIN-015: Task to Cancelled Transition =====
    #[test]
    fn test_task_cancel_from_various_states() {
        let wp_id = Uuid::new_v4();
        let mut task = Task::new(wp_id, "Task", "desc");

        assert!(task.transition_to(Status::Cancelled).is_ok());
        assert_eq!(task.status, Status::Cancelled);

        // Cannot transition from Cancelled
        assert!(task.transition_to(Status::Backlog).is_err());
    }

    // ===== FR-PHENO-DOMAIN-016: Spec Acceptance Criteria =====
    #[test]
    fn test_spec_add_acceptance_criterion() {
        let project_id = Uuid::new_v4();
        let mut spec = Spec::new(project_id, "Auth", "desc");

        assert!(spec.acceptance_criteria.is_empty());
        spec.add_acceptance_criterion("User can login");
        assert_eq!(spec.acceptance_criteria.len(), 1);

        spec.add_acceptance_criterion("User can logout");
        assert_eq!(spec.acceptance_criteria.len(), 2);
    }

    // ===== FR-PHENO-DOMAIN-017: Spec Work Package Management =====
    #[test]
    fn test_spec_add_work_package() {
        let project_id = Uuid::new_v4();
        let mut spec = Spec::new(project_id, "Auth", "desc");
        let wp_id = Uuid::new_v4();

        spec.add_work_package(wp_id);
        assert!(spec.work_packages.contains(&wp_id));

        // Adding same WP twice should not duplicate
        spec.add_work_package(wp_id);
        assert_eq!(spec.work_packages.len(), 1);
    }

    // ===== FR-PHENO-DOMAIN-018: Spec Work Package Removal =====
    #[test]
    fn test_spec_remove_work_package() {
        let project_id = Uuid::new_v4();
        let mut spec = Spec::new(project_id, "Auth", "desc");
        let wp_id = Uuid::new_v4();

        spec.add_work_package(wp_id);
        assert!(spec.remove_work_package(wp_id));
        assert!(!spec.work_packages.contains(&wp_id));

        // Removing non-existent WP returns false
        assert!(!spec.remove_work_package(wp_id));
    }

    // ===== FR-PHENO-DOMAIN-019: Spec Validity =====
    #[test]
    fn test_spec_validity_check() {
        let project_id = Uuid::new_v4();
        let mut spec = Spec::new(project_id, "Auth", "desc");

        // Invalid: no criteria, no work packages
        assert!(!spec.is_valid());

        // Add criteria
        spec.add_acceptance_criterion("User can login");
        assert!(!spec.is_valid()); // Still need work packages

        // Add work package
        spec.add_work_package(Uuid::new_v4());
        assert!(spec.is_valid());
    }

    // ===== FR-PHENO-DOMAIN-020: Spec State Transition with Validation =====
    #[test]
    fn test_spec_state_transition_with_validation() {
        let project_id = Uuid::new_v4();
        let mut spec = Spec::new(project_id, "Auth", "desc");

        // Can transition from Backlog to Todo even without criteria
        assert!(spec.transition_to(Status::Todo).is_ok());
        assert!(spec.transition_to(Status::InProgress).is_ok());
        assert!(spec.transition_to(Status::InReview).is_ok());

        // Cannot transition to Done without valid criteria and work packages
        assert!(spec.transition_to(Status::Done).is_err());

        // Add criteria and work packages
        spec.add_acceptance_criterion("Login works");
        spec.add_work_package(Uuid::new_v4());

        // Now can transition to Done
        assert!(spec.transition_to(Status::Done).is_ok());
        assert_eq!(spec.status, Status::Done);
    }

    // ===== FR-PHENO-DOMAIN-021: Spec Version Management =====
    #[test]
    fn test_spec_version_increment() {
        let project_id = Uuid::new_v4();
        let mut spec = Spec::new(project_id, "Auth", "desc");

        assert_eq!(spec.version, "0.1.0");

        spec.increment_version();
        assert_eq!(spec.version, "0.2.0");

        spec.increment_version();
        assert_eq!(spec.version, "0.3.0");
    }

    // ===== FR-PHENO-DOMAIN-022: Invalid State Transitions Prevented =====
    #[test]
    fn test_invalid_state_transitions() {
        let wp_id = Uuid::new_v4();
        let mut task = Task::new(wp_id, "Task", "desc");

        // Cannot skip from Backlog to InReview
        assert!(task.transition_to(Status::InReview).is_err());

        // Cannot go backward from Todo to Backlog
        assert!(task.transition_to(Status::Todo).is_ok());
        assert!(task.transition_to(Status::Backlog).is_err());
    }

    // ===== FR-PHENO-DOMAIN-023: WorkPackage Cannot Transition from Done =====
    #[test]
    fn test_work_package_done_is_final() {
        let spec_id = Uuid::new_v4();
        let mut wp = WorkPackage::new(spec_id, "WP", "desc");

        wp.transition_to(Status::Todo).ok();
        wp.transition_to(Status::InProgress).ok();
        wp.transition_to(Status::InReview).ok();
        wp.transition_to(Status::Done).ok();

        assert!(wp.transition_to(Status::Cancelled).is_err());
        assert!(wp.transition_to(Status::Backlog).is_err());
    }

    // ===== FR-PHENO-DOMAIN-024: Spec Can Be Cancelled =====
    #[test]
    fn test_spec_can_be_cancelled_early() {
        let project_id = Uuid::new_v4();
        let mut spec = Spec::new(project_id, "Auth", "desc");

        assert!(spec.transition_to(Status::Todo).is_ok());
        assert!(spec.transition_to(Status::Cancelled).is_ok());
        assert_eq!(spec.status, Status::Cancelled);

        // Cannot transition from Cancelled
        assert!(spec.transition_to(Status::Backlog).is_err());
    }

    // ===== FR-PHENO-DOMAIN-025: Task Can Be Cancelled from InProgress =====
    #[test]
    fn test_task_can_be_cancelled_midwork() {
        let wp_id = Uuid::new_v4();
        let mut task = Task::new(wp_id, "Task", "desc");

        task.transition_to(Status::Todo).ok();
        task.transition_to(Status::InProgress).ok();
        assert!(task.transition_to(Status::Cancelled).is_ok());
        assert_eq!(task.status, Status::Cancelled);
    }

    // ===== FR-PHENO-DOMAIN-026: Timestamps Updated on Mutations =====
    #[test]
    fn test_timestamps_updated_on_mutations() {
        let spec_id = Uuid::new_v4();
        let mut wp = WorkPackage::new(spec_id, "WP", "desc");
        let initial_updated = wp.updated_at;

        // Small delay to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        wp.add_task(Uuid::new_v4());
        assert!(wp.updated_at > initial_updated);
    }

    // ===== FR-PHENO-DOMAIN-027: Serialization Roundtrip =====
    #[test]
    fn test_spec_serialization() {
        let project_id = Uuid::new_v4();
        let mut spec = Spec::new(project_id, "Auth", "desc");
        spec.add_acceptance_criterion("Login");
        spec.add_work_package(Uuid::new_v4());

        let json = serde_json::to_string(&spec).expect("should serialize");
        let deserialized: Spec = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(deserialized.id, spec.id);
        assert_eq!(deserialized.acceptance_criteria, spec.acceptance_criteria);
        assert_eq!(deserialized.work_packages, spec.work_packages);
    }

    // ===== FR-PHENO-DOMAIN-028: Multiple Task Dependencies =====
    #[test]
    fn test_task_with_multiple_dependencies() {
        let wp_id = Uuid::new_v4();
        let mut task = Task::new(wp_id, "Integration Test", "desc");
        let dep1 = Uuid::new_v4();
        let dep2 = Uuid::new_v4();
        let dep3 = Uuid::new_v4();

        task.add_dependency(dep1);
        task.add_dependency(dep2);
        task.add_dependency(dep3);

        assert_eq!(task.dependencies.len(), 3);

        // Blocked if any dependency is missing
        assert!(task.is_blocked(&[dep1, dep2]));

        // Not blocked when all dependencies completed
        assert!(!task.is_blocked(&[dep1, dep2, dep3]));
    }

    // ===== FR-PHENO-DOMAIN-029: Spec with Multiple Work Packages =====
    #[test]
    fn test_spec_with_multiple_work_packages() {
        let project_id = Uuid::new_v4();
        let mut spec = Spec::new(project_id, "Platform", "desc");

        let wp1 = Uuid::new_v4();
        let wp2 = Uuid::new_v4();
        let wp3 = Uuid::new_v4();

        spec.add_work_package(wp1);
        spec.add_work_package(wp2);
        spec.add_work_package(wp3);

        assert_eq!(spec.work_packages.len(), 3);
    }

    // ===== FR-PHENO-DOMAIN-030: WorkPackage with Multiple Tasks =====
    #[test]
    fn test_work_package_with_multiple_tasks() {
        let spec_id = Uuid::new_v4();
        let mut wp = WorkPackage::new(spec_id, "WP", "desc");

        let task1 = Uuid::new_v4();
        let task2 = Uuid::new_v4();
        let task3 = Uuid::new_v4();

        wp.add_task(task1);
        wp.add_task(task2);
        wp.add_task(task3);

        assert_eq!(wp.tasks.len(), 3);
        assert!(wp.remove_task(task2));
        assert_eq!(wp.tasks.len(), 2);
    }
}
