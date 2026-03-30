//! Integration tests for phenotype-config-core

use phenotype_config_core::{ConfigBuilder, ConfigFormat, ConfigValidator};
use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_file_loader_json() -> phenotype_config_core::Result<()> {
    let mut file = NamedTempFile::new()?;
    writeln!(file, r#"{{"port": 3000, "debug": true}}"#)?;

    let config = ConfigBuilder::new()
        .with_file(file.path())
        .build()?;

    assert_eq!(config.get_i64("port"), Some(3000));
    assert_eq!(config.get_bool("debug"), Some(true));
    Ok(())
}

#[test]
fn test_file_loader_toml() -> phenotype_config_core::Result<()> {
    let mut file = NamedTempFile::new()?;
    writeln!(file, "port = 3000\ndebug = true")?;
    let path = file.path().with_extension("toml");
    let mut file = std::fs::File::create(&path)?;
    writeln!(file, "port = 3000\ndebug = true")?;

    let config = ConfigBuilder::new()
        .with_file(&path)
        .build()?;

    assert_eq!(config.get_i64("port"), Some(3000));
    assert_eq!(config.get_bool("debug"), Some(true));

    std::fs::remove_file(&path)?;
    Ok(())
}

#[test]
fn test_file_loader_yaml() -> phenotype_config_core::Result<()> {
    let mut file = NamedTempFile::new()?;
    writeln!(file, "port: 3000\ndebug: true")?;
    let path = file.path().with_extension("yaml");
    let mut file = std::fs::File::create(&path)?;
    writeln!(file, "port: 3000\ndebug: true")?;

    let config = ConfigBuilder::new()
        .with_file(&path)
        .build()?;

    assert_eq!(config.get_i64("port"), Some(3000));
    assert_eq!(config.get_bool("debug"), Some(true));

    std::fs::remove_file(&path)?;
    Ok(())
}

#[test]
fn test_inline_loader() -> phenotype_config_core::Result<()> {
    let config = ConfigBuilder::new()
        .with_inline_value("port", json!(3000))
        .with_inline_value("debug", json!(true))
        .with_inline_value("name", json!("app"))
        .build()?;

    assert_eq!(config.get_i64("port"), Some(3000));
    assert_eq!(config.get_bool("debug"), Some(true));
    assert_eq!(config.get_string("name"), Some("app".to_string()));
    Ok(())
}

#[test]
fn test_override_chain() -> phenotype_config_core::Result<()> {
    // Test that later sources override earlier ones
    let config = ConfigBuilder::new()
        .with_inline_value("port", json!(2000))
        .with_inline_value("debug", json!(false))
        .with_inline_value("port", json!(3000)) // Override
        .build()?;

    assert_eq!(config.get_i64("port"), Some(3000));
    assert_eq!(config.get_bool("debug"), Some(false));
    Ok(())
}

#[test]
fn test_config_access_methods() -> phenotype_config_core::Result<()> {
    let config = ConfigBuilder::new()
        .with_inline_value("port", json!(3000))
        .with_inline_value("debug", json!(true))
        .with_inline_value("name", json!("app"))
        .build()?;

    // Test get_string_required
    assert_eq!(config.get_string_required("name")?, "app".to_string());

    // Test get_i64_required
    assert_eq!(config.get_i64_required("port")?, 3000);

    // Test get_bool_required
    assert_eq!(config.get_bool_required("debug")?, true);

    // Test contains_key
    assert!(config.contains_key("name"));
    assert!(!config.contains_key("nonexistent"));

    // Test missing required keys
    assert!(config.get_string_required("missing").is_err());
    Ok(())
}

#[test]
fn test_env_loader() -> phenotype_config_core::Result<()> {
    // Note: This test depends on environment variables being set
    // In a real test suite, you'd use a test framework that can set env vars
    std::env::set_var("APP_PORT", "3000");
    std::env::set_var("APP_DEBUG", "true");

    let config = ConfigBuilder::new()
        .with_env_prefix("APP_")
        .build()?;

    assert_eq!(config.get_string("port"), Some("3000".to_string()));
    assert_eq!(config.get_string("debug"), Some("true".to_string()));

    std::env::remove_var("APP_PORT");
    std::env::remove_var("APP_DEBUG");
    Ok(())
}

#[test]
fn test_nested_config_sections() -> phenotype_config_core::Result<()> {
    let config = ConfigBuilder::new()
        .with_inline_value("database", json!({
            "host": "localhost",
            "port": 5432,
            "enabled": true
        }))
        .build()?;

    let db_config = config.get_section("database").expect("database section");
    assert_eq!(db_config.get_string("host"), Some("localhost".to_string()));
    assert_eq!(db_config.get_i64("port"), Some(5432));
    assert_eq!(db_config.get_bool("enabled"), Some(true));
    Ok(())
}

#[test]
fn test_config_builder_chain() -> phenotype_config_core::Result<()> {
    let config = ConfigBuilder::new()
        .with_inline_value("a", json!(1))
        .with_inline_value("b", json!(2))
        .with_inline_value("c", json!(3))
        .build()?;

    assert_eq!(config.len(), 3);
    assert!(!config.is_empty());

    let keys: Vec<_> = config.keys().collect();
    assert_eq!(keys.len(), 3);
    Ok(())
}

#[test]
fn test_file_not_found() -> phenotype_config_core::Result<()> {
    let result = ConfigBuilder::new()
        .with_file("/nonexistent/config.json")
        .build();

    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_config_validator_required_keys() -> phenotype_config_core::Result<()> {
    let validator = ConfigValidator::new()
        .require_key("port")
        .require_key("host");

    let valid = json!({"port": 3000, "host": "localhost"});
    assert!(validator.validate(&valid).is_ok());

    let invalid = json!({"port": 3000});
    assert!(validator.validate(&invalid).is_err());
    Ok(())
}

#[test]
fn test_config_validator_type_checks() {
    use phenotype_config_core::validator::ValueType;

    let validator = ConfigValidator::new()
        .require_type("port", ValueType::Number)
        .require_type("debug", ValueType::Boolean)
        .require_type("name", ValueType::String);

    let valid = json!({
        "port": 3000,
        "debug": true,
        "name": "app"
    });
    assert!(validator.validate(&valid).is_ok());

    let invalid_type = json!({
        "port": "3000",
        "debug": true,
        "name": "app"
    });
    assert!(validator.validate(&invalid_type).is_err());
}

#[test]
fn test_config_to_json_value() -> phenotype_config_core::Result<()> {
    let config = ConfigBuilder::new()
        .with_inline_value("port", json!(3000))
        .with_inline_value("debug", json!(true))
        .build()?;

    let json_value = config.to_json_value();
    assert!(json_value.is_object());
    assert_eq!(json_value.get("port").and_then(|v| v.as_i64()), Some(3000));
    Ok(())
}

#[test]
fn test_auto_format_detection() -> phenotype_config_core::Result<()> {
    // Test that format is auto-detected from file extension
    let result = ConfigFormat::from_extension(std::path::Path::new("config.json"));
    assert_eq!(result?, ConfigFormat::Json);

    let result = ConfigFormat::from_extension(std::path::Path::new("config.toml"));
    assert_eq!(result?, ConfigFormat::Toml);

    let result = ConfigFormat::from_extension(std::path::Path::new("config.yaml"));
    assert_eq!(result?, ConfigFormat::Yaml);

    let result = ConfigFormat::from_extension(std::path::Path::new("config.yml"));
    assert_eq!(result?, ConfigFormat::Yaml);
    Ok(())
}

#[test]
fn test_explicit_format_specification() -> phenotype_config_core::Result<()> {
    let mut file = NamedTempFile::new()?;
    writeln!(file, r#"{{"port": 3000}}"#)?;

    // Explicitly specify format even though file has .txt extension
    let config = ConfigBuilder::new()
        .with_file_format(file.path(), ConfigFormat::Json)
        .build()?;

    assert_eq!(config.get_i64("port"), Some(3000));
    Ok(())
}

#[test]
fn test_multiple_source_override() -> phenotype_config_core::Result<()> {
    let mut file1 = NamedTempFile::new()?;
    writeln!(file1, r#"{{"port": 2000, "host": "localhost"}}"#)?;

    let config = ConfigBuilder::new()
        .with_inline_value("port", json!(3000))
        .with_inline_value("name", json!("app"))
        .with_file(file1.path())
        .build()?;

    // File values override inline values (added later)
    assert_eq!(config.get_i64("port"), Some(2000));
    assert_eq!(config.get_string("host"), Some("localhost".to_string()));
    assert_eq!(config.get_string("name"), Some("app".to_string()));
    Ok(())
}

#[test]
fn test_empty_config() -> phenotype_config_core::Result<()> {
    let config = ConfigBuilder::new().build()?;

    assert!(config.is_empty());
    assert_eq!(config.len(), 0);
    assert_eq!(config.keys().count(), 0);
    Ok(())
}
