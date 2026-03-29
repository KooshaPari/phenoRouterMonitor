//! # Phenotype Macros
//!
//! Procedural macros for domain-driven design patterns.
//!
//! # Features
//!
//! - `#[derive(Entity)]` - Domain entities with ID and timestamps
//! - `#[derive(ValueObject)]` - Immutable value objects
//! - `#[derive(Command)]` - CQRS commands
//! - `#[derive(DomainEvent)]` - Event sourcing events
//! - `#[derive(Aggregate)]` - Aggregate roots
//! - `#[derive(Error)]` - User-facing errors
//!
//! # Example
//!
//! ```rust,ignore
//! use phenotype_macros::{Entity, ValueObject, Command, DomainEvent};
//!
//! #[derive(Entity)]
//! #[entity(name = "user")]
//! struct User {
//!     id: Uuid,
//!     email: String,
//!     created_at: DateTime<Utc>,
//!     updated_at: DateTime<Utc>,
//! }
//!
//! #[derive(ValueObject)]
//! #[value_object(name = "email")]
//! struct Email(String);
//!
//! #[derive(Command)]
//! #[command(name = "create_user")]
//! struct CreateUserCommand {
//!     email: String,
//! }
//!
//! #[derive(DomainEvent)]
//! #[event(name = "user_created")]
//! #[aggregate(name = "user")]
//! struct UserCreated {
//!     id: Uuid,
//!     email: String,
//!     occurred_at: DateTime<Utc>,
//! }
//! ```

mod aggregate;
mod command;
mod entity;
mod error;
mod event;
mod value_object;

// Export derive macros
#[proc_macro_derive(Entity, attributes(entity))]
pub fn derive_entity(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    entity::derive_entity(input).into()
}

#[proc_macro_derive(ValueObject, attributes(value_object))]
pub fn derive_value_object(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    value_object::derive_value_object(input).into()
}

#[proc_macro_derive(Command, attributes(command))]
pub fn derive_command(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    command::derive_command(input).into()
}

#[proc_macro_derive(DomainEvent, attributes(event, aggregate))]
pub fn derive_event(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    event::derive_event(input).into()
}

#[proc_macro_derive(Aggregate, attributes(aggregate, state))]
pub fn derive_aggregate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    aggregate::derive_aggregate(input).into()
}

#[proc_macro_derive(Error)]
pub fn derive_error(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    error::derive_error(input).into()
}
