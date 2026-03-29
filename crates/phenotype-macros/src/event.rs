//! Event derive macro
//!
//! Generates domain event implementations for event sourcing.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_event(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let event_name = get_event_name(&input.attrs);
    let aggregate_name = get_aggregate_name(&input.attrs);

    let expanded = quote! {
        impl #impl_generics DomainEvent for #name #ty_generics #where_clause {
            const EVENT_TYPE: &'static str = #event_name;
            const AGGREGATE: &'static str = #aggregate_name;
        }

        impl #impl_generics #name #ty_generics #where_clause {
            /// Get event type
            pub fn event_type() -> &'static str {
                #event_name
            }

            /// Get aggregate name
            pub fn aggregate() -> &'static str {
                #aggregate_name
            }

            /// Get occurred_at timestamp
            pub fn occurred_at(&self) -> chrono::DateTime<chrono::Utc> {
                self.occurred_at
            }

            /// Get event ID
            pub fn id(&self) -> uuid::Uuid {
                self.id
            }

            /// Convert to JSON
            pub fn to_json(&self) -> Result<String, serde_json::Error> {
                serde_json::to_string(self)
            }

            /// Convert from JSON
            pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
                serde_json::from_str(json)
            }
        }
    };

    expanded.into()
}

pub trait DomainEvent: Send + Sync + Clone + serde::Serialize + for<'de> serde::Deserialize<'de> {
    const EVENT_TYPE: &'static str;
    const AGGREGATE: &'static str;

    fn id(&self) -> uuid::Uuid;
    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc>;
    fn event_type() -> &'static str { Self::EVENT_TYPE }
    fn aggregate() -> &'static str { Self::AGGREGATE }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum EventError {
    #[error("Deserialization failed: {0}")]
    Deserialization(String),
    #[error("Invalid event type: {expected}")]
    InvalidType { expected: String },
    #[error("Event version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },
}

fn get_event_name(attrs: &[syn::Attribute]) -> String {
    for attr in attrs {
        if attr.path().is_ident("event") {
            if let Ok(syn::Meta::NameValue(nv)) = attr.parse_args() {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                    return s.value();
                }
            }
        }
    }
    "Event".to_string()
}

fn get_aggregate_name(attrs: &[syn::Attribute]) -> String {
    for attr in attrs {
        if attr.path().is_ident("aggregate") {
            if let Ok(syn::Meta::NameValue(nv)) = attr.parse_args() {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                    return s.value();
                }
            }
        }
    }
    "Unknown".to_string()
}
