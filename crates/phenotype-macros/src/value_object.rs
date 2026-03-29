//! Value Object derive macro
//!
//! Generates value object implementations with comparison.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_value_object(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let vo_name = get_value_object_name(&input.attrs);

    let expanded = quote! {
        impl #impl_generics ValueObject for #name #ty_generics #where_clause {
            const NAME: &'static str = #vo_name;
        }

        impl #impl_generics std::cmp::PartialEq for #name #ty_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl #impl_generics std::cmp::Eq for #name #ty_generics #where_clause {}

        impl #impl_generics std::hash::Hash for #name #ty_generics #where_clause {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }
    };

    expanded.into()
}

fn get_value_object_name(attrs: &[syn::Attribute]) -> String {
    for attr in attrs {
        if attr.path().is_ident("value_object") {
            if let Ok(syn::Meta::NameValue(nv)) = attr.parse_args() {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                    return s.value();
                }
            }
        }
    }
    "ValueObject".to_string()
}
