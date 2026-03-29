//! Command derive macro
//!
//! Generates command handler implementations for CQRS pattern.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_command(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let command_name = get_command_name(&input.attrs);

    let expanded = quote! {
        impl #impl_generics Command for #name #ty_generics #where_clause {
            const NAME: &'static str = #command_name;
        }

        impl #impl_generics #name #ty_generics #where_clause {
            /// Get command name
            pub fn command_name() -> &'static str {
                #command_name
            }
        }
    };

    expanded.into()
}

fn get_command_name(attrs: &[syn::Attribute]) -> String {
    for attr in attrs {
        if attr.path().is_ident("command") {
            if let Ok(syn::Meta::NameValue(nv)) = attr.parse_args() {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                    return s.value();
                }
            }
        }
    }
    "Command".to_string()
}
