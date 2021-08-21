use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, Data, DeriveInput, LitInt, Meta, NestedMeta};

#[derive(darling::FromField)]
#[darling(attributes(assoc), forward_attrs)]
struct AssocField {
    ident: Option<syn::Ident>,
    vis: syn::Visibility,
    ty: syn::Type,
    assoc: bool,
}

#[derive(darling::FromDeriveInput)]
#[darling(attributes(assoc), forward_attrs)]
struct AssocOptions {
    ident: syn::Ident,
    id: u64,
}

#[proc_macro_derive(Assoc, attributes(assoc))]
pub fn derive_assoc(tokens: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(tokens as DeriveInput);
    let options = AssocOptions::from_derive_input(&input).unwrap();
    let name = &options.ident;
    let id = options.id;
    let tokens = quote! {
        #[automatically_derived]
        impl #name  {
            fn type_id() -> ::tea::AssocType {
                ::tea::AssocType::from_nonzero_u64(
                    unsafe { ::std::num::NonZeroU64::new_unchecked(#id) }
                )
            }
        }
    };

    tokens.into()
}

#[derive(darling::FromDeriveInput)]
#[darling(attributes(entity), forward_attrs)]
struct EntityOptions {
    ident: syn::Ident,
    id: u64,
}

#[proc_macro_derive(Entity, attributes(entity))]
pub fn derive_entity(tokens: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(tokens as DeriveInput);
    let options = EntityOptions::from_derive_input(&input).unwrap();
    let name = &options.ident;
    let id = options.id;
    let tokens = quote! {
        #[automatically_derived]
        impl Entity for #name  {
            const TYPE_ID: EntityType = EntityType::from_nonzero_u64(
                unsafe { ::std::num::NonZeroU64::new_unchecked(#id) }
            );
        }
    };

    tokens.into()
}
