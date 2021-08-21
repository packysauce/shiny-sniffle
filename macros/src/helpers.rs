use darling::{ast::Data, FromDeriveInput, FromMeta, ToTokens};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

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
            pub struct #ent_name<S: PersistedState> {
                ent: #name,
                db_state: S,
            }

            impl Save<RawEntity> for #name {
                type Saved = #ent_name<Saved<RawEntity>>;

                fn save(self, db: &mut dyn tea::TeaConnection) -> tea::Result<Self::Saved, SaveError<Self>> {
                    let data = match serde_json::to_vec(&self) {
                        Ok(d) => d,
                        Err(e) => return Err(SaveError::Serde(self, e)),
                    };
                    let ty = Self::entity_type();
                    let id = match db.ent_add(ty, &data) {
                        Ok(d) => d,
                        Err(e) => return Err(SaveError::Tea(self, e)),
                    };
                    let raw = RawEntity { id, ty };
                    Ok(#ent_name {
                        ent: self,
                        db_state: Saved(raw),
                    })
                }
            }

            impl Entity for #ent_name<Saved<RawEntity>> {
                fn ty(&self) -> tea::EntityType {
                    EntityType::from_u64(#id).expect("bad id")
                }
                fn id(&self) -> tea::EntityId {
                    self.db_state.0.id
                }
                fn to_entity(&self) -> RawEntity {
                    self.db_state.0
                }
            }

            #[automatically_derived]
            impl ToEntity for #name {
                type Entity = #ent_name<Dirty>;

                fn entity_type() -> EntityType {
                    EntityType::from_u64(#id).expect("bad id")
                }

                fn ent(self) -> Self::Entity {
                    #ent_name {
                        ent: self,
                        db_state: Dirty,
                    }
                }
            }
        };
        tokens.extend(new_stuff)
    }
}
