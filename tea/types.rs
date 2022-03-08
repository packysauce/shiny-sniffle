//! data: An undiscovered masterpiece
//! ============================================================================
//!
//! TODO: Descriptive crate-level documentation for this program.

use std::convert::TryFrom;
use std::num::NonZeroU64;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::TeaError;


// /////////////////////////////////////////////////////////////////////////////
// NEWTYPES ////////////////////////////////////////////////////////////////////
// /////////////////////////////////////////////////////////////////////////////

/// Entity Identifier
///
/// All Tea entities are uniquely identified by a 64-bit nonzero integer ID.
/// This number is wrapped in a newtype to prevent accidental API mistakes —
/// stuff like accidentally using an ent ID as a type ID or vice versa.
#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct EntityId(NonZeroU64);
impl EntityId {
    /// Construct an entity ID from a non-zero u64.
    pub const fn from_u64(ent_id_u64: u64) -> crate::Result<EntityId> {
        if let Some(ent_id) = NonZeroU64::new(ent_id_u64) {
            Ok(EntityId(ent_id))
        } else {
            Err(TeaError::ZeroIsNotAValidID {})
        }
    }
    /// Infallibly construct an entity ID from a `NonZeroU64`
    pub const fn from_nonzero_u64(ent_id: NonZeroU64) -> EntityId {
        EntityId(ent_id)
    }
    /// Convert this entity ID into a u64. This will never return zero.
    pub fn as_u64(&self) -> u64 {
        self.0.into()
    }
}
impl TryFrom<u64> for EntityId {
    type Error = TeaError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::from_u64(value)
    }
}
impl From<EntityId> for NonZeroU64 {
    fn from(EntityId(ty): EntityId) -> Self {
        ty
    }
}
impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ent({})", self.0)
    }
}

/// Entity Type Identifier
///
/// `EntityType` is a newtype around a nonzero u64 which is used to
/// discriminate, as you might guess, types of entities. Every ent has one of
/// these, and they're meant to be unique.
#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct EntityType(NonZeroU64);
impl EntityType {
    /// Construct an entity type from a non-zero u64.
    pub const fn from_u64(ty_u64: u64) -> crate::Result<EntityType> {
        if let Some(ty) = NonZeroU64::new(ty_u64) {
            Ok(EntityType(ty))
        } else {
            Err(TeaError::ZeroIsNotAValidType {})
        }
    }
    /// Infallibly construct an entity type from a `NonZeroU64`
    pub const fn from_nonzero_u64(ty: NonZeroU64) -> EntityType {
        EntityType(ty)
    }
    /// Convert this entity type into a u64. This will never return zero.
    pub fn as_u64(&self) -> u64 {
        self.0.into()
    }
}
impl TryFrom<u64> for EntityType {
    type Error = TeaError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::from_u64(value)
    }
}
impl From<EntityType> for NonZeroU64 {
    fn from(EntityType(ty): EntityType) -> Self {
        ty
    }
}
impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntType({})", self.0)
    }
}

/// Association Type Identifier
///
/// `AssocType` is a newtype around a nonzero u64 which is used to discriminate,
/// as you might guess, types of associations. Every assoc has one of these,
/// and they're meant to be unique.
#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct AssocType(NonZeroU64);
impl AssocType {
    /// Construct an association type from a non-zero u64.
    pub const fn from_u64(ty_u64: u64) -> crate::Result<AssocType> {
        if let Some(ty) = NonZeroU64::new(ty_u64) {
            Ok(AssocType(ty))
        } else {
            Err(TeaError::ZeroIsNotAValidID {})
        }
    }
    /// Infallibly construct an association type from a `NonZeroU64`
    pub const fn from_nonzero_u64(ty: NonZeroU64) -> AssocType {
        AssocType(ty)
    }
    /// Infallibly construct an association type from a `NonZeroU64`
    ///
    /// # Safety
    /// The passed value must be a valid u64 greater than zero
    pub const unsafe fn from_u64_unchecked(ty: u64) -> AssocType {
        AssocType(NonZeroU64::new_unchecked(ty))
    }
    /// Convert this association type into a u64. This will never return zero.
    pub fn as_u64(&self) -> u64 {
        self.0.into()
    }
}
impl TryFrom<u64> for AssocType {
    type Error = TeaError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::from_u64(value)
    }
}
impl From<AssocType> for NonZeroU64 {
    fn from(AssocType(ty): AssocType) -> Self {
        ty
    }
}
impl std::fmt::Display for AssocType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AssocType({})", self.0)
    }
}

/// Association Storage Type
///
/// An `AssocStorage` comprises all the information necessary to interact with
/// an assoc. Specifically, it includes the uniquely identifying 3-tuple of
/// `(ty, id1, id2)` — the type of the assoc and its starting/ending ents —
/// along with a timestamp for the last time the assoc was modified, and the
/// associated data.
///
/// Note that while associated data is represented as an arbitrary-size vector,
/// assocs typically have a maximum limit on data at the storage layer. If
/// you're bumping up against that limit, you might want to consider breaking
/// apart some of the edge information and either storing it on an ent or
/// another assoc. If that's still not enough storage for you (you glutton), you
/// might also consider storing a reference to another external storage system
/// here, although bear in mind you'll have to give some thought to consistency
/// between this system and that one.
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct AssocStorage {
    /// The type of association
    pub ty: AssocType,
    /// The originating entity for this association
    pub id1: EntityId,
    /// The terminating entity for this association
    pub id2: EntityId,
    /// The date and time of the last modification of this association
    pub last_change: DateTime<Utc>,
    /// Arbitrary data attached to this association.
    pub data: Vec<u8>,
}


/// Association Range Query: After ID
///
/// `AssocRangeAfter` is used in association range queries to control result
/// pagination — you can get the first page of results by sending `First`, then
/// subsequent pages by issuing another request with the last entity ID of the
/// previous page in `ID(id)`.
#[derive(Debug, Clone, Copy, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum AssocRangeAfter {
    /// Fetch the first page of results
    First,
    /// Fetch a page of results starting with the next entity after this one
    ID(EntityId),
}


/// Association Range Query: Limit per Page
///
/// Association queries are paginated by default. This parameter allows you to
/// specify how many entries you'd like to fetch per page.
///
/// Note that storage implementations set their own maximums for how high you
/// can set this limit, so if you're using a specific number of results here you
/// may want to consult the documentation (or sources) for your storage layer to
/// make sure you're not overdoing it.
#[derive(Debug, Clone, Copy, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum AssocRangeLimit {
    /// Use the default pagination limit.
    ///
    /// This is generally in the low hundreds of records per page.
    Default,
    /// Fetch a specific number of results per page.
    ///
    /// If this is larger than the underlying storage's maximum results per
    /// page, your query is likely to return an error.
    Limit(usize),
    /// Fetch as many results per page as the underlying storage will allow.
    ///
    /// If you're looking to make as few round trips as possible to
    /// exhaustively list records you might want to use this instead of a
    /// `Limit` number, since this way your call site won't drift when the
    /// underlying storage's maximum is changed later.
    Maximum,
}
