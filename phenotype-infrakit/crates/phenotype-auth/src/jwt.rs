//! JWT validation utilities

use crate::{AuthError, Claims};
use jsonwebtoken::{decode, DecodingKey, Validation};

/// JWT token validator
#[derive(Debug, Clone)]
pub struct JwtValidator {
    secret: String,
}

impl JwtValidator {
    /// Create a new validator with the given secret
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    /// Validate a JWT token and return claims
    pub fn validate(&self, token: &str) -> Result<Claims, AuthError> {
        let validation = Validation::default();
        let key = DecodingKey::from_secret(self.secret.as_bytes());
        
        decode::<Claims>(token, &key, &validation)
            .map(|data| data.claims)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))
    }

    /// Extract user ID from token without full validation (for logging/debugging)
    pub fn extract_subject(&self, token: &str) -> Option<String> {
        self.validate(token).ok().map(|c| c.sub)
    }
}

impl Default for JwtValidator {
    fn default() -> Self {
        Self::new("default-secret-change-in-production")
    }
}
