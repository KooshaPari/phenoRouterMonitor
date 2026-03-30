//! # Phenotype String
//!
//! String utilities: slugify, truncate, case conversion, and sanitization.

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
    // Find a char boundary
    let end = s.floor_char_boundary(end);
    format!("{}{}", &s[..end], suffix)
}

/// Convert a string to snake_case.
pub fn to_snake_case(s: &str) -> String {
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

/// Convert a string to PascalCase.
pub fn to_pascal_case(s: &str) -> String {
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
