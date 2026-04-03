//! # Phenotype Authentication & Authorization
//!
//! Shared authentication abstractions including:
//! - JWT validation
//! - Session management
//! - RBAC permissions
//! - Auth middleware hooks

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Core authentication errors
#[derive(Error, Debug, Clone)]
pub enum AuthError {
    #[error("invalid token: {0}")]
    InvalidToken(String),
    #[error("expired token")]
    ExpiredToken,
    #[error("insufficient permissions")]
    InsufficientPermissions,
    #[error("session not found")]
    SessionNotFound,
}

/// Port trait for user storage (driven port)
#[async_trait]
pub trait UserStorage: Send + Sync {
    async fn find_by_id(&self, user_id: &str) -> Result<Option<User>, AuthError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AuthError>;
    async fn store(&self, user: &User) -> Result<(), AuthError>;
}

/// Port trait for session storage (driven port)
#[async_trait]
pub trait SessionStorage: Send + Sync {
    async fn get_session(&self, session_id: &str) -> Result<Option<Session>, AuthError>;
    async fn store_session(&self, session: &Session) -> Result<(), AuthError>;
    async fn delete_session(&self, session_id: &str) -> Result<(), AuthError>;
}

/// Port trait for password hashing (driven port)
pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> String;
    fn verify(&self, password: &str, hash: &str) -> bool;
}

/// User domain model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub roles: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Session domain model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub expires_at: u64,
}

/// JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub roles: Vec<String>,
}

pub mod jwt;
pub mod middleware;
pub mod session;

pub use jwt::JwtValidator;
pub use middleware::AuthMiddleware;
pub use session::SessionManager;
