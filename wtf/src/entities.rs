pub type TypeId = u64;
pub type EntityId = u64;
use serde::Serialize;
use tea::{EntityType, TeaConnection};

use crate::{
    state::{SaveResult, Saved},
    SaveError,
};

pub use super::assocs::AssocType;

pub trait EntityTypeID {
    const TYPE_ID: TypeId;
}

pub struct Ent<T: EntityTypeID>(T, Saved<EntityId>);

impl<T: EntityTypeID> Ent<T> {
    pub fn id(&self) -> EntityId {
        self.1 .0
    }
}

pub trait Save
where
    Self: Sized + EntityTypeID,
{
    fn save<DB: TeaConnection>(self, db: &mut DB) -> SaveResult<Ent<Self>>;
}

impl<T: EntityTypeID + Serialize> Save for T {
    fn save<DB: TeaConnection>(self, db: &mut DB) -> SaveResult<Ent<Self>> {
        let data = serde_json::to_vec(&self).map_err(SaveError::from)?;
        let ty = EntityType::from_u64(T::TYPE_ID).unwrap();
        let id = db.ent_add(ty, &data).map_err(SaveError::from)?;
        Ok(Ent(self, Saved(id.as_u64())))
    }
}
