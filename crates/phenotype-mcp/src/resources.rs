//! Phenotype MCP Resources Module
//!
//! Provides resource management for MCP server.

use fastmcp::Resource;

/// Available resources
pub const RESOURCES: &[(&str, &str, &str)] = &[
    ("phenotype://version", "Phenotype version information", "json"),
    ("phenotype://config", "Current configuration", "json"),
    ("phenotype://health", "Health status", "json"),
    ("phenotype://features", "Feature registry", "json"),
    ("phenotype://agents", "Available agents", "json"),
];

/// Get a resource by URI
pub async fn get_resource(uri: &str) -> Option<String> {
    match uri {
        "phenotype://version" => Some(serde_json::json!({
            "name": "phenotype-infrakit",
            "version": env!("CARGO_PKG_VERSION"),
            "mcp_version": "3.0"
        }).to_string()),
        "phenotype://config" => Some(serde_json::json!({
            "environment": std::env::var("PHENOTYPE_ENV").unwrap_or_else(|_| "development".to_string()),
            "log_level": std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        }).to_string()),
        "phenotype://health" => Some(serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }).to_string()),
        _ => None,
    }
}
