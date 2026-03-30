//! Case conversion utilities for string transformations.
//!
//! Provides pure, stateless functions for converting between various string case formats:
//! - snake_case: lowercase with underscores
//! - camelCase: first word lowercase, subsequent words capitalized
//! - PascalCase: all words capitalized
//! - kebab-case: lowercase with hyphens

use regex::Regex;

/// Converts a string to snake_case (lowercase with underscores).
///
/// # Arguments
/// * `s` - String to convert
///
/// # Returns
/// snake_case converted string
///
/// # Example
/// ```
/// use phenotype_string::to_snake_case;
/// assert_eq!(to_snake_case("CamelCase"), "camel_case");
/// assert_eq!(to_snake_case("PascalCase"), "pascal_case");
/// ```
pub fn to_snake_case(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }

    let mut result = String::with_capacity(s.len() + 4);
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.extend(ch.to_lowercase());
        } else if ch == '-' || ch == ' ' {
            result.push('_');
        } else {
            result.push(ch);
        }
    }
    result
}

/// Converts a string to camelCase (first word lowercase, rest capitalized).
///
/// # Arguments
/// * `s` - String to convert
///
/// # Returns
/// camelCase converted string
///
/// # Example
/// ```
/// use phenotype_string::to_camel_case;
/// assert_eq!(to_camel_case("snake_case"), "snakeCase");
/// assert_eq!(to_camel_case("PascalCase"), "pascalCase");
/// ```
pub fn to_camel_case(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }

    let words = split_on_boundaries(s);
    let mut result = String::new();

    for (i, word) in words.iter().enumerate() {
        if i == 0 {
            result.push_str(&word.to_lowercase());
        } else {
            result.push_str(&capitalize(word));
        }
    }

    result
}

/// Converts a string to PascalCase (all words capitalized).
///
/// # Arguments
/// * `s` - String to convert
///
/// # Returns
/// PascalCase converted string
///
/// # Example
/// ```
/// use phenotype_string::to_pascal_case;
/// assert_eq!(to_pascal_case("snake_case"), "SnakeCase");
/// assert_eq!(to_pascal_case("kebab-case"), "KebabCase");
/// ```
pub fn to_pascal_case(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }

    s.split(['_', '-', ' '])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut s = first.to_uppercase().to_string();
                    s.extend(chars.flat_map(|c| c.to_lowercase()));
                    s
                }
            }
        })
        .collect()
}

/// Converts a string to kebab-case (lowercase with hyphens).
///
/// # Arguments
/// * `s` - String to convert
///
/// # Returns
/// kebab-case converted string
///
/// # Example
/// ```
/// use phenotype_string::to_kebab_case;
/// assert_eq!(to_kebab_case("snake_case"), "snake-case");
/// assert_eq!(to_kebab_case("camelCase"), "camel-case");
/// ```
pub fn to_kebab_case(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }

    let s = replace_boundaries(s, "-");
    s.to_lowercase()
}

/// CaseConverter provides a builder-style interface for string case conversion.
///
/// # Example
/// ```
/// use phenotype_string::CaseConverter;
/// let result = CaseConverter::new("hello_world")
///     .to_pascal()
///     .build();
/// assert_eq!(result, "HelloWorld");
/// ```
#[derive(Clone)]
pub struct CaseConverter {
    input: String,
}

impl CaseConverter {
    pub fn new(s: &str) -> Self {
        CaseConverter {
            input: s.to_string(),
        }
    }

    pub fn to_snake(self) -> CaseBuilder {
        CaseBuilder {
            result: to_snake_case(&self.input),
        }
    }

    pub fn to_camel(self) -> CaseBuilder {
        CaseBuilder {
            result: to_camel_case(&self.input),
        }
    }

    pub fn to_pascal(self) -> CaseBuilder {
        CaseBuilder {
            result: to_pascal_case(&self.input),
        }
    }

    pub fn to_kebab(self) -> CaseBuilder {
        CaseBuilder {
            result: to_kebab_case(&self.input),
        }
    }
}

/// Builder for case conversion, returned by CaseConverter methods.
#[derive(Clone)]
pub struct CaseBuilder {
    result: String,
}

impl CaseBuilder {
    /// Returns the converted string.
    pub fn build(self) -> String {
        self.result
    }
}

fn capitalize(s: &str) -> String {
    if s.is_empty() {
        s.to_string()
    } else {
        let mut chars = s.chars();
        chars.next().unwrap().to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
    }
}

fn replace_boundaries(s: &str, separator: &str) -> String {
    let normalized = s.replace(['-', '_', ' '], separator);
    let regex = Regex::new(r"([a-z])([A-Z])").expect("Failed to compile case regex");
    regex
        .replace_all(&normalized, |caps: &regex::Captures| {
            format!("{}{}{}", &caps[1], separator, &caps[2])
        })
        .to_string()
}

fn split_on_boundaries(s: &str) -> Vec<String> {
    let normalized = s.replace(['-', '_', ' '], "|");
    let regex = Regex::new(r"([a-z])([A-Z])").expect("Failed to compile case regex");
    let with_boundaries = regex
        .replace_all(&normalized, |caps: &regex::Captures| {
            format!("{}|{}", &caps[1], &caps[2])
        })
        .to_string();

    with_boundaries
        .split('|')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("CamelCase"), "camel_case");
        assert_eq!(to_snake_case("PascalCase"), "pascal_case");
        assert_eq!(to_snake_case("kebab-case"), "kebab_case");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("snake_case"), "snakeCase");
        assert_eq!(to_camel_case("PascalCase"), "pascalCase");
        assert_eq!(to_camel_case("kebab-case"), "kebabCase");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("snake_case"), "SnakeCase");
        assert_eq!(to_pascal_case("kebab-case"), "KebabCase");
        assert_eq!(to_pascal_case("hello world"), "HelloWorld");
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("snake_case"), "snake-case");
        assert_eq!(to_kebab_case("camelCase"), "camel-case");
        assert_eq!(to_kebab_case("PascalCase"), "pascal-case");
    }

    #[test]
    fn test_case_converter_snake() {
        assert_eq!(
            CaseConverter::new("HelloWorld").to_snake().build(),
            "hello_world"
        );
    }

    #[test]
    fn test_case_converter_camel() {
        assert_eq!(
            CaseConverter::new("snake_case").to_camel().build(),
            "snakeCase"
        );
    }

    #[test]
    fn test_case_converter_pascal() {
        assert_eq!(
            CaseConverter::new("snake_case").to_pascal().build(),
            "SnakeCase"
        );
    }

    #[test]
    fn test_case_converter_kebab() {
        assert_eq!(
            CaseConverter::new("camelCase").to_kebab().build(),
            "camel-case"
        );
    }

    #[test]
    fn test_case_converter_clone() {
        let converter = CaseConverter::new("test_string");
        let cloned = converter.clone();
        assert_eq!(cloned.to_snake().build(), to_snake_case("test_string"));
    }

    #[test]
    fn test_mixed_separators() {
        assert_eq!(to_pascal_case("hello-world_test case"), "HelloWorldTestCase");
        assert_eq!(to_snake_case("hello-world_testCase"), "hello_world_test_case");
    }

    #[test]
    fn test_empty_strings() {
        assert_eq!(to_snake_case(""), "");
        assert_eq!(to_camel_case(""), "");
        assert_eq!(to_pascal_case(""), "");
        assert_eq!(to_kebab_case(""), "");
    }

    #[test]
    fn test_single_characters() {
        assert_eq!(to_snake_case("a"), "a");
        assert_eq!(to_snake_case("A"), "a");
        assert_eq!(to_camel_case("a"), "a");
        assert_eq!(to_pascal_case("a"), "A");
    }

    #[test]
    fn test_consecutive_underscores() {
        assert_eq!(to_snake_case("hello__world"), "hello__world");
        assert_eq!(to_camel_case("hello__world"), "helloWorld");
    }

    #[test]
    fn test_leading_trailing_separators() {
        // Function keeps leading/trailing separators as is in snake_case
        assert_eq!(to_snake_case("_hello_world_"), "_hello_world_");
        // Hyphens become underscores in camel_case
        assert_eq!(to_camel_case("-hello-world-"), "helloWorld");
    }

    #[test]
    fn test_acronyms() {
        assert_eq!(to_snake_case("HTTPServer"), "h_t_t_p_server");
        assert_eq!(to_pascal_case("http_server"), "HttpServer");
    }

    #[test]
    fn test_numbers() {
        assert_eq!(to_snake_case("Test123Case"), "test123_case");
        assert_eq!(to_camel_case("test_123_case"), "test123Case");
        assert_eq!(to_pascal_case("test_123_case"), "Test123Case");
    }

    #[test]
    fn test_case_builder_clone() {
        let builder = CaseConverter::new("hello_world").to_camel();
        let cloned = builder.clone();
        assert_eq!(builder.build(), cloned.build());
    }
}
