use darling::{FromDeriveInput, ToTokens};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod assoc;
mod entity;
use entity::EntityDeriveInput;

use self::assoc::AssocDeriveInput;

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

#[proc_macro_derive(Assoc, attributes(assoc))]
pub fn make_assoc_macro(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let stuff = AssocDeriveInput::from_derive_input(&input).unwrap();
    stuff.to_token_stream().into()
}
