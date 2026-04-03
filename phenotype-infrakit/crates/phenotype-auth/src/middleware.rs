//! Authentication middleware traits

use crate::{AuthError, User};

/// Middleware trait for authenticating requests
pub trait AuthMiddleware: Send + Sync {
    type Request;
    type Response;
    type Error;

    /// Authenticate a request and return the user or an error
    fn authenticate(&self, request: &Self::Request) -> Result<User, Self::Error>;
}
