//! Subscription handler trait for processing incoming messages.

use async_trait::async_trait;

use crate::bus::EventBusError;
use crate::envelope::Envelope;

/// A handler that processes incoming envelopes on a subscribed subject.
#[async_trait]
pub trait Handler: Send + Sync {
    /// Process one envelope. Return `Ok(())` to acknowledge, or an error to
    /// signal processing failure (the bus may redeliver depending on config).
    async fn handle(&self, envelope: &Envelope) -> Result<(), EventBusError>;
}

/// A simple handler that delegates to a closure. Useful for tests.
pub struct FnHandler<F>(pub F)
where
    F: Fn(&Envelope) -> Result<(), EventBusError> + Send + Sync;

#[async_trait]
impl<F> Handler for FnHandler<F>
where
    F: Fn(&Envelope) -> Result<(), EventBusError> + Send + Sync,
{
    async fn handle(&self, envelope: &Envelope) -> Result<(), EventBusError> {
        (self.0)(envelope)
    }
}
