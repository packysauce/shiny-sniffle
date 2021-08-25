use darling::{FromDeriveInput, ToTokens};
use heck::SnekCase;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(FromDeriveInput)]
#[darling(attributes(assoc))]
pub struct Assoc {
    ident: syn::Ident,
    id: u64,
}

impl ToTokens for Assoc {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.ident;
        let id = self.id;
        let assoc_name = syn::Ident::new(&format!("Assoc{}", &self.ident), self.ident.span());
        let fn_name = syn::Ident::new(&name.to_string().to_snek_case(), self.ident.span());
        let new_stuff = quote! {
            impl AsRef<::wtf::RawAssoc> for #name<::wtf::Saved<::wtf::RawAssoc>> {
                fn as_ref(&self) -> &RawAssoc {
                    self.as_ref()
                }
            }

            pub trait #assoc_name {
                fn #fn_name<Ent: ::wtf::Entity>(&self, what: &Ent) -> #name<::wtf::Dirty>;
            }

            impl ::wtf::Save<()> for #name<::wtf::Dirty> {
                type Saved = #name<::wtf::Saved<()>>;

                fn save(self, db: &mut dyn ::wtf::TeaConnection) -> Result<Self::Saved, ::wtf::SaveError<Self>> {
                    let Self(assoc, _) = self;
                    let (from, to, ty) = assoc.split();
                    if let Err(e) = db.assoc_add(ty, from.id(), to.id(), &[]) {
                        return Err(::wtf::SaveError::Tea(self, e));
                    }
                    Ok(#name(
                        ::wtf::RawAssoc::new(from, to, ty.as_u64()),
                        ::wtf::Saved::new(()),
                    ))
                }
            }

            impl<T> #assoc_name for T where T: ::wtf::Entity {
                fn #fn_name<Ent: ::wtf::Entity>(&self, what: &Ent) -> #name<::wtf::Dirty> {
                    #name(
                        ::wtf::RawAssoc::new(
                            Self::entity(self),
                            Ent::entity(what),
                            #id,
                        ),
                        ::wtf::Dirty,
                    )
                }
            }
        };
        tokens.extend(new_stuff)
    }
}

#[derive(FromDeriveInput)]
#[darling(attributes(entity))]
pub struct Entity {
    ident: syn::Ident,
    // data: Data<(), syn::Field>,
    id: u64,
}

impl ToTokens for Entity {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.ident;
        let id = self.id;
        let ent_name = syn::Ident::new(&format!("Ent{}", &self.ident), self.ident.span());
        let new_stuff = quote! {
            #[derive(Debug)]
            pub struct #ent_name<S: ::wtf::PersistedState> {
                ent: #name,
                db_state: S,
            }

            impl ::wtf::Save<::wtf::RawEntity> for #ent_name<::wtf::Dirty> {
                type Saved = #ent_name<::wtf::Saved<::wtf::RawEntity>>;
                fn save(self, db: &mut dyn ::tea::TeaConnection) -> tea::Result<Self::Saved, ::wtf::SaveError<Self>> {
                    #ent_name::from(self).save(db)
                }
            }

            impl From<#name> for #ent_name<::wtf::Dirty> {
                fn from(t: #name) -> #ent_name<::wtf::Dirty> {
                    Self {
                        ent: t,
                        db_state: ::wtf::Dirty
                    }
                }
            }

            impl ::wtf::Save<::wtf::RawEntity> for #name {
                type Saved = #ent_name<::wtf::Saved<::wtf::RawEntity>>;

                fn save(self, db: &mut dyn ::tea::TeaConnection) -> ::tea::Result<Self::Saved, ::wtf::SaveError<Self>> {
                    let data = match serde_json::to_vec(&self) {
                        Ok(d) => d,
                        Err(e) => return Err(::wtf::SaveError::Serde(self, e)),
                    };
                    let ty = Self::entity_type();
                    let id = match db.ent_add(ty, &data) {
                        Ok(d) => d,
                        Err(e) => return Err(::wtf::SaveError::Tea(self, e)),
                    };
                    let raw = ::wtf::RawEntity::new(id, ty);
                    Ok(#ent_name {
                        ent: self,
                        db_state: ::wtf::Saved::new(raw),
                    })
                }
            }

            impl ::wtf::Entity for #ent_name<::wtf::Saved<::wtf::RawEntity>> {
                fn ty(&self) -> ::tea::EntityType {
                    ::tea::EntityType::from_u64(#id).expect("bad id")
                }
                fn id(&self) -> ::tea::EntityId {
                    self.db_state.as_ref().id()
                }
                fn entity(&self) -> ::wtf::RawEntity {
                    *self.db_state.as_ref()
                }
            }

            #[automatically_derived]
            impl ::wtf::ToEntity for #name {
                type Entity = #ent_name<::wtf::Dirty>;

                fn entity_type() -> wtf::EntityType {
                    wtf::EntityType::from_u64(#id).expect("bad id")
                }

                fn into_entity(self) -> Self::Entity {
                    #ent_name {
                        ent: self,
                        db_state: ::wtf::Dirty,
                    }
                }
            }
        };
        tokens.extend(new_stuff)
    }
}
