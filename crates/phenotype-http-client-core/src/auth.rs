//! HTTP authentication middleware.

use http::{Request, Response, header::HeaderName, header::HeaderValue};
use std::sync::Arc;

/// Authentication credentials.
#[derive(Debug, Clone)]
pub enum AuthCredentials {
    /// Bearer token authentication.
    Bearer(String),
    /// API key authentication.
    ApiKey {
        key: String,
        value: String,
        location: ApiKeyLocation,
    },
    /// Basic authentication.
    Basic { username: String, password: String },
}

/// Where to place the API key.
#[derive(Debug, Clone, Copy)]
pub enum ApiKeyLocation {
    /// In the Authorization header.
    Authorization,
    /// In a custom header.
    Header(HeaderName),
    /// As a query parameter.
    QueryParam(String),
}

impl AuthCredentials {
    /// Create bearer token credentials.
    pub fn bearer(token: impl Into<String>) -> Self {
        Self::Bearer(token.into())
    }

    /// Create API key credentials.
    pub fn api_key(key: impl Into<String>, value: impl Into<String>, location: ApiKeyLocation) -> Self {
        Self::ApiKey {
            key: key.into(),
            value: value.into(),
            location,
        }
    }

    /// Create basic auth credentials.
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::Basic {
            username: username.into(),
            password: password.into(),
        }
    }
}

/// Authentication middleware.
#[derive(Debug, Clone)]
pub struct AuthMiddleware {
    credentials: Arc<AuthCredentials>,
}

impl AuthMiddleware {
    /// Create new auth middleware.
    pub fn new(credentials: AuthCredentials) -> Self {
        Self {
            credentials: Arc::new(credentials),
        }
    }

    /// Apply authentication to a request.
    pub fn apply<B>(&self, mut request: Request<B>) -> Result<Response<Vec<u8>>, crate::error::TransportError> {
        match self.credentials.as_ref() {
            AuthCredentials::Bearer(token) => {
                let header_value = HeaderValue::from_str(&format!("Bearer {}", token))
                    .map_err(|e| crate::error::TransportError::Auth(format!("Invalid bearer token: {}", e)))?;
                request.headers_mut().insert(header::AUTHORIZATION, header_value);
            }
            AuthCredentials::ApiKey { key, value, location } => {
                let header_name = match location {
                    ApiKeyLocation::Authorization => header::AUTHORIZATION,
                    ApiKeyLocation::Header(name) => *name,
                    ApiKeyLocation::QueryParam(_) => {
                        return Err(crate::error::TransportError::Auth(
                            "Query param auth not supported in middleware".into(),
                        ));
                    }
                };

                let header_value = HeaderValue::from_str(&value)
                    .map_err(|e| crate::error::TransportError::Auth(format!("Invalid API key: {}", e)))?;
                request.headers_mut().insert(header_name, header_value);
            }
            AuthCredentials::Basic { username, password } => {
                let credentials = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    format!("{}:{}", username, password),
                );
                let header_value = HeaderValue::from_str(&format!("Basic {}", credentials))
                    .map_err(|e| crate::error::TransportError::Auth(format!("Invalid basic auth: {}", e)))?;
                request.headers_mut().insert(header::AUTHORIZATION, header_value);
            }
        }

        Ok(Response::new(Vec::new()))
    }
}

/// Convenience extension trait for adding auth to requests.
pub trait RequestAuthExt {
    /// Add bearer authentication.
    fn bearer_auth(self, token: impl Into<String>) -> Self;

    /// Add API key authentication.
    fn api_key_auth(self, key: impl Into<String>, value: impl Into<String>) -> Self;
}

impl<B> RequestAuthExt for Request<B> {
    fn bearer_auth(mut self, token: impl Into<String>) -> Self {
        let header_value = HeaderValue::from_str(&format!("Bearer {}", token.into())).unwrap();
        self.headers_mut().insert(header::AUTHORIZATION, header_value);
        self
    }

    fn api_key_auth(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let header_name = HeaderName::from_bytes(key.into().as_bytes()).unwrap();
        let header_value = HeaderValue::from_str(&value.into()).unwrap();
        self.headers_mut().insert(header_name, header_value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bearer_auth() {
        let creds = AuthCredentials::bearer("my-token");
        match creds {
            AuthCredentials::Bearer(token) => assert_eq!(token, "my-token"),
            _ => panic!("Expected Bearer variant"),
        }
    }

    #[test]
    fn test_api_key_auth() {
        let creds = AuthCredentials::api_key("X-API-Key", "secret-key", ApiKeyLocation::Authorization);
        match creds {
            AuthCredentials::ApiKey { key, value, location } => {
                assert_eq!(key, "X-API-Key");
                assert_eq!(value, "secret-key");
                matches!(location, ApiKeyLocation::Authorization);
            }
            _ => panic!("Expected ApiKey variant"),
        }
    }
}
