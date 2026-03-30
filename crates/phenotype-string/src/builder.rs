//! Efficient string building utilities.
//!
//! Provides a `StringBuilder` for efficient string concatenation with methods
//! for common operations like joining, repeating, and conditional appending.

use std::fmt::Write;

/// Efficiently builds strings with various operations.
///
/// `StringBuilder` is designed for efficient string concatenation, especially
/// when building strings from many parts or with repeated operations.
///
/// # Examples
/// ```
/// use phenotype_string::StringBuilder;
///
/// let result = StringBuilder::new()
///     .append("Hello")
///     .append(" ")
///     .append("World")
///     .build();
/// assert_eq!(result, "Hello World");
/// ```
#[derive(Clone)]
pub struct StringBuilder {
    buffer: String,
}

impl StringBuilder {
    /// Creates a new empty StringBuilder.
    pub fn new() -> Self {
        StringBuilder {
            buffer: String::new(),
        }
    }

    /// Creates a new StringBuilder with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        StringBuilder {
            buffer: String::with_capacity(capacity),
        }
    }

    /// Appends a string slice to the builder.
    ///
    /// # Examples
    /// ```
    /// use phenotype_string::StringBuilder;
    /// let result = StringBuilder::new()
    ///     .append("Hello")
    ///     .append(" ")
    ///     .append("World")
    ///     .build();
    /// assert_eq!(result, "Hello World");
    /// ```
    pub fn append<S: AsRef<str>>(mut self, s: S) -> Self {
        self.buffer.push_str(s.as_ref());
        self
    }

    /// Appends a single character to the builder.
    ///
    /// # Examples
    /// ```
    /// use phenotype_string::StringBuilder;
    /// let result = StringBuilder::new()
    ///     .append("Hello")
    ///     .append_char('!')
    ///     .build();
    /// assert_eq!(result, "Hello!");
    /// ```
    pub fn append_char(mut self, c: char) -> Self {
        self.buffer.push(c);
        self
    }

    /// Appends a newline character to the builder.
    ///
    /// # Examples
    /// ```
    /// use phenotype_string::StringBuilder;
    /// let result = StringBuilder::new()
    ///     .append("Line 1")
    ///     .newline()
    ///     .append("Line 2")
    ///     .build();
    /// assert_eq!(result, "Line 1\nLine 2");
    /// ```
    pub fn newline(mut self) -> Self {
        self.buffer.push('\n');
        self
    }

    /// Appends a formatted string to the builder.
    ///
    /// # Examples
    /// ```
    /// use phenotype_string::StringBuilder;
    /// let result = StringBuilder::new()
    ///     .format(format_args!("Value: {}", 42))
    ///     .build();
    /// assert_eq!(result, "Value: 42");
    /// ```
    pub fn format(mut self, args: std::fmt::Arguments) -> Self {
        let _ = write!(self.buffer, "{}", args);
        self
    }

    /// Appends a string multiple times.
    ///
    /// # Examples
    /// ```
    /// use phenotype_string::StringBuilder;
    /// let result = StringBuilder::new()
    ///     .repeat("ab", 3)
    ///     .build();
    /// assert_eq!(result, "ababab");
    /// ```
    pub fn repeat<S: AsRef<str>>(mut self, s: S, count: usize) -> Self {
        for _ in 0..count {
            self.buffer.push_str(s.as_ref());
        }
        self
    }

    /// Joins multiple strings with a separator.
    ///
    /// # Examples
    /// ```
    /// use phenotype_string::StringBuilder;
    /// let result = StringBuilder::new()
    ///     .join(&["world", "!"], "hello ")
    ///     .build();
    /// assert_eq!(result, "helloworld !world !");
    /// ```
    pub fn join<I, S>(mut self, parts: I, separator: &str) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut first = true;
        for part in parts {
            if !first {
                self.buffer.push_str(separator);
            }
            self.buffer.push_str(part.as_ref());
            first = false;
        }
        self
    }

    /// Appends a string if a condition is true.
    ///
    /// # Examples
    /// ```
    /// use phenotype_string::StringBuilder;
    /// let result = StringBuilder::new()
    ///     .append("Hello")
    ///     .append_if(true, " World")
    ///     .append_if(false, "!")
    ///     .build();
    /// assert_eq!(result, "Hello World");
    /// ```
    pub fn append_if<S: AsRef<str>>(mut self, condition: bool, s: S) -> Self {
        if condition {
            self.buffer.push_str(s.as_ref());
        }
        self
    }

    /// Appends a character if a condition is true.
    pub fn append_char_if(mut self, condition: bool, c: char) -> Self {
        if condition {
            self.buffer.push(c);
        }
        self
    }

    /// Returns the current length of the builder's buffer.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns true if the builder is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Clears the builder.
    pub fn clear(mut self) -> Self {
        self.buffer.clear();
        self
    }

    /// Returns a reference to the current buffer.
    pub fn as_str(&self) -> &str {
        &self.buffer
    }

    /// Consumes the builder and returns the built string.
    pub fn build(self) -> String {
        self.buffer
    }
}

impl Default for StringBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<str> for StringBuilder {
    fn as_ref(&self) -> &str {
        &self.buffer
    }
}

impl From<StringBuilder> for String {
    fn from(builder: StringBuilder) -> Self {
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append() {
        let result = StringBuilder::new()
            .append("Hello")
            .append(" ")
            .append("World")
            .build();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_append_char() {
        let result = StringBuilder::new()
            .append("Hello")
            .append_char('!')
            .build();
        assert_eq!(result, "Hello!");
    }

    #[test]
    fn test_newline() {
        let result = StringBuilder::new()
            .append("Line 1")
            .newline()
            .append("Line 2")
            .build();
        assert_eq!(result, "Line 1\nLine 2");
    }

    #[test]
    fn test_format() {
        let result = StringBuilder::new()
            .format(format_args!("Value: {}", 42))
            .build();
        assert_eq!(result, "Value: 42");
    }

    #[test]
    fn test_repeat() {
        let result = StringBuilder::new().repeat("ab", 3).build();
        assert_eq!(result, "ababab");
    }

    #[test]
    fn test_join() {
        let parts = vec!["world", "!"];
        let result = StringBuilder::new()
            .join(parts, " ")
            .build();
        assert_eq!(result, "world !");
    }

    #[test]
    fn test_append_if_true() {
        let result = StringBuilder::new()
            .append("Hello")
            .append_if(true, " World")
            .build();
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_append_if_false() {
        let result = StringBuilder::new()
            .append("Hello")
            .append_if(false, " World")
            .build();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_append_char_if() {
        let result = StringBuilder::new()
            .append("Hello")
            .append_char_if(true, '!')
            .build();
        assert_eq!(result, "Hello!");
    }

    #[test]
    fn test_len() {
        let builder = StringBuilder::new().append("Hello");
        assert_eq!(builder.len(), 5);
    }

    #[test]
    fn test_is_empty() {
        assert!(StringBuilder::new().is_empty());
        assert!(!StringBuilder::new().append("x").is_empty());
    }

    #[test]
    fn test_clear() {
        let result = StringBuilder::new()
            .append("Hello")
            .clear()
            .append("World")
            .build();
        assert_eq!(result, "World");
    }

    #[test]
    fn test_as_str() {
        let builder = StringBuilder::new().append("Hello");
        assert_eq!(builder.as_str(), "Hello");
    }

    #[test]
    fn test_from_string_builder() {
        let builder = StringBuilder::new().append("Hello");
        let s: String = builder.into();
        assert_eq!(s, "Hello");
    }
}
