//#![allow(unused)]

mod tea_reexports {
    pub use tea::{AssocType, EntityId, EntityType, TeaError, TeaConnection};
}

pub use crate::tea_reexports::*;

/// Implementation of `PersistedState` indicating the data is unsaved
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dirty;

/// Implementation of `PersistedState` indicating the data is commited.
#[derive(Debug)]
pub struct Saved<Id: std::fmt::Debug>(Id);

impl<Id: std::fmt::Debug> Saved<Id> {
    pub fn new(id: Id) -> Self {
        Self(id)
    }
}

/// Marker trait for Ent typestates
pub trait PersistedState {}
impl PersistedState for Dirty {}
impl<Id: std::fmt::Debug> PersistedState for Saved<Id> {}

/// An entity, at the atomic level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawEntity {
    // alright fine, quarks here
    /// the entity's ID - this is globally unique
    id: EntityId, // i have 100% thought of this as subnet
    ty: EntityType, // and host and you bet your fuckin biscuits i will try it out
}

impl RawEntity {
    pub fn new(id: EntityId, ty: EntityType) -> Self { Self { id, ty } }

    /// Get a reference to the raw entity's id.
    pub fn id(&self) -> EntityId {
        self.id
    }

    /// Get a reference to the raw entity's ty.
    pub fn ty(&self) -> EntityType {
        self.ty
    }
}

/// If you have a well known type ID, and are Serialize + Deserialize, you can become an Ent!
pub trait ToEntity {
    type Entity;

    fn entity_type() -> EntityType;
    fn into_entity(self) -> Self::Entity;
}

/// An Entity consts of a grand total of 128 bits of data.
/// 64 of which is a type identifier, and the remainder a global ID
pub trait Entity {
    fn ty(&self) -> EntityType;
    fn id(&self) -> EntityId;
    fn entity(&self) -> RawEntity;
}
/// Storage of an assocation. If you think of an assocation as an arrow,
/// then the base of the arrow is the "from" entity, and the "to" entity
/// is being pointed at by the arrow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawAssoc {
    // In TAO parlance, id1
    pub from: RawEntity,
    // In TAO parlance, id2
    pub to: RawEntity,
    // In TAO parlance, atype
    ty: AssocType,
}

impl RawAssoc {
    pub fn new(from: RawEntity, to: RawEntity, ty: u64) -> Self { Self { from, to, ty: AssocType::from_u64(ty).unwrap() } }

    pub fn split(&self) -> (RawEntity, RawEntity, AssocType) {
        (self.from, self.to, self.ty)
    }
}

/// If you can tell me what you are and what 2 entities you describe, you can be an Assoc!
pub trait Assoc {
    fn obj1(&self) -> RawEntity;
    fn obj2(&self) -> RawEntity;
    fn assoc(&self) -> RawAssoc;
}

impl Assoc for RawAssoc {
    fn obj1(&self) -> RawEntity {
        self.from
    }

    fn obj2(&self) -> RawEntity {
        self.to
    }

    fn assoc(&self) -> RawAssoc {
        *self
    }
}

impl<T> Assoc for T
where
    T: AsRef<RawAssoc>,
{
    fn obj1(&self) -> RawEntity {
        self.as_ref().obj1()
    }

    fn obj2(&self) -> RawEntity {
        self.as_ref().obj2()
    }

    fn assoc(&self) -> RawAssoc {
        self.as_ref().assoc()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SaveError<T: std::fmt::Debug> {
    #[error("Serialization failure: {1}")]
    Serde(T, #[source] serde_json::Error),
    #[error("Database failure: {1}")]
    Tea(T, #[source] TeaError),
}

pub type SaveResult<T> = std::result::Result<T, SaveError<T>>;

/// Save is how the object -> database serialization goes
pub trait Save<Id>: Sized + std::fmt::Debug {
    type Saved: Sized + std::fmt::Debug;

    fn save(self, db: &mut dyn TeaConnection) -> Result<Self::Saved, SaveError<Self>>;
}

impl<Id: std::fmt::Debug> AsRef<Id> for Saved<Id> {
    fn as_ref(&self) -> &Id {
        &self.0
    }
}