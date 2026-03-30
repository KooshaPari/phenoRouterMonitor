//! Port interfaces for AgilePlus domain.

use async_trait::async_trait;
use uuid::Uuid;

use crate::entities::{Project, Sprint, WorkItem};

/// Project repository port.
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn get(&self, id: &Uuid) -> Result<Option<Project>, Self::Error>;
    async fn save(&self, project: &Project) -> Result<(), Self::Error>;
    async fn list(&self) -> Result<Vec<Project>, Self::Error>;
    async fn delete(&self, id: &Uuid) -> Result<(), Self::Error>;
}

/// Work item repository port.
#[async_trait]
pub trait WorkItemRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn get(&self, id: &Uuid) -> Result<Option<WorkItem>, Self::Error>;
    async fn save(&self, item: &WorkItem) -> Result<(), Self::Error>;
    async fn list_by_project(&self, project_id: &Uuid) -> Result<Vec<WorkItem>, Self::Error>;
    async fn delete(&self, id: &Uuid) -> Result<(), Self::Error>;
}

/// Sprint repository port.
#[async_trait]
pub trait SprintRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn get(&self, id: &Uuid) -> Result<Option<Sprint>, Self::Error>;
    async fn save(&self, sprint: &Sprint) -> Result<(), Self::Error>;
    async fn list_by_project(&self, project_id: &Uuid) -> Result<Vec<Sprint>, Self::Error>;
}
