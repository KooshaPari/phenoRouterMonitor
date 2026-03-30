/// Test suite for Builder derive macro
use phenotype_macros::Builder;

#[derive(Builder, Debug, Clone)]
pub struct User {
    name: String,
    email: String,
    age: u32,
}

#[test]
fn test_builder_basic() {
    let user = User::builder()
        .name("Alice".to_string())
        .email("alice@example.com".to_string())
        .age(30)
        .build();

    assert!(user.is_ok());
    let user = user.unwrap();
    assert_eq!(user.name, "Alice");
    assert_eq!(user.email, "alice@example.com");
    assert_eq!(user.age, 30);
}

#[test]
fn test_builder_missing_field() {
    let result = User::builder()
        .name("Bob".to_string())
        .email("bob@example.com".to_string())
        // Missing age
        .build();

    assert!(result.is_err());
}

#[test]
fn test_builder_default() {
    let builder = User::builder();
    // Verify builder is created with default values
    assert!(builder.build().is_err()); // Should fail with missing fields
}

#[derive(Builder, Debug)]
pub struct Product {
    id: String,
    name: String,
    price: f64,
}

#[test]
fn test_builder_product() {
    let product = Product::builder()
        .id("SKU-001".to_string())
        .name("Widget".to_string())
        .price(19.99)
        .build();

    assert!(product.is_ok());
    let product = product.unwrap();
    assert_eq!(product.id, "SKU-001");
    assert_eq!(product.name, "Widget");
    assert_eq!(product.price, 19.99);
}

#[test]
fn test_builder_chaining() {
    let user = User::builder()
        .name("Charlie".to_string())
        .email("charlie@example.com".to_string())
        .age(25)
        .build();

    assert!(user.is_ok());
}

#[derive(Builder, Debug)]
pub struct Config {
    host: String,
    port: u16,
    debug: bool,
}

#[test]
fn test_builder_config() {
    let config = Config::builder()
        .host("localhost".to_string())
        .port(8080)
        .debug(true)
        .build();

    assert!(config.is_ok());
    let config = config.unwrap();
    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 8080);
    assert!(config.debug);
}
