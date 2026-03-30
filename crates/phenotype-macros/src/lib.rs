//! Phenotype Macros
//!
//! This crate provides procedural macros for common Phenotype DDD patterns:
//! - Builder pattern for fluent struct construction
//! - FromStr parsing for string conversion
//! - #[async_main] for async entry points
//! - Domain-driven design macros (Entity, ValueObject, Command, etc.)
//!
//! Traces to: FR-PHENO-MACRO-001

mod aggregate;
mod async_main;
mod command;
mod derive_builder;
mod derive_errors;
mod derive_from_str;
mod derive_serde;
mod entity;
mod error;
mod event;
mod value_object;

// Domain-Driven Design Macros
#[proc_macro_derive(Entity, attributes(entity))]
pub fn derive_entity(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    entity::derive(i).into()
}

#[proc_macro_derive(ValueObject)]
pub fn derive_value_object(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    value_object::derive(i).into()
}

#[proc_macro_derive(Command, attributes(command))]
pub fn derive_command(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    command::derive(i).into()
}

#[proc_macro_derive(DomainEvent, attributes(event))]
pub fn derive_event(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    event::derive(i).into()
}

#[proc_macro_derive(Aggregate, attributes(aggregate))]
pub fn derive_aggregate(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    aggregate::derive(i).into()
}

#[proc_macro_derive(Error)]
pub fn derive_error(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    error::derive(i).into()
}

// Structural Macros
/// Builder pattern macro — generates fluent builder for structs
/// Traces to: FR-PHENO-MACRO-001
#[proc_macro_derive(Builder)]
pub fn derive_builder(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    derive_builder::derive(i).into()
}

/// FromStr trait implementation for string parsing
/// Traces to: FR-PHENO-MACRO-002
#[proc_macro_derive(FromStr, attributes(from_str))]
pub fn derive_from_str(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    derive_from_str::derive(i).into()
}

/// Enhanced serialization helpers for serde integration
#[proc_macro_derive(SerdeHelper)]
pub fn derive_serde_helper(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    derive_serde::derive(i).into()
}

/// Enhanced error type with Error trait implementation
#[proc_macro_derive(ErrorType)]
pub fn derive_error_type(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    derive_errors::derive_error_type(i).into()
}

// Attribute Macros
/// #[async_main] attribute macro for async entry points
/// Traces to: FR-PHENO-MACRO-003
#[proc_macro_attribute]
pub fn async_main(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    async_main::derive(input).into()
}
