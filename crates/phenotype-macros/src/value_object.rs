use proc_macro2::TokenStream; use quote::quote; use syn::DeriveInput;
pub fn derive(input: DeriveInput) -> TokenStream {
    let name = &input.ident; let (ig, tg, wc) = input.generics.split_for_impl();
    quote! { impl #ig ValueObject for #name #tg #wc {} impl #ig std::cmp::PartialEq for #name #tg #wc { fn eq(&self, o: &Self) -> bool { self.0 == o.0 } } impl #ig std::cmp::Eq for #name #tg #wc {} impl #ig std::hash::Hash for #name #tg #wc { fn hash<H: std::hash::Hasher>(&self, s: &mut H) { self.0.hash(s); } } }
}
