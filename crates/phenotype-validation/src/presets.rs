use crate::rules::*;
use crate::validator::Validator;

pub struct EmailValidator;
impl EmailValidator {
    pub fn email() -> Validator {
        Validator::new()
            .add_rule(RequiredRule)
            .add_rule(EmailRule::new())
    }
}
impl Default for EmailValidator {
    fn default() -> Self { Self }
}

pub struct UrlValidator;
impl UrlValidator {
    pub fn url() -> Validator {
        Validator::new()
            .add_rule(RequiredRule)
            .add_rule(UrlRule::new())
    }
}
impl Default for UrlValidator {
    fn default() -> Self { Self }
}

pub struct UsernameValidator;
impl UsernameValidator {
    pub fn username() -> Validator {
        let pattern = PatternRule::new(r"^[a-zA-Z0-9_-]{3,20}$").unwrap();
        Validator::new()
            .add_rule(RequiredRule)
            .add_rule(pattern)
    }
}
impl Default for UsernameValidator {
    fn default() -> Self { Self }
}

pub struct PasswordValidator;
impl PasswordValidator {
    pub fn strong() -> Validator {
        Validator::new()
            .add_rule(RequiredRule)
            .add_rule(MinLengthRule::new(8))
            .add_rule(MaxLengthRule::new(128))
    }
    pub fn moderate() -> Validator {
        Validator::new()
            .add_rule(RequiredRule)
            .add_rule(MinLengthRule::new(8))
            .add_rule(MaxLengthRule::new(128))
    }
    pub fn basic() -> Validator {
        Validator::new()
            .add_rule(RequiredRule)
            .add_rule(MinLengthRule::new(6))
            .add_rule(MaxLengthRule::new(128))
    }
}
impl Default for PasswordValidator {
    fn default() -> Self { Self }
}

pub struct PhoneValidator;
impl PhoneValidator {
    pub fn us_format() -> Validator {
        let pattern = PatternRule::new(r"^\d{3}-\d{3}-\d{4}$").unwrap();
        Validator::new()
            .add_rule(RequiredRule)
            .add_rule(pattern)
    }
}
impl Default for PhoneValidator {
    fn default() -> Self { Self }
}

pub struct SlugValidator;
impl SlugValidator {
    pub fn slug() -> Validator {
        let pattern = PatternRule::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
        Validator::new()
            .add_rule(RequiredRule)
            .add_rule(MinLengthRule::new(1))
            .add_rule(MaxLengthRule::new(50))
            .add_rule(pattern)
    }
}
impl Default for SlugValidator {
    fn default() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validator() {
        let validator = EmailValidator::email();
        assert!(validator.validate("user@example.com", "email").is_ok());
        assert!(validator.validate("invalid", "email").is_err());
        assert!(validator.validate("", "email").is_err());
    }

    #[test]
    fn test_url_validator() {
        let validator = UrlValidator::url();
        assert!(validator.validate("https://example.com", "url").is_ok());
        assert!(validator.validate("not-a-url", "url").is_err());
    }

    #[test]
    fn test_username_validator() {
        let validator = UsernameValidator::username();
        assert!(validator.validate("user_123", "username").is_ok());
        assert!(validator.validate("ab", "username").is_err());
    }

    #[test]
    fn test_password_basic() {
        let validator = PasswordValidator::basic();
        assert!(validator.validate("mypass123", "password").is_ok());
        assert!(validator.validate("short", "password").is_err());
    }

    #[test]
    fn test_phone_us_format() {
        let validator = PhoneValidator::us_format();
        assert!(validator.validate("555-123-4567", "phone").is_ok());
        assert!(validator.validate("invalid", "phone").is_err());
    }

    #[test]
    fn test_slug_validator() {
        let validator = SlugValidator::slug();
        assert!(validator.validate("my-slug", "slug").is_ok());
        assert!(validator.validate("my_slug", "slug").is_err());
    }
}
