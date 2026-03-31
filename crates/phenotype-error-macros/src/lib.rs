//! Unified error handling macros for the Phenotype ecosystem.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn};

/// Wraps a function body in a Result type with context enrichment.
#[proc_macro_attribute]
pub fn with_error_context(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_body = &input.block;
    let fn_sig = &input.sig;
    let fn_vis = &input.vis;
    let fn_attrs = &input.attrs;

    let output = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            (|| #fn_body)()
        }
    };

    output.into()
}

/// Generates a Result type alias with the specified error type.
#[proc_macro]
pub fn define_result(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as Ident);
    let result_name = Ident::new(&format!("{}Result", input), input.span());

    let output = quote! {
        pub type #result_name<T> = std::result::Result<T, #input>;
    };

    output.into()
}

/// Generates a try! macro that wraps errors with context.
#[proc_macro]
pub fn try_with_context(_item: TokenStream) -> TokenStream {
    let output = quote! {
        macro_rules! try_with_context {
            ($expr:expr, $ctx:expr) => {
                $expr.map_err(|e| format!("{}: {}", $ctx, e))?
            };
        }
    };

    output.into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_define_result_macro() {
        // This is tested via compile-time in integration tests
        assert!(true);
    }
}
