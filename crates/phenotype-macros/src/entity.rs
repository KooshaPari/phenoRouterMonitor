//! Entity derive macro
//!
//! Generates entity implementations with ID, created_at, updated_at fields.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_entity(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let entity_name = get_entity_name(&input.attrs);

    let expanded = quote! {
        impl #impl_generics Entity for #name #ty_generics #where_clause {
            const ENTITY_NAME: &'static str = #entity_name;
        }

        impl #impl_generics #name #ty_generics #where_clause {
            /// Get the entity ID
            pub fn id(&self) -> &Self::Id {
                &self.id
            }

            /// Get the entity's created_at timestamp
            pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
                self.created_at
            }

            /// Get the entity's updated_at timestamp
            pub fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
                self.updated_at
            }

            /// Update the updated_at timestamp
            pub fn touch(&mut self) {
                self.updated_at = chrono::Utc::now();
            }
        }
    };

    expanded.into()
}

fn get_entity_name(attrs: &[syn::Attribute]) -> String {
    for attr in attrs {
        if attr.path().is_ident("entity") {
            if let Ok(syn::Meta::NameValue(nv)) = attr.parse_args() {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                    return s.value();
                }
            }
        }
    }
    "Entity".to_string()
}
