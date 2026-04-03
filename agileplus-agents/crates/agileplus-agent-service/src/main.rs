//! `agileplus-agent-service` — gRPC server entrypoint (T049b).
//!
//! Binds to `AGENT_SERVICE_ADDR` (default `[::1]:50052`) and serves:
//! - `AgentDispatchService` — agent spawn / status / cancel / review loop
//! - gRPC health check — for readiness probes

use agileplus_agent_dispatch::AgentDispatchAdapter;
use service::proto::agent_dispatch_service_server::AgentDispatchServiceServer;
use service::AgentDispatchServiceImpl;
use std::sync::Arc;
use tonic::transport::Server;
use tonic_health::server::health_reporter;
use tracing::info;

mod service;

// Minimal no-op VCS adapter for startup wiring.
// Real VCS adapter is provided by agileplus-core integration.
mod vcs_noop {
    use agileplus_agent_dispatch::{DomainError, VcsPort};
    use async_trait::async_trait;
    use std::path::PathBuf;

    pub struct NoopVcs;

    #[async_trait]
    impl VcsPort for NoopVcs {
        async fn create_worktree(
            &self,
            feature: &str,
            wp: &str,
        ) -> Result<PathBuf, DomainError> {
            let path = std::env::temp_dir().join(format!("{feature}-{wp}"));
            tokio::fs::create_dir_all(&path).await?;
            Ok(path)
        }

        async fn remove_worktree(&self, path: &PathBuf) -> Result<(), DomainError> {
            let _ = tokio::fs::remove_dir_all(path).await;
            Ok(())
        }

        async fn new_commits_since(
            &self,
            _path: &PathBuf,
            _since: &str,
        ) -> Result<Vec<String>, DomainError> {
            Ok(vec![])
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("agileplus_agent_service=info".parse()?)
                .add_directive("agileplus_agent_dispatch=info".parse()?),
        )
        .init();

    let addr = std::env::var("AGENT_SERVICE_ADDR")
        .unwrap_or_else(|_| "[::1]:50052".to_owned())
        .parse()?;

    // Assemble adapter.
    let vcs = Arc::new(vcs_noop::NoopVcs);
    let adapter = Arc::new(AgentDispatchAdapter::new(vcs));
    let service_impl = AgentDispatchServiceImpl::new(Arc::clone(&adapter));

    // Health check.
    let (mut health_reporter, health_service) = health_reporter();
    health_reporter
        .set_serving::<AgentDispatchServiceServer<AgentDispatchServiceImpl>>()
        .await;

    info!(%addr, "agileplus-agent-service starting");

    Server::builder()
        .add_service(health_service)
        .add_service(AgentDispatchServiceServer::new(service_impl))
        .serve(addr)
        .await?;

    Ok(())
}
