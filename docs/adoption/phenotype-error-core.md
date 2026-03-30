# Adopting phenotype-error-core

Centralized error handling for the Phenotype ecosystem.

## Quick Start

```rust
use phenotype_error_core::{ApiError, DomainError, RepositoryError};

// Use built-in error types
pub fn fetch_user(id: UserId) -> Result<User, ApiError> {
    find_user(id)?
        .ok_or_else(|| ApiError::NotFound(format!("User {} not found", id)))
}

// Implement your own errors
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Validation: {0}")]
    Validation(String),
    
    #[error(transparent)]
    Database(#[from] RepositoryError),
    
    #[error(transparent)]
    Domain(#[from] DomainError),
}
```

## Error Conversion

```rust
use phenotype_error_core::prelude::*;

fn fallible_operation() -> Result<Value, ApiError> {
    external_api_call()
        .map_err(|e| ApiError::External(e.to_string()))
        .context("Calling external API")?
}
```

## Feature Flags

```toml
phenotype-error-core = { version = "0.1", features = ["full"] }
```

## Related Crates

- `phenotype-error-core` - Error types
- `phenotype-logging` - Error logging integration
- `phenotype-api-types` - API error responses
