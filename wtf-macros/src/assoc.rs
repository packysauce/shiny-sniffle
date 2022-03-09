use darling::{FromDeriveInput, FromMeta, ToTokens};
use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, AttributeArgs};

#[derive(FromMeta)]
struct IdAttribute {
    pub(crate) id: u64,
}

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

#[derive(FromDeriveInput)]
#[darling(attributes(assoc), forward_attrs(allow, doc, cfg))]
pub struct AssocDeriveInput {
    ident: syn::Ident,
    id: u64,
    forward: String,
    reverse: String,
}

impl ToTokens for AssocDeriveInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            ident,
            id,
            forward,
            reverse,
        } = self;
        let assoc_name = syn::Ident::new(&format!("{}Assoc", &self.ident), ident.span());
        let fwd_trait = syn::Ident::new(&forward.to_upper_camel_case(), forward.span());
        let rev_trait = syn::Ident::new(&reverse.to_upper_camel_case(), reverse.span());
        let fwd_fn_name = syn::Ident::new(&forward.to_snake_case(), forward.span());
        let rev_fn_name = syn::Ident::new(&reverse.to_string().to_snake_case(), reverse.span());

        let new_stuff = quote! {
            #[automatically_derived]
            pub type #assoc_name<'f, 't, Id1, Id2> = ::wtf::assocs::Assoc<'f, 't, Id1, #ident, Id2>;

            #[automatically_derived]
            impl ::wtf::assocs::AssocTypeID for #ident {
                const TYPE_ID: ::wtf::assocs::AssocType = #id;
            }

            #[automatically_derived]
            pub trait #fwd_trait<'a, Id1, Id2>: EntityTypeID + Sized
            where
                Id1: EntityTypeID,
                Id2: EntityTypeID,
            {
                fn #fwd_fn_name(&'a self, other: &'a Ent<Id2>) -> #assoc_name<'a, '_, Id1, Id2>;
            }

            impl<'a, Id1, Id2> #fwd_trait<'a, Id1, Id2> for Ent<Id1>
            where
                Id1: EntityTypeID + 'a,
                Id2: EntityTypeID + 'a,
            {
                fn #fwd_fn_name(&'a self, other: &'a Ent<Id2>) -> #assoc_name<'a, '_, Id1, Id2> {
                    Assoc::new(self, other)
                }
            }

            pub trait #rev_trait<'a, Id1, Id2>: EntityTypeID + Sized
            where
                Id1: EntityTypeID,
                Id2: EntityTypeID,
            {
                fn #rev_fn_name(&'a self, other: &'a Ent<Id2>) -> #assoc_name<'a, '_, Id1, Id2>;
            }

            impl<'a, Id1, Id2> #rev_trait<'a, Id1, Id2> for Ent<Id1>
            where
                Id1: EntityTypeID + 'a,
                Id2: EntityTypeID + 'a,
            {
                fn #rev_fn_name(&'a self, other: &'a Ent<Id2>) -> #assoc_name<'a, '_, Id1, Id2> {
                    Assoc::new(self, other)
                }
            }

        };
        tokens.extend(new_stuff)
    }
}
