# Validation System Examples

Comprehensive examples demonstrating the phenotype-validation framework.

---

## Example 1: Using Built-in Presets

Register and use pre-built validators from the registry.

```rust
use phenotype_validation::presets::register_presets;
use phenotype_validation::registry::ValidatorRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize all built-in validators
    register_presets();

    // Get email validator
    let email_validator = ValidatorRegistry::get("email")
        .expect("email validator registered");

    // Validate various inputs
    match email_validator.validate("user@example.com") {
        Ok(()) => println!("Email is valid"),
        Err(e) => println!("Validation failed: {}", e.display_message()),
    }

    // Get password validator
    let password_validator = ValidatorRegistry::get("password_strong")
        .expect("password_strong validator registered");

    let pwd_result = password_validator.validate("MyPass123");
    if pwd_result.is_ok() {
        println!("Password strength OK");
    }

    Ok(())
}
```

---

## Example 2: Building Custom Field Validators

Compose rules to create field-specific validators.

```rust
use phenotype_validation::traits::field_validator::FieldValidator;
use phenotype_validation::traits::rule::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a username validator: 3-20 chars, alphanumeric + underscore
    let username_validator = FieldValidator::new()
        .with_rule(RequiredRule::new())
        .with_rule(LengthRule::range(3, 20))
        .with_rule(
            PatternRule::new(r"^[a-zA-Z0-9_]+$")
                .expect("pattern is valid")
        );

    // Test it
    assert!(username_validator.validate("john_doe").is_ok());
    assert!(username_validator.validate("ab").is_err());  // too short
    assert!(username_validator.validate("user@name").is_err()); // invalid char
    assert!(username_validator.validate("").is_err());    // empty

    Ok(())
}
```

---

## Example 3: Command-Level Validation

Validate entire commands with multiple fields.

```rust
use phenotype_validation::traits::{CommandValidator, FieldValidator};
use phenotype_validation::traits::rule::*;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a command validator for "CreateUserCommand"
    let user_validator = CommandValidator::new()
        .add_field(
            "name",
            FieldValidator::new()
                .with_rule(RequiredRule::new())
                .with_rule(LengthRule::range(2, 50))
        )
        .add_field(
            "email",
            FieldValidator::new()
                .with_rule(RequiredRule::new())
                .with_rule(
                    PatternRule::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
                        .expect("valid pattern")
                )
        )
        .add_field(
            "age",
            FieldValidator::new()
                .with_rule(RequiredRule::new())
                .with_rule(NumericRangeRule::range(13.0, 150.0))
        );

    // Prepare command data
    let mut command_fields = HashMap::new();
    command_fields.insert("name".to_string(), "John Doe".to_string());
    command_fields.insert("email".to_string(), "john@example.com".to_string());
    command_fields.insert("age".to_string(), "30".to_string());

    // Validate
    let result = user_validator.validate(&command_fields);

    if result.is_ok() {
        println!("User command is valid!");
    } else {
        println!("{}", result.summary());
        // Output:
        // Validation failed with N error(s):
        //   - field_name: error message (suggestion)
    }

    Ok(())
}
```

---

## Example 4: Custom Validation Rules

Create domain-specific rules for business logic.

```rust
use phenotype_validation::traits::rule::ValidationRule;
use phenotype_validation::traits::{ValidationError, Severity};

/// Validates that a workflow status is in an allowed set.
#[derive(Clone, Debug)]
pub struct WorkflowStatusRule {
    allowed_states: Vec<String>,
}

impl WorkflowStatusRule {
    pub fn new(allowed: Vec<&str>) -> Self {
        Self {
            allowed_states: allowed.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl ValidationRule for WorkflowStatusRule {
    fn validate(&self, value: &str) -> Result<(), ValidationError> {
        if self.allowed_states.contains(&value.to_string()) {
            Ok(())
        } else {
            Err(ValidationError::new("invalid_workflow_status")
                .with_severity(Severity::Error)
                .with_suggestion(format!(
                    "Status must be one of: {}",
                    self.allowed_states.join(", ")
                )))
        }
    }

    fn name(&self) -> &'static str {
        "workflow_status"
    }
}

// Usage:
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rule = WorkflowStatusRule::new(vec!["pending", "active", "completed"]);

    assert!(rule.validate("active").is_ok());
    assert!(rule.validate("unknown").is_err());

    Ok(())
}
```

---

## Example 5: Plugin Registration and Discovery

Register custom validators in a global registry.

```rust
use phenotype_validation::registry::ValidatorRegistry;
use phenotype_validation::traits::field_validator::FieldValidator;
use phenotype_validation::traits::rule::*;

// Define custom validator
fn corporate_email_validator() -> FieldValidator {
    FieldValidator::new()
        .with_rule(RequiredRule::new())
        .with_rule(
            PatternRule::new(r"^[a-zA-Z0-9._%+-]+@company\.com$")
                .expect("valid pattern")
        )
}

// Register at startup
pub fn register_company_validators() {
    ValidatorRegistry::register("corporate_email", corporate_email_validator);
    ValidatorRegistry::register("slack_handle", || {
        FieldValidator::new()
            .with_rule(RequiredRule::new())
            .with_rule(PatternRule::new(r"^@[a-z0-9_-]+$").expect("valid"))
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Register custom validators
    register_company_validators();

    // List available validators
    let all_validators = ValidatorRegistry::list();
    println!("Available validators: {:?}", all_validators);

    // Get validator by name
    let corporate = ValidatorRegistry::get("corporate_email")
        .expect("corporate_email registered");

    assert!(corporate.validate("user@company.com").is_ok());
    assert!(corporate.validate("user@example.com").is_err());

    Ok(())
}
```

---

## Example 6: Error Context and Suggestions

Rich error reporting with context.

```rust
use phenotype_validation::traits::field_validator::FieldValidator;
use phenotype_validation::traits::rule::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let validator = FieldValidator::new()
        .with_rule(RequiredRule::new())
        .with_rule(LengthRule::range(8, 128));

    // Trigger various errors
    match validator.validate("short") {
        Ok(()) => println!("Valid"),
        Err(e) => {
            // Display error with context
            println!("Error: {}", e.display_message());
            // Output: "min_length (field: password). Minimum 8 characters required"
            
            println!("Severity: {}", e.severity);
            // Output: "error"
            
            if let Some(suggestion) = &e.suggestion {
                println!("Suggestion: {}", suggestion);
            }
        }
    }

    Ok(())
}
```

---

## Example 7: Testing Validators

Unit and integration tests for validators.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use phenotype_validation::traits::field_validator::FieldValidator;
    use phenotype_validation::traits::rule::*;

    #[test]
    fn test_email_field_validator() {
        let validator = FieldValidator::new()
            .with_rule(RequiredRule::new())
            .with_rule(
                PatternRule::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
                    .expect("valid pattern")
            );

        // Valid cases
        assert!(validator.validate("user@example.com").is_ok());
        assert!(validator.validate("first.last+tag@example.co.uk").is_ok());

        // Invalid cases
        assert!(validator.validate("invalid.email").is_err());
        assert!(validator.validate("").is_err());
        assert!(validator.validate("user@").is_err());
    }

    #[test]
    fn test_collect_all_errors() {
        let validator = FieldValidator::new()
            .with_rule(RequiredRule::new())
            .with_rule(LengthRule::min(5));

        let errors = validator.validate_all("ab");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "min_length");
    }

    #[test]
    fn test_command_validator_integration() {
        use std::collections::HashMap;

        let validator = CommandValidator::new()
            .add_field("username", FieldValidator::new()
                .with_rule(RequiredRule::new())
                .with_rule(LengthRule::min(3)))
            .add_field("password", FieldValidator::new()
                .with_rule(RequiredRule::new())
                .with_rule(LengthRule::min(8)));

        // Valid command
        let mut valid_fields = HashMap::new();
        valid_fields.insert("username".to_string(), "alice".to_string());
        valid_fields.insert("password".to_string(), "securepass123".to_string());

        assert!(validator.validate(&valid_fields).is_ok());

        // Invalid command (short password)
        let mut invalid_fields = HashMap::new();
        invalid_fields.insert("username".to_string(), "alice".to_string());
        invalid_fields.insert("password".to_string(), "short".to_string());

        let result = validator.validate(&invalid_fields);
        assert!(!result.is_ok());
        assert!(result.field_errors.contains_key("password"));
    }
}
```

---

## Example 8: Async Validation (Future)

Preparing for async validation (e.g., database uniqueness checks).

```rust
// Pseudocode - not yet implemented in phenotype-validation
// Planned for Phase 2

use std::sync::Arc;

/// Async rule: check username uniqueness against database
#[derive(Clone)]
pub struct UniqueUsernameRule {
    db: Arc<dyn UserRepository>, // Async DB client
}

impl UniqueUsernameRule {
    pub async fn validate_async(&self, value: &str) -> Result<(), ValidationError> {
        let exists = self.db.username_exists(value).await?;
        
        if exists {
            Err(ValidationError::new("username_taken")
                .with_suggestion("This username is already registered. Please choose another."))
        } else {
            Ok(())
        }
    }
}

// Usage:
#[tokio::test]
async fn test_unique_username() {
    let mock_db = Arc::new(MockUserRepository::new());
    let rule = UniqueUsernameRule { db: mock_db };

    assert!(rule.validate_async("alice").await.is_ok());
    // (assuming "alice" is not in the mock DB)
}
```

---

## Example 9: Preset Composition

Reusing presets to build more complex validators.

```rust
use phenotype_validation::presets::*;
use phenotype_validation::traits::field_validator::FieldValidator;
use phenotype_validation::traits::rule::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start with a preset and add custom rules
    let enhanced_email = {
        let mut v = email_validator(); // Built-in preset
        
        // Add custom domain restriction
        let domain_rule = PatternRule::new(r"@(example\.com|company\.com)$")
            .expect("valid pattern");
        
        // Note: FieldValidator::add_rule takes ownership, so we need to rebuild
        FieldValidator::new()
            .with_rule(RequiredRule::new())
            .with_rule(PatternRule::new(r"^[a-zA-Z0-9._%+-]+@(example\.com|company\.com)$")
                .expect("valid pattern"))
    };

    assert!(enhanced_email.validate("user@example.com").is_ok());
    assert!(enhanced_email.validate("user@gmail.com").is_err());

    Ok(())
}
```

---

## Example 10: Registry Integration Test

Full integration from registration to CLI usage.

```rust
use phenotype_validation::presets::register_presets;
use phenotype_validation::registry::ValidatorRegistry;
use std::collections::HashMap;

#[test]
fn test_registry_integration() {
    // Register built-in validators
    register_presets();

    // Verify validators are available
    assert!(ValidatorRegistry::exists("email"));
    assert!(ValidatorRegistry::exists("password_strong"));
    assert!(ValidatorRegistry::exists("username"));

    // Simulate CLI command validation
    let mut user_data = HashMap::new();
    user_data.insert("email".to_string(), "user@example.com".to_string());
    user_data.insert("username".to_string(), "alice_123".to_string());
    user_data.insert("password".to_string(), "MySecurePass123".to_string());

    // Validate each field via registry
    for (field_name, field_value) in &user_data {
        if let Some(validator) = ValidatorRegistry::get(field_name) {
            assert!(
                validator.validate(field_value).is_ok(),
                "Field {} validation failed", field_name
            );
        }
    }
}
```

---

## Running Examples

To run these examples:

```bash
# From the phenotype repo root
cd /Users/kooshapari/CodeProjects/Phenotype/repos

# Add example code to crates/phenotype-validation/examples/
cat > crates/phenotype-validation/examples/basic_usage.rs << 'EOF'
// ... copy Example 1 above ...
EOF

# Run example
cargo run --example basic_usage

# Run tests
cargo test --lib traits::
cargo test --lib registry::
cargo test --lib presets::
```

---

## Next Steps

1. Explore the API documentation: `cargo doc --open`
2. Review source code: `/crates/phenotype-validation/src/`
3. Check out the architecture: `docs/guides/VALIDATION_DECOMPOSITION_AND_REGISTRY_PATTERN.md`
4. Follow migration guide: `docs/changes/validation-decomposition/MIGRATION_GUIDE.md`

