//! Router configuration types and structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Router configuration loaded from TOML
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouterConfig {
    /// Router name
    pub name: String,

    /// Server configuration
    pub server: ServerConfig,

    /// Route definitions
    pub routes: Vec<RouteConfig>,

    /// Optional middleware settings
    #[serde(default)]
    pub middleware: MiddlewareConfig,

    /// Optional additional settings
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerConfig {
    /// Server host/bind address
    pub host: String,

    /// Server port
    pub port: u16,

    /// Server timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Max concurrent connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

/// Route configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteConfig {
    /// Route path/pattern
    pub path: String,

    /// HTTP method (GET, POST, etc.)
    #[serde(default = "default_method")]
    pub method: String,

    /// Route handler/target
    pub handler: String,

    /// Enable CORS for this route
    #[serde(default)]
    pub cors_enabled: bool,

    /// Route-specific timeout in seconds
    #[serde(default)]
    pub timeout_secs: Option<u64>,

    /// Route metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Middleware configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MiddlewareConfig {
    /// Enable request logging
    #[serde(default)]
    pub logging_enabled: bool,

    /// Enable authentication middleware
    #[serde(default)]
    pub auth_enabled: bool,

    /// Enable CORS globally
    #[serde(default)]
    pub cors_enabled: bool,

    /// Compression settings
    #[serde(default)]
    pub compression: Option<CompressionConfig>,
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompressionConfig {
    /// Enable compression
    pub enabled: bool,

    /// Compression level (1-9)
    #[serde(default = "default_compression_level")]
    pub level: u32,
}

fn default_timeout() -> u64 {
    30
}

fn default_max_connections() -> u32 {
    1000
}

fn default_method() -> String {
    "GET".to_string()
}

fn default_compression_level() -> u32 {
    6
}

impl RouterConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate server
        if self.port == 0 {
            return Err("Port must be greater than 0".to_string());
        }

        if self.host.is_empty() {
            return Err("Host cannot be empty".to_string());
        }

        if self.timeout_secs == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        if self.max_connections == 0 {
            return Err("Max connections must be greater than 0".to_string());
        }

        // Validate routes
        if self.routes.is_empty() {
            return Err("At least one route must be defined".to_string());
        }

        for (idx, route) in self.routes.iter().enumerate() {
            if route.path.is_empty() {
                return Err(format!("Route {} path cannot be empty", idx));
            }

            if route.handler.is_empty() {
                return Err(format!("Route {} handler cannot be empty", idx));
            }

            if !route.method.is_empty() {
                let valid_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
                if !valid_methods.contains(&route.method.as_str()) {
                    return Err(format!("Route {} has invalid HTTP method: {}", idx, route.method));
                }
            }
        }

        // Validate middleware compression
        if let Some(compression) = &self.middleware.compression {
            if compression.level < 1 || compression.level > 9 {
                return Err("Compression level must be between 1 and 9".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> RouterConfig {
        RouterConfig {
            name: "test-router".to_string(),
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                timeout_secs: 30,
                max_connections: 1000,
            },
            routes: vec![RouteConfig {
                path: "/api/users".to_string(),
                method: "GET".to_string(),
                handler: "user_handler".to_string(),
                cors_enabled: false,
                timeout_secs: None,
                metadata: HashMap::new(),
            }],
            middleware: MiddlewareConfig::default(),
            extra: HashMap::new(),
        }
    }

    #[test]
    fn test_valid_config() {
        let config = create_test_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_port_zero() {
        let mut config = create_test_config();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_empty_host() {
        let mut config = create_test_config();
        config.server.host = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_zero_timeout() {
        let mut config = create_test_config();
        config.server.timeout_secs = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_zero_max_connections() {
        let mut config = create_test_config();
        config.server.max_connections = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_routes() {
        let mut config = create_test_config();
        config.routes.clear();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_route_path_empty() {
        let mut config = create_test_config();
        config.routes[0].path = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_route_handler_empty() {
        let mut config = create_test_config();
        config.routes[0].handler = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_http_method() {
        let mut config = create_test_config();
        config.routes[0].method = "INVALID".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_compression_level_low() {
        let mut config = create_test_config();
        config.middleware.compression = Some(CompressionConfig {
            enabled: true,
            level: 0,
        });
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_compression_level_high() {
        let mut config = create_test_config();
        config.middleware.compression = Some(CompressionConfig {
            enabled: true,
            level: 10,
        });
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_valid_compression_level() {
        let mut config = create_test_config();
        config.middleware.compression = Some(CompressionConfig {
            enabled: true,
            level: 6,
        });
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_values() {
        let server = ServerConfig {
            host: "localhost".to_string(),
            port: 8080,
            timeout_secs: default_timeout(),
            max_connections: default_max_connections(),
        };
        assert_eq!(server.timeout_secs, 30);
        assert_eq!(server.max_connections, 1000);
    }
}
