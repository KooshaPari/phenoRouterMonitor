/// #[async_main] attribute macro for async entry points
/// Traces to: FR-PHENO-MACRO-003
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, ItemFn, ReturnType, Type};

/// Transforms an async fn main() into a tokio runtime entry point
///
/// Usage:
/// ```ignore
/// #[async_main]
/// async fn main() {
///     // async code here
/// }
/// ```
///
/// Expands to:
/// ```ignore
/// fn main() {
///     tokio::runtime::Runtime::new().unwrap().block_on(async {
///         // async code here
///     })
/// }
/// ```
pub fn derive(input: ItemFn) -> TokenStream {
    // Validate this is the main function
    if input.sig.ident != "main" {
        return Error::new_spanned(
            &input.sig.ident,
            "#[async_main] can only be applied to the main function",
        )
        .to_compile_error();
    }

    // Validate it's async
    if input.sig.asyncness.is_none() {
        return Error::new_spanned(&input.sig, "#[async_main] requires an async function")
            .to_compile_error();
    }

    // Validate no parameters
    if !input.sig.inputs.is_empty() {
        return Error::new_spanned(
            &input.sig,
            "#[async_main] main function must have no parameters",
        )
        .to_compile_error();
    }

    // Check if return type is Result or ()
    let is_result_return =
        matches!(&input.sig.output, ReturnType::Type(_, ty) if type_is_result(ty));

    let body = &input.block;

    if is_result_return {
        quote! {
            fn main() {
                if let Err(e) = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(async { #body })
                {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        quote! {
            fn main() {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(async { #body })
            }
        }
    }
}

fn type_is_result(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Result";
        }
    }
    false
}
