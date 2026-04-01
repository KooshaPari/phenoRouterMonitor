/// Builder pattern derive macro for structs with fluent interface
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Error, Fields};

/// Generates a builder struct and impl for any struct with named fields
pub fn derive(input: DeriveInput) -> TokenStream {
    match &input.data {
        Data::Struct(data) => {
            let name = &input.ident;
            let builder_name = format_ident!("{}Builder", name);
            let fields = match &data.fields {
                Fields::Named(fields) => &fields.named,
                _ => {
                    return Error::new_spanned(&input, "Builder only supports named fields")
                        .to_compile_error()
                }
            };

            let builder_field_defs = fields.iter().map(|f| {
                let field_name = &f.ident;
                let field_type = &f.ty;
                quote! { #field_name: Option<#field_type>, }
            });

            let builder_methods = fields.iter().map(|f| {
                let field_name = &f.ident;
                let field_type = &f.ty;
                quote! {
                    pub fn #field_name(mut self, #field_name: #field_type) -> Self {
                        self.#field_name = Some(#field_name);
                        self
                    }
                }
            });

            let builder_field_inits = fields.iter().map(|f| {
                let field_name = &f.ident;
                quote! { #field_name: None, }
            });

            let build_field_assignments = fields.iter().map(|f| {
                let field_name = &f.ident;
                let field_str = field_name
                    .as_ref()
                    .map(|n| n.to_string())
                    .unwrap_or_default();
                let err_msg = format!("missing field: {}", field_str);
                quote! { #field_name: self.#field_name.ok_or_else(|| #err_msg.to_string())?, }
            });

            quote! {
                pub struct #builder_name { #(#builder_field_defs)* }
                impl #builder_name {
                    pub fn new() -> Self { Self { #(#builder_field_inits)* } }
                    #(#builder_methods)*
                    pub fn build(self) -> Result<#name, String> {
                        Ok(#name { #(#build_field_assignments)* })
                    }
                }
                impl Default for #builder_name { fn default() -> Self { Self::new() } }
                impl #name { pub fn builder() -> #builder_name { #builder_name::new() } }
            }
        }
        _ => Error::new_spanned(&input, "Builder only supports structs").to_compile_error(),
    }
}
