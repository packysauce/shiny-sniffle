use darling::{FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DeriveInput};

mod helpers;

#[proc_macro_derive(Entity, attributes(entity))]
pub fn make_entity_macro(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let stuff = helpers::EntityDeriveInput::from_derive_input(&input).unwrap();
    let t = quote!(#stuff);
    t.into()
}

#[proc_macro_derive(Assoc, attributes(assoc))]
pub fn make_assoc_macro(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let stuff = helpers::AssocDeriveInput::from_derive_input(&input).unwrap();
    let t = quote!(#stuff);
    t.into()
}

#[derive(FromMeta)]
struct IdAttribute {
    pub id: u64,
}

#[proc_macro_attribute]
pub fn assoc(args: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(args as AttributeArgs);
    let item = parse_macro_input!(input as syn::ItemTrait);
    let args = match IdAttribute::from_list(&attrs) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let name = &item.ident;
    let id = args.id;

    let impl_def = quote! {
        impl #impl_generics ::wtf::AssocTypeID for dyn #name #ty_generics
        #where_clause
        {
            const TYPE_ID: u64 = #id;
        }

        #item
    };
    impl_def.into()
}
