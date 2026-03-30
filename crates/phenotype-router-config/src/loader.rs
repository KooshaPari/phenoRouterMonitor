//! Configuration loader for TOML files

use crate::config::RouterConfig;
use crate::error::{Result, RouterConfigError};
use std::path::Path;
use tokio::fs;
use tracing::{debug, info};

/// Configuration loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from a TOML file
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> Result<RouterConfig> {
        let path = path.as_ref();
        debug!("Loading configuration from: {:?}", path);

        // Read file content
        let content = fs::read_to_string(path).await?;

        // Parse TOML
        let config = toml::from_str::<RouterConfig>(&content)?;

        // Validate configuration
        config.validate().map_err(RouterConfigError::ValidationError)?;

        info!("Successfully loaded configuration from: {:?}", path);
        Ok(config)
    }

    /// Load configuration from TOML string
    pub fn load_from_str(content: &str) -> Result<RouterConfig> {
        debug!("Loading configuration from string");

        // Parse TOML
        let config = toml::from_str::<RouterConfig>(content)?;

        // Validate configuration
        config.validate().map_err(RouterConfigError::ValidationError)?;

        info!("Successfully loaded configuration from string");
        Ok(config)
    }

    /// Load configuration from TOML string without validation
    pub fn load_from_str_unvalidated(content: &str) -> Result<RouterConfig> {
        debug!("Loading configuration from string (unvalidated)");
        toml::from_str::<RouterConfig>(content).map_err(|e| RouterConfigError::from(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_load_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
name = "test-router"

[server]
host = "127.0.0.1"
port = 8080
timeout_secs = 30
max_connections = 1000

[[routes]]
path = "/api/users"
method = "GET"
handler = "user_handler"
cors_enabled = false
"#;
        temp_file.write_all(config_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = ConfigLoader::load_from_file(temp_file.path()).await;
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.name, "test-router");
        assert_eq!(config.server.port, 8080);
    }

    #[test]
    fn test_load_from_str() {
        let config_content = r#"
name = "test-router"

[server]
host = "127.0.0.1"
port = 8080
timeout_secs = 30
max_connections = 1000

[[routes]]
path = "/api/users"
method = "GET"
handler = "user_handler"
cors_enabled = false
"#;

        let config = ConfigLoader::load_from_str(config_content);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.name, "test-router");
        assert_eq!(config.server.host, "127.0.0.1");
    }

    #[test]
    fn test_load_from_str_invalid_toml() {
        let invalid_content = r#"
name = "test
server
port = 8080
"#;
        let config = ConfigLoader::load_from_str(invalid_content);
        assert!(config.is_err());
    }

    #[test]
    fn test_load_from_str_validation_failure() {
        let config_content = r#"
name = "test-router"

[server]
host = "127.0.0.1"
port = 0
timeout_secs = 30
max_connections = 1000

[[routes]]
path = "/api/users"
method = "GET"
handler = "user_handler"
cors_enabled = false
"#;
        let config = ConfigLoader::load_from_str(config_content);
        assert!(config.is_err());
    }

    #[test]
    fn test_load_from_str_unvalidated() {
        let config_content = r#"
name = "test-router"

[server]
host = "127.0.0.1"
port = 0
timeout_secs = 30
max_connections = 1000

[[routes]]
path = "/api/users"
method = "GET"
handler = "user_handler"
cors_enabled = false
"#;
        let config = ConfigLoader::load_from_str_unvalidated(config_content);
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.server.port, 0);
    }
}
