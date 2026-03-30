//! Phenotype Macros
mod aggregate; mod command; mod entity; mod error; mod event; mod value_object;

#[proc_macro_derive(Entity, attributes(entity))]
pub fn derive_entity(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    entity::derive(i).into()
}
#[proc_macro_derive(ValueObject)]
pub fn derive_value_object(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    value_object::derive(i).into()
}
#[proc_macro_derive(Command, attributes(command))]
pub fn derive_command(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    command::derive(i).into()
}
#[proc_macro_derive(DomainEvent, attributes(event))]
pub fn derive_event(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    event::derive(i).into()
}
#[proc_macro_derive(Aggregate, attributes(aggregate))]
pub fn derive_aggregate(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    aggregate::derive(i).into()
}
#[proc_macro_derive(Error)]
pub fn derive_error(i: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let i = syn::parse_macro_input!(i as syn::DeriveInput);
    error::derive(i).into()
}
