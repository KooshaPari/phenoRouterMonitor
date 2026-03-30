//! Configuration loading and validation
//!
//! Loads router configuration from TOML files with validation

use crate::error::{Result, RouterError};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Route configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Service name (e.g., "agileplus", "heliosapp")
    pub service: String,

    /// Path pattern (regex, wildcard, or exact)
    pub path_pattern: String,

    /// Backend addresses
    pub backends: Vec<String>,

    /// Request timeout in milliseconds
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,

    /// Load balancing strategy
    #[serde(default)]
    pub strategy: String,
}

fn default_timeout_ms() -> u64 {
    30000
}

/// Complete router configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Listen address
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,

    /// Listen port
    #[serde(default = "default_listen_port")]
    pub listen_port: u16,

    /// Routes configuration
    pub routes: Vec<RouteConfig>,

    /// Optional max request body size (bytes)
    #[serde(default = "default_max_body_size")]
    pub max_body_size: u64,

    /// Optional request timeout (ms)
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_listen_addr() -> String {
    "127.0.0.1".to_string()
}

fn default_listen_port() -> u16 {
    3030
}

fn default_max_body_size() -> u64 {
    10 * 1024 * 1024 // 10 MB
}

impl RouterConfig {
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.routes.is_empty() {
            return Err(RouterError::ConfigError(
                "At least one route must be configured".to_string(),
            ));
        }

        for route in &self.routes {
            if route.service.is_empty() {
                return Err(RouterError::ConfigError(
                    "Route service name cannot be empty".to_string(),
                ));
            }

            if route.path_pattern.is_empty() {
                return Err(RouterError::ConfigError(
                    "Route path_pattern cannot be empty".to_string(),
                ));
            }

            if route.backends.is_empty() {
                return Err(RouterError::ConfigError(
                    format!("Route {} must have at least one backend", route.service),
                ));
            }

            for backend in &route.backends {
                if !backend.starts_with("http://") && !backend.starts_with("https://") {
                    return Err(RouterError::ConfigError(format!(
                        "Backend URL must start with http:// or https://: {}",
                        backend
                    )));
                }
            }

            if route.timeout_ms == 0 {
                return Err(RouterError::ConfigError(
                    format!("Route {} timeout must be > 0", route.service),
                ));
            }
        }

        Ok(())
    }

    /// Get listen socket address
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.listen_addr, self.listen_port)
    }
}

/// Configuration loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from TOML file
    pub fn from_file(path: impl AsRef<Path>) -> Result<RouterConfig> {
        let content = std::fs::read_to_string(path)?;
        Self::from_string(&content)
    }

    /// Load configuration from TOML string
    pub fn from_string(content: &str) -> Result<RouterConfig> {
        let config: RouterConfig = toml::from_str(content)?;
        config.validate()?;
        Ok(config)
    }

    /// Load configuration from environment
    pub fn from_env() -> Result<RouterConfig> {
        // Try to load from ROUTER_CONFIG_PATH env var
        if let Ok(path) = std::env::var("ROUTER_CONFIG_PATH") {
            return Self::from_file(path);
        }

        // Try default locations
        for path in &["./config.toml", "/etc/router/config.toml", "config.toml"] {
            if Path::new(path).exists() {
                return Self::from_file(path);
            }
        }

        Err(RouterError::ConfigError(
            "No configuration file found. Set ROUTER_CONFIG_PATH or place config.toml in current directory"
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-005 (Configuration loading)
    #[test]
    fn test_route_config_creation() {
        let route = RouteConfig {
            service: "api".to_string(),
            path_pattern: "^/api/.*".to_string(),
            backends: vec!["http://localhost:3000".to_string()],
            timeout_ms: 30000,
            strategy: "roundrobin".to_string(),
        };
        assert_eq!(route.service, "api");
    }

    // Traces to: FR-ROUTER-005
    #[test]
    fn test_router_config_validation_valid() {
        let config = RouterConfig {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 3030,
            routes: vec![RouteConfig {
                service: "api".to_string(),
                path_pattern: "^/api/.*".to_string(),
                backends: vec!["http://localhost:3000".to_string()],
                timeout_ms: 30000,
                strategy: "roundrobin".to_string(),
            }],
            max_body_size: 10 * 1024 * 1024,
            timeout_ms: 30000,
        };
        assert!(config.validate().is_ok());
    }

    // Traces to: FR-ROUTER-005
    #[test]
    fn test_router_config_validation_no_routes() {
        let config = RouterConfig {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 3030,
            routes: vec![],
            max_body_size: 10 * 1024 * 1024,
            timeout_ms: 30000,
        };
        assert!(config.validate().is_err());
    }

    // Traces to: FR-ROUTER-005
    #[test]
    fn test_router_config_validation_no_backends() {
        let config = RouterConfig {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 3030,
            routes: vec![RouteConfig {
                service: "api".to_string(),
                path_pattern: "^/api/.*".to_string(),
                backends: vec![],
                timeout_ms: 30000,
                strategy: "roundrobin".to_string(),
            }],
            max_body_size: 10 * 1024 * 1024,
            timeout_ms: 30000,
        };
        assert!(config.validate().is_err());
    }

    // Traces to: FR-ROUTER-005
    #[test]
    fn test_router_config_validation_invalid_backend_url() {
        let config = RouterConfig {
            listen_addr: "127.0.0.1".to_string(),
            listen_port: 3030,
            routes: vec![RouteConfig {
                service: "api".to_string(),
                path_pattern: "^/api/.*".to_string(),
                backends: vec!["invalid-url".to_string()],
                timeout_ms: 30000,
                strategy: "roundrobin".to_string(),
            }],
            max_body_size: 10 * 1024 * 1024,
            timeout_ms: 30000,
        };
        assert!(config.validate().is_err());
    }

    // Traces to: FR-ROUTER-005
    #[test]
    fn test_config_loader_from_string() {
        let toml = r#"
listen_addr = "0.0.0.0"
listen_port = 3030

[[routes]]
service = "api"
path_pattern = "^/api/.*"
backends = ["http://localhost:3000"]
timeout_ms = 30000
strategy = "roundrobin"
"#;
        let config = ConfigLoader::from_string(toml);
        assert!(config.is_ok());
        let cfg = config.unwrap();
        assert_eq!(cfg.routes.len(), 1);
        assert_eq!(cfg.routes[0].service, "api");
    }

    // Traces to: FR-ROUTER-005
    #[test]
    fn test_config_socket_addr() {
        let config = RouterConfig {
            listen_addr: "0.0.0.0".to_string(),
            listen_port: 8080,
            routes: vec![RouteConfig {
                service: "api".to_string(),
                path_pattern: "^/api/.*".to_string(),
                backends: vec!["http://localhost:3000".to_string()],
                timeout_ms: 30000,
                strategy: "roundrobin".to_string(),
            }],
            max_body_size: 10 * 1024 * 1024,
            timeout_ms: 30000,
        };
        assert_eq!(config.socket_addr(), "0.0.0.0:8080");
    }
}
