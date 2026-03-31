//! String sanitization utilities.

/// Trait for sanitizing strings.
pub trait Sanitize {
    /// Convert to snake_case.
    fn to_snake_case(&self) -> String;

    /// Convert to SCREAMING_SNAKE_CASE.
    fn to_screaming_snake(&self) -> String;

    /// Convert to kebab-case.
    fn to_kebab_case(&self) -> String;

    /// Convert to PascalCase.
    fn to_pascal_case(&self) -> String;

    /// Convert to camelCase.
    fn to_camel_case(&self) -> String;

    /// Remove leading/trailing whitespace and collapse internal whitespace.
    fn normalize_whitespace(&self) -> String;

    /// Truncate to max length with ellipsis.
    fn truncate(&self, max_len: usize) -> String;
}

impl<T: AsRef<str>> Sanitize for T {
    fn to_snake_case(&self) -> String {
        let s = self.as_ref();
        let mut result = String::with_capacity(s.len());
        let mut prev_was_upper = false;

        for (i, c) in s.char_indices() {
            if c.is_uppercase() {
                if i > 0 && !prev_was_upper {
                    result.push('_');
                }
                result.push(c.to_ascii_lowercase());
                prev_was_upper = true;
            } else if c == '-' || c == ' ' {
                if !result.ends_with('_') {
                    result.push('_');
                }
                prev_was_upper = false;
            } else {
                result.push(c);
                prev_was_upper = false;
            }
        }

        result.trim_matches('_').to_lowercase()
    }

    fn to_screaming_snake(&self) -> String {
        self.to_snake_case().to_uppercase()
    }

    fn to_kebab_case(&self) -> String {
        self.to_snake_case().replace('_', "-")
    }

    fn to_pascal_case(&self) -> String {
        let s = self.as_ref();
        let mut result = String::with_capacity(s.len());
        let mut capitalize_next = true;

        for c in s.chars() {
            if c == '_' || c == '-' || c == ' ' {
                capitalize_next = true;
            } else if capitalize_next {
                result.extend(c.to_uppercase());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        }

        result
    }

    fn to_camel_case(&self) -> String {
        let pascal = self.to_pascal_case();
        let mut chars = pascal.chars();
        match chars.next() {
            Some(c) => c.to_ascii_lowercase().to_string() + chars.as_str(),
            None => String::new(),
        }
    }

    fn normalize_whitespace(&self) -> String {
        self.as_ref()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn truncate(&self, max_len: usize) -> String {
        let s = self.as_ref();
        if s.len() <= max_len {
            s.to_string()
        } else if max_len < 3 {
            s[..max_len].to_string()
        } else {
            format!("{}...", &s[..max_len - 3])
        }
    }
}
