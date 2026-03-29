//! # phenotype-macros
//! Procedural macros for the Phenotype ecosystem

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for implementing error types
#[proc_macro_derive(ErrorDerive, attributes(error_code, status_code))]
pub fn error_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    quote! {
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
        
        impl std::error::Error for #name {}
    }
    .into()
}

/// Macro for defining result types
#[proc_macro]
pub fn define_result(input: TokenStream) -> TokenStream {
    let ty = syn::parse_macro_input!(input as syn::Type);
    quote! {
        pub type Result<T = #ty> = std::result::Result<T, #ty>;
    }.into()
}
