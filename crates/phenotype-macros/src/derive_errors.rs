/// Enhanced error type derives with automatic Error trait implementation
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Data, Fields, Error};

pub fn derive_error_type(input: DeriveInput) -> TokenStream {
    let name = &input.ident;

    match &input.data {
        Data::Struct(data) => {
            let fields = match &data.fields {
                Fields::Named(fields) => fields,
                Fields::Unnamed(_) => {
                    return quote! {
                        impl std::fmt::Display for #name {
                            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                                write!(f, "{:?}", self)
                            }
                        }
                        impl std::fmt::Debug for #name {
                            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                                std::fmt::Display::fmt(self, f)
                            }
                        }
                        impl std::error::Error for #name {}
                    };
                }
                Fields::Unit => {
                    return quote! {
                        impl std::fmt::Display for #name {
                            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                                write!(f, "{}", stringify!(#name))
                            }
                        }
                        impl std::fmt::Debug for #name {
                            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                                std::fmt::Display::fmt(self, f)
                            }
                        }
                        impl std::error::Error for #name {}
                    };
                }
            };

            let message_field = fields.named.iter().find(|f| {
                f.ident.as_ref().map(|i| i == "message").unwrap_or(false)
            });

            let display_impl = if message_field.is_some() {
                quote! {
                    impl std::fmt::Display for #name {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            write!(f, "{}", self.message)
                        }
                    }
                }
            } else {
                quote! {
                    impl std::fmt::Display for #name {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            write!(f, "{:?}", self)
                        }
                    }
                }
            };

            quote! {
                #display_impl
                impl std::fmt::Debug for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        std::fmt::Display::fmt(self, f)
                    }
                }
                impl std::error::Error for #name {}
            }
        }
        Data::Enum(data) => {
            let display_arms = data.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                match &variant.fields {
                    Fields::Unit => {
                        quote! { Self::#variant_name => write!(f, "{}", stringify!(#variant_name)), }
                    }
                    Fields::Named(_) => {
                        quote! { Self::#variant_name { .. } => write!(f, "{}", stringify!(#variant_name)) }
                    }
                    Fields::Unnamed(_) => {
                        quote! { Self::#variant_name(..) => write!(f, "{}", stringify!(#variant_name)) }
                    }
                }
            });

            quote! {
                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self { #(#display_arms)* }
                    }
                }
                impl std::fmt::Debug for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        std::fmt::Display::fmt(self, f)
                    }
                }
                impl std::error::Error for #name {}
            }
        }
        _ => Error::new_spanned(&input, "ErrorType only supports structs and enums")
            .to_compile_error(),
    }
}
