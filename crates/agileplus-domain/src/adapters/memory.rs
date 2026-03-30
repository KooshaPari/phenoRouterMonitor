//! Thread-safe in-memory stores with [`agileplus_error_core::StorageError`] as the error type.

use std::collections::HashMap;
use std::sync::Arc;

use agileplus_error_core::{NotFoundMarker, StorageError};
use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::entities::{Project, Sprint, WorkItem};
use crate::ports::{ProjectRepository, SprintRepository, WorkItemRepository};

/// In-memory [`ProjectRepository`].
#[derive(Debug, Default, Clone)]
pub struct InMemoryProjectRepository {
    inner: Arc<RwLock<HashMap<Uuid, Project>>>,
}

impl InMemoryProjectRepository {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ProjectRepository for InMemoryProjectRepository {
    type Error = StorageError;

    async fn get(&self, id: &Uuid) -> Result<Option<Project>, Self::Error> {
        let g = self.inner.read().await;
        Ok(g.get(id).cloned())
    }

    async fn save(&self, project: &Project) -> Result<(), Self::Error> {
        let mut g = self.inner.write().await;
        g.insert(project.id, project.clone());
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Project>, Self::Error> {
        let g = self.inner.read().await;
        Ok(g.values().cloned().collect())
    }

    async fn delete(&self, id: &Uuid) -> Result<(), Self::Error> {
        let mut g = self.inner.write().await;
        if g.remove(id).is_none() {
            return Err(StorageError::not_found(id.to_string()));
        }
        Ok(())
    }
}

/// In-memory [`WorkItemRepository`].
#[derive(Debug, Default, Clone)]
pub struct InMemoryWorkItemRepository {
    inner: Arc<RwLock<HashMap<Uuid, WorkItem>>>,
}

impl InMemoryWorkItemRepository {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl WorkItemRepository for InMemoryWorkItemRepository {
    type Error = StorageError;

    async fn get(&self, id: &Uuid) -> Result<Option<WorkItem>, Self::Error> {
        let g = self.inner.read().await;
        Ok(g.get(id).cloned())
    }

    async fn save(&self, item: &WorkItem) -> Result<(), Self::Error> {
        let mut g = self.inner.write().await;
        g.insert(item.id, item.clone());
        Ok(())
    }

    async fn list_by_project(&self, project_id: &Uuid) -> Result<Vec<WorkItem>, Self::Error> {
        let g = self.inner.read().await;
        Ok(g
            .values()
            .filter(|w| w.project_id == *project_id)
            .cloned()
            .collect())
    }

    async fn delete(&self, id: &Uuid) -> Result<(), Self::Error> {
        let mut g = self.inner.write().await;
        if g.remove(id).is_none() {
            return Err(StorageError::not_found(id.to_string()));
        }
        Ok(())
    }
}

/// In-memory [`SprintRepository`].
#[derive(Debug, Default, Clone)]
pub struct InMemorySprintRepository {
    inner: Arc<RwLock<HashMap<Uuid, Sprint>>>,
}

impl InMemorySprintRepository {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SprintRepository for InMemorySprintRepository {
    type Error = StorageError;

    async fn get(&self, id: &Uuid) -> Result<Option<Sprint>, Self::Error> {
        let g = self.inner.read().await;
        Ok(g.get(id).cloned())
    }

    async fn save(&self, sprint: &Sprint) -> Result<(), Self::Error> {
        let mut g = self.inner.write().await;
        g.insert(sprint.id, sprint.clone());
        Ok(())
    }

    async fn list_by_project(&self, project_id: &Uuid) -> Result<Vec<Sprint>, Self::Error> {
        let g = self.inner.read().await;
        Ok(g
            .values()
            .filter(|s| s.project_id == *project_id)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Project;

    #[tokio::test]
    async fn project_round_trip() {
        let repo = InMemoryProjectRepository::new();
        let p = Project::new("Demo", "desc");
        repo.save(&p).await.unwrap();
        let got = repo.get(&p.id).await.unwrap().unwrap();
        assert_eq!(got.name, "Demo");
    }
}
