use darling::{FromDeriveInput, FromMeta, ToTokens};
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs};

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
}

impl ToTokens for AssocDeriveInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.ident;
        let id = self.id;
        let assoc_name = syn::Ident::new(&format!("{}Assoc", &self.ident), self.ident.span());
        let fn_name = syn::Ident::new(&name.to_string().to_snake_case(), self.ident.span());
        let new_stuff = quote! {
            #[automatically_derived]
            impl ::wtf::assocs::AssocTypeID for #name {
                const TYPE_ID: u64 = #id;
            }

            #[automatically_derived]
            pub trait #assoc_name<F, T> {
                fn #fn_name(&self, what: &::wtf::Ent<T>) -> ::wtf::Assoc<#assoc_name, F, T, ::wtf::Dirty>
            }

            #[automatically_derived]
            impl<F, T> #assoc_name<F, T> for ::wtf::Ent<F>
            {
                fn #fn_name(&self, what: &::wtf::Ent<T>)
                -> ::wtf::Assoc<#name, F, T, ::wtf::Dirty>
                {
                    ::wtf::Assoc::new(
                        self.0,
                        what.0,
                    )
                }
            }
        };
        tokens.extend(new_stuff)
    }
}
