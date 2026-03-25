use std::net::SocketAddr;
use std::sync::Arc;

use tonic::transport::Server;
use tracing::info;

use agileplus_domain::ports::{AgentPort, ObservabilityPort, ReviewPort, StoragePort, VcsPort};
use agileplus_proto::agileplus::v1::{
    agile_plus_core_service_server::AgilePlusCoreServiceServer,
    integrations_service_server::IntegrationsServiceServer,
};

use super::AgilePlusCoreServer;
use crate::event_bus::EventBus;
use crate::proxy::ProxyRouter;

/// Start the gRPC server, binding to the given address.
#[allow(clippy::too_many_arguments)] // Server bootstrap requires all service ports
pub async fn start_server<S, V, A, R, O>(
    addr: SocketAddr,
    storage: Arc<S>,
    vcs: Arc<V>,
    agents: Arc<A>,
    review: Arc<R>,
    telemetry: Arc<O>,
    event_bus: Arc<EventBus>,
    proxy: Arc<ProxyRouter>,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: StoragePort + 'static,
    V: VcsPort + 'static,
    A: AgentPort + 'static,
    R: ReviewPort + 'static,
    O: ObservabilityPort + 'static,
{
    let service =
        AgilePlusCoreServer::new(storage, vcs, agents, review, telemetry, event_bus, proxy);

    info!(%addr, "starting AgilePlus gRPC server");

    Server::builder()
        .add_service(AgilePlusCoreServiceServer::new(service.clone()))
        .add_service(IntegrationsServiceServer::new(service))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    info!("gRPC server shut down gracefully");
    Ok(())
}

/// Listens for SIGTERM / SIGINT and resolves when either is received.
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutdown signal received");
}
