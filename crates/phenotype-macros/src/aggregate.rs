use proc_macro2::TokenStream; use quote::quote; use syn::DeriveInput;
pub fn derive(input: DeriveInput) -> TokenStream {
    let name = &input.ident; let (ig, tg, wc) = input.generics.split_for_impl();
    quote! { impl #ig Aggregate for #name #tg #wc {} impl #ig #name #tg #wc { pub fn aggregate_name() -> &'static str { stringify!(#name) } } }.into()
}
