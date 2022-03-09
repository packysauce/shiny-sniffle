pub type TypeId = u64;
pub type EntityId = u64;
use std::ops::Deref;

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

/// A saved Entity
pub struct Ent<T: EntityTypeID>(T, Saved<EntityId>);

pub trait Entity {
    type ObjectType: EntityTypeID;
    fn type_id(&self) -> TypeId;
    fn obj_id(&self) -> EntityId;
}

impl<T: EntityTypeID> AsRef<Ent<T>> for Ent<T> {
    fn as_ref(&self) -> &Ent<T> {
        self
    }
}

impl<T: EntityTypeID> EntityTypeID for &T {
    const TYPE_ID: TypeId = T::TYPE_ID;
}

impl<T: EntityTypeID> EntityTypeID for Ent<T> {
    const TYPE_ID: TypeId = T::TYPE_ID;
}

impl<T: EntityTypeID> Entity for Ent<T> {
    type ObjectType = T;
    fn type_id(&self) -> TypeId {
        T::TYPE_ID
    }

    fn obj_id(&self) -> EntityId {
        self.id()
    }
}

impl<'e, T: EntityTypeID> Ent<T> {
    pub fn id(&self) -> EntityId {
        self.1 .0
    }
}

impl<'e, T: EntityTypeID> Deref for Ent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait Save
where
    Self: Sized + EntityTypeID,
{
    fn save<DB: TeaConnection>(self, db: &mut DB) -> SaveResult<Ent<Self>>;
}

impl<T: EntityTypeID + Serialize> Save for T {
    fn save<DB: TeaConnection>(self, db: &mut DB) -> SaveResult<Ent<T>> {
        let data = serde_json::to_vec(&self).map_err(SaveError::from)?;
        let ty = EntityType::from_u64(T::TYPE_ID).unwrap();
        let id = db.ent_add(ty, &data).map_err(SaveError::from)?;
        Ok(Ent(self, Saved::new(id.as_u64())))
    }
}
