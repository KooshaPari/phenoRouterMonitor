//! String utilities for Phenotype.

/// Convert string to a URL-friendly slug.
pub fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Truncate string to maximum length, preserving word boundaries when possible.
pub fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        return s;
    }
    let truncated = &s[..max_len];
    match truncated.rfind(' ') {
        Some(pos) => &s[..pos],
        None => truncated,
    }
}

/// Convert string to Title Case.
pub fn to_title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let rest = chars.as_str().to_lowercase();
                    first.to_uppercase().collect::<String>() + &rest
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert string to snake_case.
pub fn to_snake_case(s: &str) -> String {
    let s = s.to_lowercase();
    s.chars()
        .map(|c| if c == '-' || c == ' ' { '_' } else { c })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// Convert string to kebab-case.
pub fn to_kebab_case(s: &str) -> String {
    let s = s.to_lowercase();
    s.chars()
        .map(|c| if c == '_' || c == ' ' { '-' } else { c })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Strip ANSI escape codes from string.
pub fn strip_ansi(s: &str) -> String {
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").expect("regex is valid");
    re.replace_all(s, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Hello  World"), "hello-world");
        assert_eq!(slugify("Hello-World"), "hello-world");
        assert_eq!(slugify("The Quick Brown Fox"), "the-quick-brown-fox");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello world", 20), "hello world");
        assert_eq!(truncate("hello world test", 11), "hello");
        assert_eq!(truncate("hello world test", 6), "hello");
    }

    #[test]
    fn test_to_title_case() {
        assert_eq!(to_title_case("hello world"), "Hello World");
        assert_eq!(to_title_case("the quick brown fox"), "The Quick Brown Fox");
        assert_eq!(to_title_case("HELLO"), "Hello");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("HelloWorld"), "helloworld");
        assert_eq!(to_snake_case("hello world"), "hello_world");
        assert_eq!(to_snake_case("THE_QUICK"), "the_quick");
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("HelloWorld"), "helloworld");
        assert_eq!(to_kebab_case("hello world"), "hello-world");
        assert_eq!(to_kebab_case("THE_QUICK"), "the-quick");
    }

    #[test]
    fn test_strip_ansi() {
        let input = "\x1b[31mRed Text\x1b[0m";
        assert_eq!(strip_ansi(input), "Red Text");
        assert_eq!(strip_ansi("Normal Text"), "Normal Text");
    }
}
