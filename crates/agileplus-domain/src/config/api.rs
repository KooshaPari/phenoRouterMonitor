use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiConfig {
    #[serde(default = "default_api_port")]
    pub port: u16,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default)]
    pub cors_origins: Vec<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            port: default_api_port(),
            grpc_port: default_grpc_port(),
            cors_origins: Vec::new(),
        }
    }
}

fn default_api_port() -> u16 {
    3000
}

fn default_grpc_port() -> u16 {
    50051
}
