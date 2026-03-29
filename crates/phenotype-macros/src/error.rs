//! Error derive macro
//!
//! Generates error implementations with source tracking and context.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_error(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics std::fmt::Debug for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(self, f)
            }
        }

        impl #impl_generics std::error::Error for #name #ty_generics #where_clause {}
    };

    expanded.into()
}
