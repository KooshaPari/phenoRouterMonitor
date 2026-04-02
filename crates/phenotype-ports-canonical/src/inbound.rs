//! Inbound ports (driving adapters) for hexagonal architecture

use async_trait::async_trait;

pub trait Command: Send + Sync {
    type Output;
    fn validate(&self) -> Result<(), crate::PortError>;
}

pub trait Query: Send + Sync {
    type Output;
}

pub trait UseCaseInput: Send + Sync {}
pub trait UseCaseOutput: Send + Sync {}

#[async_trait]
pub trait UseCase<I: UseCaseInput, O: UseCaseOutput>: Send + Sync {
    async fn execute(&self, input: I) -> Result<O, crate::PortError>;
}

#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    async fn handle(&self, command: C) -> Result<C::Output, crate::PortError>;
}

#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
    async fn handle(&self, query: Q) -> Result<Q::Output, crate::PortError>;
}

#[async_trait]
pub trait EventHandler<E: crate::DomainEvent>: Send + Sync {
    async fn handle(&self, event: &E) -> Result<(), crate::PortError>;
}
