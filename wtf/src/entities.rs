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

/// An entity, at the atomic level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawEntity {
    // alright fine, quarks here
    /// the entity's ID - this is globally unique
    id: EntityId, // i have 100% thought of this as subnet
    ty: TypeId, // and host and you bet your fuckin biscuits i will try it out
}

impl RawEntity {
    pub fn new(id: EntityId, ty: TypeId) -> Self {
        Self { id, ty }
    }

    /// Get a reference to the raw entity's id.
    pub fn id(&self) -> EntityId {
        self.id
    }

    /// Get a reference to the raw entity's ty.
    pub fn ty(&self) -> TypeId {
        self.ty
    }
}

pub trait Save
where
    Self: Sized + EntityTypeID,
{
    fn save(self, db: &mut dyn TeaConnection) -> SaveResult<Ent<Self>>;
}

impl<T: EntityTypeID + Serialize> Save for T {
    fn save(self, db: &mut dyn TeaConnection) -> SaveResult<Ent<Self>> {
        let data = serde_json::to_vec(&self).map_err(SaveError::from)?;
        let ty = EntityType::from_u64(T::TYPE_ID).unwrap();
        let id = db.ent_add(ty, &data).map_err(SaveError::from)?;
        Ok(Ent(self, Saved(id.as_u64())))
    }
}
