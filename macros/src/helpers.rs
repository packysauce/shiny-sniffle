use darling::{FromDeriveInput, ToTokens};
use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;
#[derive(FromDeriveInput)]
#[darling(attributes(assoc), forward_attrs(allow, doc, cfg))]
pub struct AssocDeriveInput {
    ident: syn::Ident,
    id: u64,
}

impl ToTokens for AssocDeriveInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
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

#[derive(FromDeriveInput)]
#[darling(attributes(entity))]
pub struct EntityDeriveInput {
    ident: syn::Ident,
    // data: Data<(), syn::Field>,
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

            impl #name {
                fn save(self, &mut ::wtf::TeaConnection) -> ::wtf::SaveResult<
            }
        };
        tokens.extend(new_stuff)
    }
}
