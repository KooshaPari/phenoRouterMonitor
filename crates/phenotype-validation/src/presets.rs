//! Built-in validator presets for common use cases.
//!
//! These are convenience constructors that combine multiple rules
//! to validate common field types (email, password, etc.).

use crate::traits::field_validator::FieldValidator;
use crate::traits::rule::{RequiredRule, LengthRule, PatternRule};
use crate::registry::ValidatorRegistry;

/// Email validator: required + email format.
pub fn email_validator() -> FieldValidator {
    FieldValidator::new()
        .with_rule(RequiredRule::new())
        .with_rule(
            PatternRule::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
                .expect("email pattern is valid"),
        )
}

/// URL validator: required + http/https scheme + non-empty host.
pub fn url_validator() -> FieldValidator {
    FieldValidator::new()
        .with_rule(RequiredRule::new())
        .with_rule(
            PatternRule::new(r"^https?://[^\s/$.?#].[^\s]*$")
                .expect("url pattern is valid"),
        )
}

/// Username validator: required + 3-20 chars + alphanumeric/underscore/hyphen.
pub fn username_validator() -> FieldValidator {
    FieldValidator::new()
        .with_rule(RequiredRule::new())
        .with_rule(LengthRule::range(3, 20))
        .with_rule(
            PatternRule::new(r"^[a-zA-Z0-9_-]+$")
                .expect("username pattern is valid"),
        )
}

/// Slug validator: required + lowercase + kebab-case + 1-50 chars.
pub fn slug_validator() -> FieldValidator {
    FieldValidator::new()
        .with_rule(RequiredRule::new())
        .with_rule(LengthRule::range(1, 50))
        .with_rule(
            PatternRule::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
                .expect("slug pattern is valid"),
        )
}

/// Password validator: strong policy (12+ chars, mixed case recommended).
pub fn strong_password_validator() -> FieldValidator {
    FieldValidator::new()
        .with_rule(RequiredRule::new())
        .with_rule(LengthRule::range(12, 128))
        .with_rule(
            PatternRule::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{12,}$")
                .expect("strong password pattern is valid"),
        )
}

/// Password validator: moderate policy (8+ chars).
pub fn moderate_password_validator() -> FieldValidator {
    FieldValidator::new()
        .with_rule(RequiredRule::new())
        .with_rule(LengthRule::range(8, 128))
}

/// Password validator: basic policy (6+ chars).
pub fn basic_password_validator() -> FieldValidator {
    FieldValidator::new()
        .with_rule(RequiredRule::new())
        .with_rule(LengthRule::range(6, 128))
}

/// US phone number validator: required + XXX-XXX-XXXX format.
pub fn us_phone_validator() -> FieldValidator {
    FieldValidator::new()
        .with_rule(RequiredRule::new())
        .with_rule(
            PatternRule::new(r"^\d{3}-\d{3}-\d{4}$")
                .expect("phone pattern is valid"),
        )
}

/// UUID validator: required + valid UUID format (hyphenated or simple).
pub fn uuid_validator() -> FieldValidator {
    FieldValidator::new()
        .with_rule(RequiredRule::new())
        .with_rule(
            PatternRule::new(r"^[0-9a-f]{8}-?[0-9a-f]{4}-?[0-9a-f]{4}-?[0-9a-f]{4}-?[0-9a-f]{12}$")
                .expect("uuid pattern is valid"),
        )
}

/// Initialize built-in validators in the registry.
///
/// Call this once at application startup to populate the registry
/// with all standard validators.
pub fn register_presets() {
    ValidatorRegistry::register("email", email_validator);
    ValidatorRegistry::register("url", url_validator);
    ValidatorRegistry::register("username", username_validator);
    ValidatorRegistry::register("slug", slug_validator);
    ValidatorRegistry::register("password_strong", strong_password_validator);
    ValidatorRegistry::register("password_moderate", moderate_password_validator);
    ValidatorRegistry::register("password_basic", basic_password_validator);
    ValidatorRegistry::register("phone_us", us_phone_validator);
    ValidatorRegistry::register("uuid", uuid_validator);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validator() {
        let validator = email_validator();
        assert!(validator.validate("user@example.com").is_ok());
        assert!(validator.validate("invalid.email").is_err());
        assert!(validator.validate("").is_err());
    }

    #[test]
    fn test_url_validator() {
        let validator = url_validator();
        assert!(validator.validate("https://example.com").is_ok());
        assert!(validator.validate("http://localhost:8080").is_ok());
        assert!(validator.validate("example.com").is_err());
        assert!(validator.validate("ftp://example.com").is_err());
    }

    #[test]
    fn test_username_validator() {
        let validator = username_validator();
        assert!(validator.validate("user_123").is_ok());
        assert!(validator.validate("ab").is_err()); // too short
        assert!(validator.validate("a".repeat(21).as_str()).is_err()); // too long
        assert!(validator.validate("user@name").is_err()); // invalid char
    }

    #[test]
    fn test_slug_validator() {
        let validator = slug_validator();
        assert!(validator.validate("my-slug").is_ok());
        assert!(validator.validate("another-valid-slug").is_ok());
        assert!(validator.validate("my_slug").is_err()); // underscores not allowed
        assert!(validator.validate("MySlug").is_err()); // uppercase not allowed
    }

    #[test]
    fn test_strong_password_validator() {
        let validator = strong_password_validator();
        assert!(validator.validate("MyPassword123").is_ok());
        assert!(validator.validate("short").is_err());
        assert!(validator.validate("nouppercasehere123").is_err());
        assert!(validator.validate("NOLOWERCASEHERE123").is_err());
    }

    #[test]
    fn test_moderate_password_validator() {
        let validator = moderate_password_validator();
        assert!(validator.validate("mypassword").is_ok());
        assert!(validator.validate("short").is_err());
    }

    #[test]
    fn test_us_phone_validator() {
        let validator = us_phone_validator();
        assert!(validator.validate("555-123-4567").is_ok());
        assert!(validator.validate("5551234567").is_err()); // missing hyphens
        assert!(validator.validate("555-12-4567").is_err()); // wrong format
    }

    #[test]
    fn test_uuid_validator() {
        let validator = uuid_validator();
        assert!(validator.validate("550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(validator.validate("550e8400e29b41d4a716446655440000").is_ok());
        assert!(validator.validate("not-a-uuid").is_err());
    }

    #[test]
    fn test_register_presets() {
        ValidatorRegistry::clear();
        register_presets();

        assert!(ValidatorRegistry::exists("email"));
        assert!(ValidatorRegistry::exists("url"));
        assert!(ValidatorRegistry::exists("username"));
        assert!(ValidatorRegistry::exists("slug"));
        assert!(ValidatorRegistry::exists("password_strong"));
        assert!(ValidatorRegistry::exists("password_moderate"));
        assert!(ValidatorRegistry::exists("password_basic"));
        assert!(ValidatorRegistry::exists("phone_us"));
        assert!(ValidatorRegistry::exists("uuid"));
    }
}
