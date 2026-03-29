//! # phenotype-macros
//! Procedural macros for the Phenotype ecosystem

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for implementing error types
#[proc_macro_derive(ErrorDerive)]
pub fn error_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    quote! {
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
        
        impl std::error::Error for #name {}
    }.into()
}
