//! Cache configuration.

#[derive(Clone, Debug)]
pub struct CacheConfig {
    pub host: String,
    pub port: u16,
    pub pool_size: u32,
    pub default_ttl_secs: u64,
    pub connection_timeout_secs: u64,
}

impl CacheConfig {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            pool_size: 16,
            default_ttl_secs: 3600,
            connection_timeout_secs: 5,
        }
    }

    pub fn with_pool_size(mut self, size: u32) -> Self {
        self.pool_size = size;
        self
    }

    pub fn with_default_ttl(mut self, secs: u64) -> Self {
        self.default_ttl_secs = secs;
        self
    }

    pub fn redis_url(&self) -> String {
        format!("redis://{}:{}", self.host, self.port)
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self::new("localhost".into(), 6379)
    }
}
