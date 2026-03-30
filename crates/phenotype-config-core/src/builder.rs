//! Config builder module

use crate::{Config, ConfigError, ConfigSource};

/// Builder for constructing Config with multiple sources
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    sources: Vec<ConfigSource>,
}

impl ConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a configuration source
    pub fn with_source(mut self, source: ConfigSource) -> Self {
        self.sources.push(source);
        self
    }

    /// Add a file source
    pub fn with_file<P: Into<String>>(mut self, path: P) -> Self {
        self.sources.push(ConfigSource::File(path.into()));
        self
    }

    /// Add an environment source
    pub fn with_env_prefix<P: Into<String>>(mut self, prefix: P) -> Self {
        self.sources.push(ConfigSource::Env(prefix.into()));
        self
    }

    /// Add inline config
    pub fn with_inline(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.sources.push(ConfigSource::Inline(key.into(), value));
        self
    }

    /// Build the final configuration
    pub fn build(self) -> Result<Config> {
        let mut config = Config::new();

        for source in self.sources {
            let values = source.load()?;
            for (key, value) in values {
                config.data.insert(key, value);
            }
        }

        Ok(config)
    }
}
