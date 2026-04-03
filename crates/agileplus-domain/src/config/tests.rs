use std::env;

use super::*;

#[test]
fn default_config_validates() {
    AppConfig::default().validate().unwrap();
}

#[test]
fn toml_roundtrip() {
    let config = AppConfig::default();
    let toml_str = toml::to_string_pretty(&config).unwrap();
    let parsed: AppConfig = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.api.port, 3000);
    assert_eq!(parsed.api.grpc_port, 50051);
    assert_eq!(parsed.agents.max_subagents, 3);
}

#[test]
fn partial_toml_uses_defaults() {
    let partial = r#"
[api]
port = 9090
"#;
    let config: AppConfig = toml::from_str(partial).unwrap();
    assert_eq!(config.api.port, 9090);
    assert_eq!(config.api.grpc_port, 50051);
    assert_eq!(config.agents.max_subagents, 3);
}

#[test]
fn invalid_log_level_fails_validation() {
    let bad = r#"
[telemetry]
log_level = "verbose"
"#;
    let config: AppConfig = toml::from_str(bad).unwrap();
    assert!(config.validate().is_err());
}

#[test]
fn env_override_api_port() {
    unsafe {
        env::set_var("AGILEPLUS_API_PORT", "9999");
    }
    let port: u16 = env::var("AGILEPLUS_API_PORT").unwrap().parse().unwrap();
    assert_eq!(port, 9999);
    unsafe {
        env::remove_var("AGILEPLUS_API_PORT");
    }
}
