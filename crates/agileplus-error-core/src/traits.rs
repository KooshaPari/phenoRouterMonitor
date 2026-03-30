//! Trait definitions for error handling and conversion.

/// Marker trait for errors that represent "not found" conditions.
pub trait NotFoundMarker: std::error::Error {
    /// Returns true if this error represents a not found condition.
    fn is_not_found(&self) -> bool;
}

/// Trait for errors that can be converted to an error kind string.
pub trait ErrorKindProvider: std::error::Error {
    /// Get the kind of this error as a string.
    fn kind(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_kind_provider_trait_exists() {
        // This is a compile-time test that the trait is defined correctly
        let _: &dyn ErrorKindProvider;
    }
}
