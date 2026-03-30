/// FromStr derive macro for string parsing
/// Traces to: FR-PHENO-MACRO-002
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Lit, Meta};

/// Generates FromStr trait implementation for structs with a single string-like field
/// or enums with From/Try variants
///
/// For structs: converts from String to the struct type
/// For enums: matches string values to enum variants
pub fn derive(input: DeriveInput) -> TokenStream {
    match &input.data {
        Data::Struct(data) => derive_struct(&input.ident, data),
        Data::Enum(data) => derive_enum(&input.ident, data),
        Data::Union(_) => {
            Error::new_spanned(&input, "FromStr does not support unions").to_compile_error()
        }
    }
}

fn derive_struct(name: &syn::Ident, data: &syn::DataStruct) -> TokenStream {
    match &data.fields {
        Fields::Named(fields) => {
            // For single field, parse directly; for multiple fields, parse as JSON
            if fields.named.len() == 1 {
                let field = fields.named.iter().next().unwrap();
                let field_name = &field.ident;
                let field_type = &field.ty;

                quote! {
                    impl std::str::FromStr for #name {
                        type Err = String;

                        fn from_str(s: &str) -> Result<Self, Self::Err> {
                            let value: #field_type = s.parse()
                                .map_err(|_| format!("failed to parse '{}' as {}", s, stringify!(#field_type)))?;
                            Ok(#name { #field_name: value })
                        }
                    }
                }
            } else {
                // Multiple fields: parse as JSON
                quote! {
                    impl std::str::FromStr for #name {
                        type Err = String;

                        fn from_str(s: &str) -> Result<Self, Self::Err> {
                            serde_json::from_str(s)
                                .map_err(|e| format!("JSON parse error: {}", e))
                        }
                    }
                }
            }
        }
        Fields::Unnamed(_) => {
            quote! {
                impl std::str::FromStr for #name {
                    type Err = String;

                    fn from_str(s: &str) -> Result<Self, Self::Err> {
                        serde_json::from_str(s)
                            .map_err(|e| format!("JSON parse error: {}", e))
                    }
                }
            }
        }
        Fields::Unit => {
            quote! {
                impl std::str::FromStr for #name {
                    type Err = String;

                    fn from_str(s: &str) -> Result<Self, Self::Err> {
                        if s.is_empty() || s == stringify!(#name) {
                            Ok(#name)
                        } else {
                            Err(format!("expected {}, got '{}'", stringify!(#name), s))
                        }
                    }
                }
            }
        }
    }
}

fn derive_enum(name: &syn::Ident, data: &syn::DataEnum) -> TokenStream {
    let match_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_str = variant_name.to_string();

        // Check for #[from_str = "..."] attribute
        let variant_match = variant
            .attrs
            .iter()
            .find_map(|attr| {
                if attr.path().is_ident("from_str") {
                    if let Meta::NameValue(nv) = &attr.meta {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: Lit::Str(lit_str),
                            ..
                        }) = &nv.value
                        {
                            return Some(lit_str.value());
                        }
                    }
                }
                None
            })
            .unwrap_or(variant_str.clone());

        match &variant.fields {
            Fields::Unit => {
                quote! {
                    #variant_match => Ok(#name::#variant_name),
                }
            }
            _ => {
                quote! {
                    _ => Err(format!("unknown variant: {}", #variant_match)),
                }
            }
        }
    });

    quote! {
        impl std::str::FromStr for #name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#match_arms)*
                    _ => Err(format!("unknown variant: {}", s)),
                }
            }
        }
    }
}
