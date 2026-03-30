//! String joining utilities.

/// Extension trait for joining strings.
pub trait JoinExt {
    /// Join with a separator.
    fn join_with(&self, separator: &str) -> String;

    /// Join with commas.
    fn join_comma(&self) -> String;

    /// Join with "and".
    fn join_and(&self) -> String;

    /// Join with "or".
    fn join_or(&self) -> String;
}

impl<T: AsRef<str>> JoinExt for &[T] {
    fn join_with(&self, separator: &str) -> String {
        self.iter()
            .map(|s| s.as_ref())
            .collect::<Vec<_>>()
            .join(separator)
    }

    fn join_comma(&self) -> String {
        self.join_with(", ")
    }

    fn join_and(&self) -> String {
        if self.is_empty() {
            String::new()
        } else if self.len() == 1 {
            self[0].as_ref().to_string()
        } else {
            let parts: Vec<_> = self.iter().map(|s| s.as_ref()).collect();
            let last = parts.len() - 1;
            parts[..last].join(", ") + " and " + parts[last]
        }
    }

    fn join_or(&self) -> String {
        if self.is_empty() {
            String::new()
        } else if self.len() == 1 {
            self[0].as_ref().to_string()
        } else {
            let parts: Vec<_> = self.iter().map(|s| s.as_ref()).collect();
            let last = parts.len() - 1;
            parts[..last].join(", ") + " or " + parts[last]
        }
    }
}
