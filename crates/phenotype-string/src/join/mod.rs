//! String joining utilities.

use std::fmt::Display;

/// Joins strings with a separator, handling empty strings gracefully.
pub fn join_with(separator: &str, parts: &[impl AsRef<str>]) -> String {
    if parts.is_empty() {
        String::new()
    } else {
        parts
            .iter()
            .map(|p| p.as_ref())
            .collect::<Vec<_>>()
            .join(separator)
    }
}

/// Joins strings with oxford comma formatting (a, b, c, and d).
pub fn join_oxford(parts: &[impl Display]) -> String {
    match parts.len() {
        0 => String::new(),
        1 => format!("{}", parts[0]),
        2 => format!("{} and {}", parts[0], parts[1]),
        _ => {
            let all_but_last = &parts[..parts.len() - 1];
            let last = &parts[parts.len() - 1];
            let all_but_last_str = all_but_last
                .iter()
                .map(|p| format!("{}", p))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}, and {}", all_but_last_str, last)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_with_empty() {
        assert_eq!(join_with(", ", &[] as &[&str]), "");
        assert_eq!(join_with(", ", &["a"]), "a");
        assert_eq!(join_with(", ", &["a", "b"]), "a, b");
    }

    #[test]
    fn test_join_oxford() {
        assert_eq!(join_oxford(&[] as &[&str]), "");
        assert_eq!(join_oxford(&["apple"]), "apple");
        assert_eq!(join_oxford(&["apple", "banana"]), "apple and banana");
        assert_eq!(
            join_oxford(&["apple", "banana", "cherry"]),
            "apple, banana, and cherry"
        );
    }
}
