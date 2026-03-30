/// Enhanced error type derives with automatic Error trait implementation
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields};

pub fn derive_error_type(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    match &input.data {
        Data::Struct(data) => {
            let fields = match &data.fields {
                Fields::Named(fields) => fields,
                Fields::Unnamed(_) => {
                    return quote! {
                        impl std::fmt::Display for #name { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) } }
                        impl std::fmt::Debug for #name { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { std::fmt::Display::fmt(self, f) } }
                        impl std::error::Error for #name {}
                    };
                }
                Fields::Unit => {
                    return quote! {
                        impl std::fmt::Display for #name { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", stringify!(#name)) } }
                        impl std::fmt::Debug for #name { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { std::fmt::Display::fmt(self, f) } }
                        impl std::error::Error for #name {}
                    };
                }
            };

            let has_message_field = fields
                .named
                .iter()
                .any(|f| f.ident.as_ref().map(|i| i == "message").unwrap_or(false));
            let display = if has_message_field {
                quote! { impl std::fmt::Display for #name { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.message) } } }
            } else {
                quote! { impl std::fmt::Display for #name { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) } } }
            };

            quote! {
                #display
                impl std::fmt::Debug for #name { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { std::fmt::Display::fmt(self, f) } }
                impl std::error::Error for #name {}
            }
        }
        Data::Enum(data) => {
            let arms = data.variants.iter().map(|v| {
                let vname = &v.ident;
                match &v.fields {
                    Fields::Unit => quote! { Self::#vname => write!(f, "{}", stringify!(#vname)), },
                    Fields::Named(_) => {
                        quote! { Self::#vname { .. } => write!(f, "{}", stringify!(#vname)) }
                    }
                    Fields::Unnamed(_) => {
                        quote! { Self::#vname(..) => write!(f, "{}", stringify!(#vname)) }
                    }
                }
            });
            quote! {
                impl std::fmt::Display for #name { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match self { #(#arms)* } } }
                impl std::fmt::Debug for #name { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { std::fmt::Display::fmt(self, f) } }
                impl std::error::Error for #name {}
            }
        }
        _ => Error::new_spanned(&input, "ErrorType only supports structs/enums").to_compile_error(),
    }
}
