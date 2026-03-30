/// Test suite for SerdeHelper derive macro
use phenotype_macros::SerdeHelper;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, SerdeHelper, Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub timeout_secs: u32,
}

#[test]
fn test_serde_helper_from_json() {
    let json = r#"{"host":"localhost","port":8080,"timeout_secs":30}"#;
    let config = AppConfig::from_json(json);
    assert!(config.is_ok());
    let config = config.unwrap();
    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 8080);
}

#[test]
fn test_serde_helper_to_json() {
    let config = AppConfig {
        host: "example.com".to_string(),
        port: 443,
        timeout_secs: 60,
    };

    let json = config.to_json();
    assert!(json.is_ok());
    let json = json.unwrap();
    assert!(json.contains("example.com"));
    assert!(json.contains("443"));
}

#[test]
fn test_serde_helper_to_json_pretty() {
    let config = AppConfig {
        host: "api.example.com".to_string(),
        port: 3000,
        timeout_secs: 45,
    };

    let json = config.to_json_pretty();
    assert!(json.is_ok());
    let json = json.unwrap();
    // Pretty printed JSON should have newlines
    assert!(json.contains('\n'));
}

#[derive(Serialize, Deserialize, SerdeHelper, Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    pub ssl_enabled: bool,
}

#[test]
fn test_serde_helper_round_trip() {
    let original = DatabaseConfig {
        url: "postgresql://localhost/mydb".to_string(),
        pool_size: 20,
        ssl_enabled: true,
    };

    let json = original.to_json().unwrap();
    let restored = DatabaseConfig::from_json(&json).unwrap();

    assert_eq!(original.url, restored.url);
    assert_eq!(original.pool_size, restored.pool_size);
    assert_eq!(original.ssl_enabled, restored.ssl_enabled);
}

#[test]
fn test_serde_helper_invalid_json() {
    let invalid_json = r#"{"host":"localhost"#; // Incomplete JSON
    let result = AppConfig::from_json(invalid_json);
    assert!(result.is_err());
}

#[derive(Serialize, Deserialize, SerdeHelper, Debug)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file_path: String,
}

#[test]
fn test_serde_helper_default_config() {
    let config = LoggingConfig {
        level: "INFO".to_string(),
        format: "json".to_string(),
        file_path: "/var/log/app.log".to_string(),
    };
    let json = config.to_json().unwrap();
    assert!(json.contains("INFO"));
}

#[derive(Serialize, Deserialize, SerdeHelper, Debug)]
pub struct SecurityConfig {
    pub api_key: String,
    pub jwt_secret: String,
    pub cors_enabled: bool,
}

#[test]
fn test_serde_helper_sensitive_data() {
    let config = SecurityConfig {
        api_key: "sk_live_xxxxxx".to_string(),
        jwt_secret: "secret_xxxxxx".to_string(),
        cors_enabled: true,
    };

    let json = config.to_json().unwrap();
    // Verify sensitive data is in the JSON (serialization works)
    assert!(json.contains("api_key"));
    assert!(json.contains("sk_live_xxxxxx"));
}

#[test]
fn test_serde_helper_empty_struct() {
    #[derive(Serialize, Deserialize, SerdeHelper, Debug)]
    pub struct EmptyConfig {}

    let config = EmptyConfig {};
    let json = config.to_json().unwrap();
    assert_eq!(json, "{}");

    let restored = EmptyConfig::from_json(&json).unwrap();
    // Just verify it works
    let _: EmptyConfig = restored;
}

#[test]
fn test_serde_helper_nested_json() {
    let config = AppConfig {
        host: "nested.example.com".to_string(),
        port: 5432,
        timeout_secs: 120,
    };

    let json = config.to_json_pretty().unwrap();
    // Verify pretty formatting produces readable output
    assert!(json.len() > config.to_json().unwrap().len());
}
