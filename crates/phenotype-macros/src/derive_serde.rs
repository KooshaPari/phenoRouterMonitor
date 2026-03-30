/// Enhanced serialization helpers for serde integration  
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error};

pub fn derive(input: DeriveInput) -> TokenStream {
    match &input.data {
        Data::Struct(_) => {
            let name = &input.ident;
            quote! {
                impl #name {
                    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
                        serde_json::from_str(json)
                    }
                    pub fn to_json(&self) -> Result<String, serde_json::Error> {
                        serde_json::to_string(self)
                    }
                    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
                        serde_json::to_string_pretty(self)
                    }
                }
            }
        }
        _ => Error::new_spanned(&input, "SerdeHelper only supports structs").to_compile_error(),
    }
}
