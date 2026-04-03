//! Implementation of the ErrorFrom derive macro.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Index};

/// Implementation of the ErrorFrom derive macro.
pub fn impl_error_from(input: proc_macro::TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let data = &input.data;

    let from_impls = match data {
        Data::Enum(data) => {
            let variants = &data.variants;

            // Extract #[from(...)] attributes from the enum
            let from_types: Vec<_> = input
                .attrs
                .iter()
                .filter_map(|attr| {
                    if attr.path().is_ident("from") {
                        if let Ok(syn::Meta::List(meta)) = attr.parse_meta() {
                            return Some(meta.tokens.to_string());
                        }
                    }
                    None
                })
                .collect();

            // Generate From implementations for each variant
            let variant_froms: Vec<_> = variants
                .iter()
                .filter_map(|variant| {
                    let variant_name = &variant.ident;

                    match &variant.fields {
                        Fields::Unnamed(fields) => {
                            // Only handle single-field tuple variants for From
                            if fields.unnamed.len() == 1 {
                                let field = fields.unnamed.first().unwrap();
                                Some(quote! {
                                    impl #impl_generics From<#field.ty> for #name #ty_generics #where_clause {
                                        fn from(err: #field.ty) -> Self {
                                            Self::#variant_name(err)
                                        }
                                    }
                                })
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                })
                .collect();

            quote! {
                #(#variant_froms)*
            }
        }
        _ => {
            return syn::Error::new_spanned(
                input,
                "ErrorFrom only supports enums",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate a blanket From implementation for Box<dyn Error>
    quote! {
        #from_impls

        impl #impl_generics From<Box<dyn std::error::Error + Send + Sync>>
            for #name #ty_generics #where_clause
        {
            fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
                Self::Other(err.to_string())
            }
        }

        impl #impl_generics From<String> for #name #ty_generics #where_clause {
            fn from(msg: String) -> Self {
                Self::Message(msg)
            }
        }

        impl #impl_generics From<&str> for #name #ty_generics #where_clause {
            fn from(msg: &str) -> Self {
                Self::Message(msg.to_string())
            }
        }
    }
    .into()
}
