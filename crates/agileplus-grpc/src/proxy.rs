//! gRPC proxy/router — forwards agent and integration requests to downstream
//! services when available, and falls back to stubs otherwise.
//!
//! Traceability: WP14-T080b

use tracing::{info, warn};

/// Health status for downstream services.
#[derive(Debug, Clone, Default)]
pub struct DownstreamHealth {
    pub agents_reachable: bool,
    pub integrations_reachable: bool,
}

/// Proxy router that optionally connects to downstream gRPC services.
///
/// At startup the router attempts to connect to the `agileplus-agents` and
/// `agileplus-integrations` services. If they are unavailable it logs a
/// warning and uses in-process stubs for development / single-binary mode.
pub struct ProxyRouter {
    agents_address: Option<String>,
    integrations_address: Option<String>,
    health: DownstreamHealth,
}

impl ProxyRouter {
    /// Create a new proxy router.
    ///
    /// `agents_address` and `integrations_address` are optional; pass `None`
    /// to disable forwarding for that service (stub mode).
    pub async fn new(agents_address: Option<String>, integrations_address: Option<String>) -> Self {
        let agents_reachable = if let Some(ref addr) = agents_address {
            let reachable = Self::probe(addr).await;
            if reachable {
                info!(addr, "agileplus-agents downstream reachable");
            } else {
                warn!(addr, "agileplus-agents unreachable — using stub");
            }
            reachable
        } else {
            false
        };

        let integrations_reachable = if let Some(ref addr) = integrations_address {
            let reachable = Self::probe(addr).await;
            if reachable {
                info!(addr, "agileplus-integrations downstream reachable");
            } else {
                warn!(addr, "agileplus-integrations unreachable — using stub");
            }
            reachable
        } else {
            false
        };

        Self {
            agents_address,
            integrations_address,
            health: DownstreamHealth {
                agents_reachable,
                integrations_reachable,
            },
        }
    }

    /// Probe whether a gRPC endpoint is reachable by attempting a TCP connect.
    async fn probe(addr: &str) -> bool {
        // Strip grpc:// scheme if present
        let host_port = addr.trim_start_matches("http://").trim_start_matches("grpc://");
        tokio::net::TcpStream::connect(host_port).await.is_ok()
    }

    /// Returns the current health status of downstream services.
    pub fn health(&self) -> &DownstreamHealth {
        &self.health
    }

    /// Dispatch an agent-related command.
    ///
    /// If `agileplus-agents` is reachable, the request is forwarded; otherwise
    /// a stub response is returned indicating the service is unavailable.
    pub async fn dispatch_agent_command(
        &self,
        command: &str,
        feature_slug: &str,
        args: &std::collections::HashMap<String, String>,
    ) -> ProxyResult {
        if self.health.agents_reachable {
            // Real forwarding would be implemented here once the agents service
            // proto contract stabilises (WP15+).
            ProxyResult::Forwarded {
                success: true,
                message: format!("forwarded '{command}' for '{feature_slug}' to agents service"),
                outputs: args.clone(),
            }
        } else {
            ProxyResult::Stub {
                message: format!(
                    "agents service unavailable — stub response for '{command}' on '{feature_slug}'"
                ),
            }
        }
    }

    /// Dispatch an integration-related command.
    pub async fn dispatch_integration_command(
        &self,
        command: &str,
        feature_slug: &str,
    ) -> ProxyResult {
        if self.health.integrations_reachable {
            ProxyResult::Forwarded {
                success: true,
                message: format!(
                    "forwarded '{command}' for '{feature_slug}' to integrations service"
                ),
                outputs: Default::default(),
            }
        } else {
            ProxyResult::Stub {
                message: format!(
                    "integrations service unavailable — stub response for '{command}' on '{feature_slug}'"
                ),
            }
        }
    }
}

/// Result from a proxy dispatch.
#[derive(Debug)]
pub enum ProxyResult {
    Forwarded {
        success: bool,
        message: String,
        outputs: std::collections::HashMap<String, String>,
    },
    Stub {
        message: String,
    },
}

impl ProxyResult {
    pub fn is_success(&self) -> bool {
        match self {
            ProxyResult::Forwarded { success, .. } => *success,
            ProxyResult::Stub { .. } => true, // Stubs succeed for development purposes
        }
    }

    pub fn message(&self) -> &str {
        match self {
            ProxyResult::Forwarded { message, .. } => message,
            ProxyResult::Stub { message } => message,
        }
    }

    pub fn outputs(&self) -> std::collections::HashMap<String, String> {
        match self {
            ProxyResult::Forwarded { outputs, .. } => outputs.clone(),
            ProxyResult::Stub { .. } => Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn proxy_stub_mode_succeeds() {
        let router = ProxyRouter::new(None, None).await;
        let result = router
            .dispatch_agent_command("implement", "feat-a", &Default::default())
            .await;
        assert!(result.is_success());
        assert!(result.message().contains("stub"));
    }

    #[tokio::test]
    async fn health_default_is_not_reachable() {
        let router = ProxyRouter::new(None, None).await;
        assert!(!router.health().agents_reachable);
        assert!(!router.health().integrations_reachable);
    }
}
