use darling::{FromDeriveInput, ToTokens};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(FromDeriveInput)]
#[darling(attributes(entity))]
pub struct EntityDeriveInput {
    ident: syn::Ident,
    id: u64,
}

impl ToTokens for EntityDeriveInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.ident;
        let id = self.id;
        let new_stuff = quote! {
            impl ::wtf::EntityTypeID for #name {
                const TYPE_ID: u64 = #id;
            }
        };
        tokens.extend(new_stuff)
    }
}
