use std::sync::Arc;

use tonic::Status;

use agileplus_domain::ports::{AgentPort, ObservabilityPort, ReviewPort, StoragePort, VcsPort};

use crate::event_bus::EventBus;
use crate::proxy::ProxyRouter;

/// Map domain errors to gRPC Status codes consistently.
pub fn domain_error_to_status(e: agileplus_domain::error::DomainError) -> Status {
    use agileplus_domain::error::DomainError;

    match e {
        DomainError::NotFound(msg) => Status::not_found(msg),
        DomainError::InvalidTransition { from, to, reason } => {
            Status::failed_precondition(format!("invalid transition {from}->{to}: {reason}"))
        }
        DomainError::NoOpTransition(state) => {
            Status::failed_precondition(format!("already in state: {state}"))
        }
        DomainError::Conflict(msg) => Status::already_exists(msg),
        DomainError::Timeout(secs) => Status::deadline_exceeded(format!("timeout after {secs}s")),
        DomainError::NotImplemented => Status::unimplemented("not implemented"),
        other => Status::internal(other.to_string()),
    }
}

/// gRPC server struct holding references to all port implementations.
#[derive(Clone)]
pub struct AgilePlusCoreServer<S, V, A, R, O>
where
    S: StoragePort + 'static,
    V: VcsPort + 'static,
    A: AgentPort + 'static,
    R: ReviewPort + 'static,
    O: ObservabilityPort + 'static,
{
    pub(super) storage: Arc<S>,
    #[allow(dead_code)]
    pub(super) vcs: Arc<V>,
    #[allow(dead_code)]
    pub(super) agents: Arc<A>,
    #[allow(dead_code)]
    pub(super) review: Arc<R>,
    #[allow(dead_code)]
    pub(super) telemetry: Arc<O>,
    pub(super) event_bus: Arc<EventBus>,
    pub(super) proxy: Arc<ProxyRouter>,
}

impl<S, V, A, R, O> AgilePlusCoreServer<S, V, A, R, O>
where
    S: StoragePort + 'static,
    V: VcsPort + 'static,
    A: AgentPort + 'static,
    R: ReviewPort + 'static,
    O: ObservabilityPort + 'static,
{
    pub fn new(
        storage: Arc<S>,
        vcs: Arc<V>,
        agents: Arc<A>,
        review: Arc<R>,
        telemetry: Arc<O>,
        event_bus: Arc<EventBus>,
        proxy: Arc<ProxyRouter>,
    ) -> Self {
        Self {
            storage,
            vcs,
            agents,
            review,
            telemetry,
            event_bus,
            proxy,
        }
    }
}
