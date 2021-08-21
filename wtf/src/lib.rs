//#![allow(unused)]

mod demo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dirty;

#[derive(Debug)]
pub struct Saved<Id: std::fmt::Debug>(Id);

pub trait PersistedState {}
impl PersistedState for Dirty {}
impl<Id: std::fmt::Debug> PersistedState for Saved<Id> {}

pub trait Storage {
    type Id;
    type Error;
}

/// An entity, at the atomic level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawEntity {
    // alright fine, quarks here
    /// the entity's ID - this is globally unique
    id: tea::EntityId, // i have 100% thought of this as subnet
    ty: tea::EntityType, // and host and you bet your fuckin biscuits i will try it out
}

pub trait ToEntity {
    type Entity;

    fn entity_type() -> tea::EntityType;
    fn ent(self) -> Self::Entity;
}

/// An Entity consts of a grand total of 128 bits of data.
/// 64 of which is a type identifier, and the remainder a global ID
pub trait Entity {
    fn ty(&self) -> tea::EntityType;
    fn id(&self) -> tea::EntityId;
    fn to_entity(&self) -> RawEntity;
}
// .. but only if they can
// impl<T> Entity for Saved<T, RawEntity> {
//     fn to_entity(&self) -> RawEntity {
//         self.id
//     }
// }

// Assocations are merely 2 objects and the nature of assocation
/// Storage of an assocation. If you think of an assocation as an arrow,
/// then the base of the arrow is the "from" entity, and the "to" entity
/// is being pointed at by the arrow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawAssoc {
    from: RawEntity,
    to: RawEntity,
    ty: u64,
}
pub trait Assoc {
    fn obj1(&self) -> RawEntity;
    fn obj2(&self) -> RawEntity;
    fn to_assoc(&self) -> RawAssoc;
}

impl Assoc for RawAssoc {
    fn obj1(&self) -> RawEntity {
        self.from
    }

    fn obj2(&self) -> RawEntity {
        self.to
    }

    fn to_assoc(&self) -> RawAssoc {
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

    fn to_assoc(&self) -> RawAssoc {
        self.as_ref().to_assoc()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SaveError<T: std::fmt::Debug> {
    #[error("Serialization failure: {1}")]
    Serde(T, #[source] serde_json::Error),
    #[error("Database failure: {1}")]
    Tea(T, #[source] tea::TeaError),
}

pub type SaveResult<T> = std::result::Result<T, SaveError<T>>;

pub trait Save<Id>: Sized + std::fmt::Debug {
    type Saved: Sized + std::fmt::Debug;

    fn save(self, db: &mut dyn tea::TeaConnection) -> Result<Self::Saved, SaveError<Self>>;
}
