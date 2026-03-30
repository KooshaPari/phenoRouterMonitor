//! Case conversion utilities: convert between snake_case, camelCase, PascalCase, kebab-case.

use once_cell::sync::Lazy;
use regex::Regex;

static SEPARATORS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[-_\s\.]+").expect("Invalid regex")
});

/// Split a string into words by detecting case changes and separators.
fn split_into_words(input: &str) -> Vec<String> {
    if input.is_empty() {
        return vec![];
    }

    // First, split by obvious separators (-, _, space, dot)
    let parts: Vec<&str> = SEPARATORS.split(input).collect();

    let mut words = Vec::new();

    for part in parts {
        if part.is_empty() {
            continue;
        }

        // Then, split camelCase/PascalCase within each part
        let mut current_word = String::new();
        let chars: Vec<char> = part.chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            let is_upper = ch.is_uppercase();
            let is_lower = ch.is_lowercase();

            if i == 0 {
                current_word.push(ch);
            } else {
                let prev_is_upper = chars[i - 1].is_uppercase();
                let prev_is_lower = chars[i - 1].is_lowercase();
                let next_is_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();

                // Start a new word on: lowercase->uppercase, or uppercase->uppercase->lowercase (HTTPServer -> HTTP_Server)
                if (prev_is_lower && is_upper) || (prev_is_upper && is_upper && next_is_lower) {
                    if !current_word.is_empty() {
                        words.push(current_word);
                        current_word = String::new();
                    }
                }

                current_word.push(ch);
            }
        }

        if !current_word.is_empty() {
            words.push(current_word);
        }
    }

    words
}

/// Convert input to snake_case.
///
/// Handles multiple input formats:
/// - CamelCase -> camel_case
/// - kebab-case -> kebab_case
/// - PascalCase -> pascal_case
/// - spaces -> spaces_become_underscores
///
/// # Examples
/// ```
/// use phenotype_string::convert_to_snake_case;
///
/// assert_eq!(convert_to_snake_case("HelloWorld"), "hello_world");
/// assert_eq!(convert_to_snake_case("hello-world"), "hello_world");
/// assert_eq!(convert_to_snake_case("hello world"), "hello_world");
/// assert_eq!(convert_to_snake_case("hello_world"), "hello_world");
/// ```
pub fn convert_to_snake_case(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    // First pass: insert underscores at case boundaries
    let with_boundaries = CAMEL_SNAKE.replace_all(input, "${1}_${2}");

    // Second pass: normalize all separators to underscores
    let normalized = WORD_BOUNDARY.replace_all(&with_boundaries, "_");

    // Third pass: clean up and lowercase
    normalized
        .split('_')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect::<Vec<_>>()
        .join("_")
}

/// Convert input to camelCase.
///
/// Handles multiple input formats:
/// - snake_case -> snakeCase
/// - kebab-case -> kebabCase
/// - PascalCase -> pascalCase
/// - spaces -> spacesAreRemoved
///
/// # Examples
/// ```
/// use phenotype_string::convert_to_camel_case;
///
/// assert_eq!(convert_to_camel_case("hello_world"), "helloWorld");
/// assert_eq!(convert_to_camel_case("hello-world"), "helloWorld");
/// assert_eq!(convert_to_camel_case("HelloWorld"), "helloWorld");
/// assert_eq!(convert_to_camel_case("hello world"), "helloWorld");
/// ```
pub fn convert_to_camel_case(input: &str) -> String {
    let words = split_into_words(input);

    if words.is_empty() {
        return String::new();
    }

    // First word is lowercase, rest are title-cased
    let mut result = words[0].to_lowercase();
    for word in &words[1..] {
        result.push_str(&title_case(word));
    }

    result
}

/// Convert input to kebab-case.
///
/// Handles multiple input formats:
/// - CamelCase -> camel-case
/// - snake_case -> snake-case
/// - PascalCase -> pascal-case
/// - spaces -> spaces-become-dashes
///
/// # Examples
/// ```
/// use phenotype_string::convert_to_kebab_case;
///
/// assert_eq!(convert_to_kebab_case("HelloWorld"), "hello-world");
/// assert_eq!(convert_to_kebab_case("hello_world"), "hello-world");
/// assert_eq!(convert_to_kebab_case("hello world"), "hello-world");
/// ```
pub fn convert_to_kebab_case(input: &str) -> String {
    split_into_words(input)
        .iter()
        .map(|word| word.to_lowercase())
        .collect::<Vec<_>>()
        .join("-")
}

/// Convert input to PascalCase (UpperCamelCase).
///
/// Handles multiple input formats:
/// - snake_case -> SnakeCase
/// - kebab-case -> KebabCase
/// - camelCase -> CamelCase
/// - spaces -> SpacesAreRemoved
///
/// # Examples
/// ```
/// use phenotype_string::convert_to_pascal_case;
///
/// assert_eq!(convert_to_pascal_case("hello_world"), "HelloWorld");
/// assert_eq!(convert_to_pascal_case("hello-world"), "HelloWorld");
/// assert_eq!(convert_to_pascal_case("helloWorld"), "HelloWorld");
/// assert_eq!(convert_to_pascal_case("hello world"), "HelloWorld");
/// ```
pub fn convert_to_pascal_case(input: &str) -> String {
    let words = split_into_words(input);

    if words.is_empty() {
        return String::new();
    }

    // All words are title-cased
    words
        .iter()
        .map(|word| title_case(word))
        .collect::<Vec<_>>()
        .join("")
}

/// Helper function: convert a word to title case (first letter uppercase, rest lowercase).
fn title_case(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut result = first.to_uppercase().collect::<String>();
            result.push_str(&chars.collect::<String>().to_lowercase());
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_case_from_camel() {
        assert_eq!(convert_to_snake_case("helloWorld"), "hello_world");
        assert_eq!(convert_to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(convert_to_snake_case("HTTPServer"), "http_server");
        assert_eq!(convert_to_snake_case("XMLParser"), "xml_parser");
    }

    #[test]
    fn test_snake_case_from_kebab() {
        assert_eq!(convert_to_snake_case("hello-world"), "hello_world");
        assert_eq!(convert_to_snake_case("kebab-case-example"), "kebab_case_example");
    }

    #[test]
    fn test_snake_case_from_spaces() {
        assert_eq!(convert_to_snake_case("hello world"), "hello_world");
        assert_eq!(
            convert_to_snake_case("multiple space  separated words"),
            "multiple_space_separated_words"
        );
    }

    #[test]
    fn test_snake_case_empty() {
        assert_eq!(convert_to_snake_case(""), "");
    }

    #[test]
    fn test_snake_case_single_word() {
        assert_eq!(convert_to_snake_case("hello"), "hello");
        assert_eq!(convert_to_snake_case("Hello"), "hello");
    }

    #[test]
    fn test_camel_case_from_snake() {
        assert_eq!(convert_to_camel_case("hello_world"), "helloWorld");
        assert_eq!(convert_to_camel_case("hello_world_example"), "helloWorldExample");
    }

    #[test]
    fn test_camel_case_from_kebab() {
        assert_eq!(convert_to_camel_case("hello-world"), "helloWorld");
        assert_eq!(convert_to_camel_case("kebab-case-example"), "kebabCaseExample");
    }

    #[test]
    fn test_camel_case_from_pascal() {
        assert_eq!(convert_to_camel_case("HelloWorld"), "helloWorld");
        assert_eq!(convert_to_camel_case("PascalCase"), "pascalCase");
    }

    #[test]
    fn test_camel_case_empty() {
        assert_eq!(convert_to_camel_case(""), "");
    }

    #[test]
    fn test_camel_case_single_word() {
        assert_eq!(convert_to_camel_case("hello"), "hello");
        assert_eq!(convert_to_camel_case("HELLO"), "hello");
    }

    #[test]
    fn test_kebab_case_from_snake() {
        assert_eq!(convert_to_kebab_case("hello_world"), "hello-world");
        assert_eq!(convert_to_kebab_case("snake_case_example"), "snake-case-example");
    }

    #[test]
    fn test_kebab_case_from_camel() {
        assert_eq!(convert_to_kebab_case("helloWorld"), "hello-world");
        assert_eq!(convert_to_kebab_case("camelCaseExample"), "camel-case-example");
    }

    #[test]
    fn test_kebab_case_from_pascal() {
        assert_eq!(convert_to_kebab_case("HelloWorld"), "hello-world");
        assert_eq!(convert_to_kebab_case("PascalCaseExample"), "pascal-case-example");
    }

    #[test]
    fn test_kebab_case_empty() {
        assert_eq!(convert_to_kebab_case(""), "");
    }

    #[test]
    fn test_kebab_case_single_word() {
        assert_eq!(convert_to_kebab_case("hello"), "hello");
        assert_eq!(convert_to_kebab_case("HELLO"), "hello");
    }

    #[test]
    fn test_pascal_case_from_snake() {
        assert_eq!(convert_to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(convert_to_pascal_case("pascal_case_example"), "PascalCaseExample");
    }

    #[test]
    fn test_pascal_case_from_camel() {
        assert_eq!(convert_to_pascal_case("helloWorld"), "HelloWorld");
        assert_eq!(convert_to_pascal_case("camelCaseExample"), "CamelCaseExample");
    }

    #[test]
    fn test_pascal_case_from_kebab() {
        assert_eq!(convert_to_pascal_case("hello-world"), "HelloWorld");
        assert_eq!(convert_to_pascal_case("kebab-case-example"), "KebabCaseExample");
    }

    #[test]
    fn test_pascal_case_empty() {
        assert_eq!(convert_to_pascal_case(""), "");
    }

    #[test]
    fn test_pascal_case_single_word() {
        assert_eq!(convert_to_pascal_case("hello"), "Hello");
        assert_eq!(convert_to_pascal_case("HELLO"), "Hello");
    }

    #[test]
    fn test_round_trip_conversions() {
        let original = "hello_world_example";
        let to_camel = convert_to_camel_case(original);
        let back_to_snake = convert_to_snake_case(&to_camel);
        assert_eq!(back_to_snake, original);

        let to_kebab = convert_to_kebab_case(original);
        let back_to_snake2 = convert_to_snake_case(&to_kebab);
        assert_eq!(back_to_snake2, original);
    }

    #[test]
    fn test_consecutive_capitals() {
        assert_eq!(convert_to_snake_case("HTTPServer"), "http_server");
        assert_eq!(convert_to_kebab_case("XMLParser"), "xml-parser");
        assert_eq!(convert_to_camel_case("HTTP_SERVER"), "httpServer");
    }

    #[test]
    fn test_numbers_in_identifiers() {
        assert_eq!(convert_to_snake_case("hello123World"), "hello123_world");
        assert_eq!(convert_to_camel_case("hello_123_world"), "hello123World");
        assert_eq!(convert_to_kebab_case("test_2_fast"), "test-2-fast");
    }

    #[test]
    fn test_mixed_separators() {
        assert_eq!(convert_to_snake_case("hello-world_example"), "hello_world_example");
        assert_eq!(convert_to_camel_case("hello-world_example"), "helloWorldExample");
        assert_eq!(convert_to_kebab_case("hello_world-example"), "hello-world-example");
    }
}
