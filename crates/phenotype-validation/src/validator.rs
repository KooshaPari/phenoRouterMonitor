use crate::errors::ValidationErrors;
use crate::rules::ValidationRule;

#[derive(Debug)]
pub struct Validator {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl Validator {
    pub fn new() -> Self { Self { rules: Vec::new() } }
    
    pub fn single(rule: impl ValidationRule + 'static) -> Self {
        let validator = Self::new();
        validator.add_rule(rule)
    }

    pub fn add_rule(mut self, rule: impl ValidationRule + 'static) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    pub fn rule_count(&self) -> usize { self.rules.len() }

    pub fn validate(&self, value: &str, field: &str) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        for rule in &self.rules {
            errors.add_if_err(rule.validate(value, field));
        }
        errors.into_result()
    }

    pub fn validate_fields(&self, fields: &[(String, String)]) -> Result<(), ValidationErrors> {
        let mut all_errors = ValidationErrors::new();
        for (field_name, field_value) in fields {
            if let Err(errs) = self.validate(field_value, field_name) {
                all_errors.merge(errs);
            }
        }
        all_errors.into_result()
    }
}

impl Default for Validator {
    fn default() -> Self { Self::new() }
}

pub trait ValidateFields {
    fn validate_with(&self, validator: &Validator) -> Result<(), ValidationErrors>;
}

pub struct ValidatorBuilder {
    validator: Validator,
}

impl ValidatorBuilder {
    pub fn new() -> Self { Self { validator: Validator::new() } }
    pub fn add_rule(self, rule: impl ValidationRule + 'static) -> Self {
        Self {
            validator: self.validator.add_rule(rule),
        }
    }
    pub fn build(self) -> Validator { self.validator }
}

impl Default for ValidatorBuilder {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{EmailRule, MinLengthRule, RequiredRule};

    #[test]
    fn test_validator_single_rule() {
        let validator = Validator::single(RequiredRule);
        assert!(validator.validate("hello", "name").is_ok());
        assert!(validator.validate("", "name").is_err());
    }

    #[test]
    fn test_validator_multiple_rules() {
        let validator = Validator::new()
            .add_rule(RequiredRule)
            .add_rule(MinLengthRule::new(5));
        assert!(validator.validate("hello world", "name").is_ok());
        assert!(validator.validate("hi", "name").is_err());
        assert!(validator.validate("", "name").is_err());
    }

    #[test]
    fn test_validator_email() {
        let validator = Validator::single(EmailRule::new());
        assert!(validator.validate("user@example.com", "email").is_ok());
        assert!(validator.validate("invalid", "email").is_err());
    }

    #[test]
    fn test_validator_builder() {
        let validator = ValidatorBuilder::new()
            .add_rule(RequiredRule)
            .add_rule(MinLengthRule::new(3))
            .build();
        assert!(validator.validate("hello", "name").is_ok());
        assert!(validator.validate("", "name").is_err());
    }

    #[test]
    fn test_validator_rule_count() {
        let validator = Validator::new()
            .add_rule(RequiredRule)
            .add_rule(MinLengthRule::new(5));
        assert_eq!(validator.rule_count(), 2);
    }
}
