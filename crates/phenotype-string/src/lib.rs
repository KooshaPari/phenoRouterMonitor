//! # Phenotype String
//!
//! String utilities: case conversion, string building, inflection, slugify, truncate, and sanitization.
//!
//! # Features
//!
//! - **Case Conversion**: snake_case, camelCase, PascalCase, kebab-case
//! - **String Building**: Efficient builder pattern for string concatenation
//! - **Inflection**: Pluralization and singularization
//! - **Slugification**: URL-safe slug generation
//! - **Truncation**: Safely truncate strings with suffixes
//! - **ANSI Stripping**: Remove ANSI escape sequences

pub mod builder;
pub mod case;
pub mod inflection;

// Re-export common types and functions
pub use builder::StringBuilder;
pub use case::{to_camel_case, to_kebab_case, to_pascal_case, to_snake_case, CaseConverter};
pub use inflection::{pluralize, singularize, Inflection};

/// Convert a string to a URL-safe slug.
///
/// Lowercases, replaces non-alphanumeric with hyphens, collapses consecutive
/// hyphens, and trims leading/trailing hyphens.
pub fn slugify(s: &str) -> String {
    let re = regex::Regex::new(r"[^a-z0-9]+").unwrap();
    let lower = s.to_lowercase();
    let slug = re.replace_all(&lower, "-");
    slug.trim_matches('-').to_string()
}

/// Truncate a string to `max_len` characters, appending `suffix` if truncated.
pub fn truncate(s: &str, max_len: usize, suffix: &str) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let end = max_len.saturating_sub(suffix.len());
    let end = s.floor_char_boundary(end);
    format!("{}{}", &s[..end], suffix)
}

/// Strip ANSI escape sequences from a string.
pub fn strip_ansi(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello World!"), "hello-world");
        assert_eq!(slugify("  foo  BAR  "), "foo-bar");
        assert_eq!(slugify("already-slug"), "already-slug");
        assert_eq!(slugify("CamelCase"), "camelcase");
    }

    #[test]
    fn truncate_short() {
        assert_eq!(truncate("hi", 10, "..."), "hi");
    }

    #[test]
    fn truncate_long() {
        assert_eq!(truncate("hello world", 8, "..."), "hello...");
    }

    #[test]
    fn snake_case() {
        assert_eq!(to_snake_case("CamelCase"), "camel_case");
        assert_eq!(to_snake_case("hello-world"), "hello_world");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn pascal_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("foo-bar-baz"), "FooBarBaz");
        assert_eq!(to_pascal_case("already"), "Already");
    }

    #[test]
    fn strip_ansi_sequences() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(strip_ansi("no ansi"), "no ansi");
    }
}
