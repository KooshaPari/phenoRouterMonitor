//! Aggregate derive macro
//!
//! Generates aggregate root implementations for event sourcing.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_aggregate(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let aggregate_name = get_aggregate_name(&input.attrs);
    let state_type = get_state_type(&input.attrs);

    let expanded = quote! {
        impl #impl_generics Aggregate for #name #ty_generics #where_clause {
            const NAME: &'static str = #aggregate_name;
            type State = #state_type;
        }
    };

    expanded.into()
}

pub trait Aggregate: Send + Sync {
    const NAME: &'static str;
    type State: Clone + Default + serde::Serialize + for<'de> serde::Deserialize<'de>;
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ApplyError {
    #[error("Invalid event type for this aggregate")]
    InvalidEvent,
    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },
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
    "Aggregate".to_string()
}

fn get_state_type(attrs: &[syn::Attribute]) -> syn::Type {
    for attr in attrs {
        if attr.path().is_ident("state") {
            if let Ok(syn::Meta::NameValue(nv)) = attr.parse_args() {
                if let syn::Expr::Path(path) = &nv.value {
                    return syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: path.path.clone(),
                    });
                }
            }
        }
    }
    syn::parse_quote!(())
}
