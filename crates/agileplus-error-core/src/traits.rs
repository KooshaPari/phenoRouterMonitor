/// Marker for error types that can represent a missing resource.
pub trait NotFoundMarker {
    /// Build a not-found style error for the given identifier or message.
    fn not_found(msg: impl Into<String>) -> Self;
}
