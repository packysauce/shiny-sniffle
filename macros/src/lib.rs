use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod helpers;

#[proc_macro_derive(Entity, attributes(entity))]
pub fn make_macro(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let stuff = helpers::Entity::from_derive_input(&input).unwrap();
    let t = quote!(#stuff);
    t.into()
}
