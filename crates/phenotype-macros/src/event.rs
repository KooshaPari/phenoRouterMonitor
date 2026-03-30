use proc_macro2::TokenStream; use quote::quote; use syn::DeriveInput;
pub fn derive(input: DeriveInput) -> TokenStream {
    let name = &input.ident; let (ig, tg, wc) = input.generics.split_for_impl();
    quote! { impl #ig DomainEvent for #name #tg #wc {} impl #ig #name #tg #wc { pub fn event_type() -> &'static str { stringify!(#name) } } }
}
