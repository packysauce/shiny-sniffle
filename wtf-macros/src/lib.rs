use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod assoc;
mod entity;
use entity::EntityDeriveInput;

#[proc_macro_derive(Entity, attributes(entity))]
pub fn make_entity_macro(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let stuff = EntityDeriveInput::from_derive_input(&input).unwrap();
    let t = quote!(#stuff);
    t.into()
}

#[proc_macro_attribute]
pub fn assoc(args: TokenStream, input: TokenStream) -> TokenStream {
    assoc::assoc(args, input)
}
