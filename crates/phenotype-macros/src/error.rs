use proc_macro2::TokenStream; use quote::quote; use syn::DeriveInput;
pub fn derive(input: DeriveInput) -> TokenStream {
    let name = &input.ident; let (ig, tg, wc) = input.generics.split_for_impl();
    quote! { impl #ig std::fmt::Debug for #name #tg #wc { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { std::fmt::Display::fmt(self, f) } } impl #ig std::error::Error for #name #tg #wc {} }.into()
}
