//! Inbound (driving) port traits.
//!
//! These define the use-case boundary that application services expose
//! to driving adapters (HTTP handlers, CLI, gRPC, etc.).

use async_trait::async_trait;

/// Handles a command that mutates state and returns a result.
#[async_trait]
pub trait CommandHandler<C, R>
where
    C: Send + 'static,
    R: Send + 'static,
{
    type Error: std::error::Error + Send + Sync + 'static;

    async fn handle(&self, cmd: C) -> Result<R, Self::Error>;
}

/// Handles a read-only query and returns a result.
#[async_trait]
pub trait QueryHandler<Q, R>
where
    Q: Send + 'static,
    R: Send + 'static,
{
    type Error: std::error::Error + Send + Sync + 'static;

    async fn handle(&self, query: Q) -> Result<R, Self::Error>;
}
