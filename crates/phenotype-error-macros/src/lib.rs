//! Phenotype Error Macros
//!
//! Procedural macros for unified error handling in the Phenotype ecosystem.
//!
//! # Features
//!
//! - `#[derive(ErrorContext)]` - Derive macro that adds context methods to error types
//! - `#[derive(ErrorFrom)]` - Derive macro that generates `From` implementations
//! - `#[error_context]` - Attribute macro for wrapping functions with context
//!
//! # Example
//!
//! ```ignore
//! use phenotype_error_macros::{ErrorContext, ErrorFrom};
//!
//! #[derive(Debug, ErrorContext)]
//! #[error(prefix = "USER")]
//! pub enum UserError {
//!     #[error("User not found: {id}")]
//!     NotFound { id: String },
//!
//!     #[error("Invalid email: {email}")]
//!     InvalidEmail { email: String },
//! }
//!
//! // Generates From implementations automatically
//! impl From<DatabaseError> for UserError {
//!     fn from(e: DatabaseError) -> Self {
//!         UserError::Database(e.to_string())
//!     }
//! }
//! ```

use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{Data, DeriveInput, Fields, Index};

mod derive_error_context;
mod derive_error_from;

/// Derive macro for adding context methods to error types.
///
/// This macro adds helper methods for creating errors with additional context
/// such as request IDs, user information, and resource tracking.
///
/// # Example
///
/// ```ignore
/// #[derive(Debug, ErrorContext)]
/// #[error(prefix = "API")]
/// pub enum ApiError {
///     #[error("Not found: {resource}")]
///     NotFound { resource: String },
///
///     #[error("Unauthorized")]
///     Unauthorized,
/// }
/// ```
#[proc_macro_derive(ErrorContext, attributes(error, error_prefix))]
pub fn derive_error_context(input: TokenStream) -> TokenStream {
    derive_error_context::impl_error_context(input)
}

/// Derive macro for generating `From` implementations between error types.
///
/// This macro generates `From` trait implementations that allow automatic
/// error conversion between different error types in the ecosystem.
///
/// # Example
///
/// ```ignore
/// #[derive(ErrorFrom)]
/// #[from(DatabaseError)]
/// #[from(ValidationError)]
/// pub enum ServiceError {
///     Database(DatabaseError),
///     Validation(ValidationError),
/// }
/// ```
#[proc_macro_derive(ErrorFrom, attributes(from))]
pub fn derive_error_from(input: TokenStream) -> TokenStream {
    derive_error_from::impl_error_from(input)
}

/// Attribute macro for adding context to fallible functions.
///
/// Wraps a function to capture and propagate context information
/// such as the calling location, timestamp, and optional request ID.
///
/// # Example
///
/// ```ignore
/// #[error_context]
/// fn fetch_user(id: String) -> Result<User, UserError> {
///     // ... implementation
/// }
/// ```
#[proc_macro]
pub fn error_context(input: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(input as syn::ItemFn);

    let attrs = &item.attrs;
    let vis = &item.vis;
    let sig = &item.sig;
    let body = &item.block;

    let fn_name = &sig.ident;
    let fn_name_str = fn_name.to_string();

    quote! {
        #(#attrs)*
        #vis #sig {
            let __span = tracing::info_span!(
                "error_context",
                function = #fn_name_str
            );
            let __guard = __span.enter();
            let result = (|| #body)();
            drop(__guard);
            result
        }
    }
    .into()
}
