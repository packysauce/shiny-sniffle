//#![allow(unused)]

mod demo;

use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Serialize};

// there's always a lighthouse...
pub trait Database {
    type Error: std::error::Error;
}

/// A yet-to-be-committed object.
pub struct Dirty<T, Id = T>(T, PhantomData<Id>);
impl<T, U> Dirty<T, U> {
    pub fn new(t: T) -> Dirty<T, U> {
        Self(t, PhantomData::<U>)
    }

    pub fn get(&self) -> &T {
        &self.0
    }
}

/// An object that has been committed, and thus has an ID
pub struct Saved<T, Id> {
    t: T,
    id: Id,
}

impl<T, Id> std::ops::Deref for Saved<T, Id> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.t
    }
}

/// An entity, at the atomic level.
#[derive(Default, Clone, Copy, PartialEq)]
pub struct RawEntity {
    // alright fine, quarks here
    /// the entity's ID - this is globally unique
    id: u64, // i have 100% thought of this as subnet
    ty: u64, // and host and you bet your fuckin biscuits i will try it out
}

/// An Entity consts of a grand total of 128 bits of data.
/// 64 of which is a type identifier, and the remainder a global ID
pub trait Entity {
    fn to_entity(&self) -> RawEntity;
}
// .. but only if they can
impl<T> Entity for Saved<T, RawEntity> {
    fn to_entity(&self) -> RawEntity {
        self.id
    }
}

// Assocations are merely 2 objects and the nature of assocation
/// Storage of an assocation. If you think of an assocation as an arrow,
/// then the base of the arrow is the "from" entity, and the "to" entity
/// is being pointed at by the arrow.
#[derive(Clone, Copy)]
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

pub trait EntityStorage {
    type Error: std::error::Error;
    const TYPE_ID: u64;
}

pub trait AssocStorage: Serialize + DeserializeOwned {
    type Error: std::error::Error;
    const TYPE_ID: u64;
}

impl<T> Dirty<T, RawEntity> /* where T: std::fmt::Debug */ {
    fn save<DB: Database>(&self, db: &DB) -> Result<Saved<T, RawEntity>, DB::Error> {
        todo!(); // fuck you ive done enough!
                 //let out = self.0.fmt(db)?;
                 //Ok(Saved { t: self.0, id: out })
    }
}

impl<T> Dirty<T, RawAssoc> /* where T: std::fmt::Debug */ {
    fn save<DB: Database>(&self, db: &DB) -> Result<Saved<T, RawAssoc>, DB::Error> {
        todo!(); // fuck you ive done enough!
                 //let out = self.0.fmt(db)?;
                 //Ok(Saved { t: self.0, id: out })
    }
}
