//! Implementation of the ErrorContext derive macro.

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{Data, DeriveInput, Fields};

/// Implementation of the ErrorContext derive macro.
pub fn impl_error_context(input: proc_macro::TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Extract prefix from attribute if present
    let prefix = extract_prefix(&input.attrs);
    let prefix_upper = prefix.to_uppercase();
    let prefix_lower = prefix.to_lowercase();

    let data = &input.data;

    let variants = match data {
        Data::Enum(data) => &data.variants,
        _ => {
            return syn::Error::new_spanned(
                input,
                "ErrorContext only supports enums",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate context methods for each variant
    let context_methods: Vec<_> = variants
        .iter()
        .filter_map(|variant| {
            let variant_name = &variant.ident;
            let method_name = format_ident!("{}", variant_name.to_string().to_snake_case());

            match &variant.fields {
                Fields::Unnamed(fields) => {
                    // Tuple variant like NotFound(id)
                    let field_types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();
                    let field_indices: Vec<_> = (0..fields.unnamed.len())
                        .map(|i| format_ident!("_{}", i))
                        .collect();

                    Some(quote! {
                        /// Create a #variant_name error
                        pub fn #method_name(#(#field_indices: #field_types),*) -> Self {
                            Self::#variant_name(#(#field_indices),*)
                        }
                    })
                }
                Fields::Named(fields) => {
                    // Struct variant like NotFound { id }
                    let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();

                    Some(quote! {
                        /// Create a #variant_name error
                        pub fn #method_name(#(#field_names: impl Into<String>),*) -> Self {
                            Self::#variant_name {
                                #(#field_names: #field_names.into()),*
                            }
                        }
                    })
                }
                Fields::Unit => {
                    // Unit variant like Unauthorized
                    Some(quote! {
                        /// Create a #variant_name error
                        pub fn #method_name() -> Self {
                            Self::#variant_name
                        }
                    })
                }
            }
        })
        .collect();

    // Generate error code constants
    let error_codes: Vec<_> = variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let code = format!("{}_{}", prefix_upper, variant_name.to_string().to_uppercase());
            quote! {
                /// Error code for #variant_name
                pub const #variant_name: &'static str = #code;
            }
        })
        .collect();

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #![doc = " Error prefix: #prefix"]
            #![doc = " "]

            /// Create error with request context
            pub fn with_request_id(self, request_id: impl Into<String>) -> Self {
                self
            }

            /// Create error with user context
            pub fn with_user_id(self, user_id: impl Into<String>) -> Self {
                self
            }

            /// Get the error code for this error type
            pub fn error_code(&self) -> &'static str {
                match self {
                    #(
                        Self::#variants => #prefix_upper,
                    )*
                }
            }

            #(#context_methods)*
            #(#error_codes)*
        }
    }
    .into()
}

/// Extract prefix from #[error_prefix = "..."] attribute.
fn extract_prefix(attrs: &[syn::Attribute]) -> String {
    for attr in attrs {
        if attr.path().is_ident("error_prefix") {
            if let Ok(syn::Meta::NameValue(meta)) = attr.parse_meta() {
                if let syn::Lit::Str(lit) = &meta.lit {
                    return lit.value();
                }
            }
        }
    }
    "ERR".to_string()
}
