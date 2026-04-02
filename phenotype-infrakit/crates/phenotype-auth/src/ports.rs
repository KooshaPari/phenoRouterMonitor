use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Port for user storage.
#[async_trait]
pub trait UserStorage: Send + Sync {
    async fn create(&self, user: &User) -> Result<(), String>;
    async fn get_by_id(&self, id: &str) -> Result<Option<User>, String>;
    async fn get_by_email(&self, email: &str) -> Result<Option<User>, String>;
}

/// Minimal User entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
}
