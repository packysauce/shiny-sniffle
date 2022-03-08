//! Online Data Graph
//! =================
//!
//! We store all our live data in one big global graph, made up of two types of
//! data — entities and associations.
//!
//! All entities have an ID, assigned from one 64-bit pool, along with a type
//! identifier (also 64 bits). All associations (assocs for short) have a pair
//! of IDs defining the endpoints and direction of their relation, a type
//! identifier (64 bits again), and a timestamp indicating when they were last
//! changed.
//!
//! Finally, both entities and assocs have storage for data — up to a megabyte
//! per entity or 255 bytes per assoc. By convention, that data is filled with
//! serialized Rust structs, using zstd compressed serde-postcard.
//!
//! The types, then, are more or less as follows:
//!
//! ```sql,ignore
//! ID:         u64
//! TypeID:     u64
//! LastChange: datetime
//! Entity:     (id, typeid, data: [u8; 255])
//! Assoc:      (id1, id2, typeid, last_change, data: [u8; 1M])
//! ```
//!
//! This system is inspired by Tao, Facebook's graph abstraction over shards of
//! `MySQL` databases.

use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};

#[cfg(feature = "sqlite")]
pub mod sqlite;

pub mod errors;
pub mod types;

pub use errors::TeaError;
pub use types::{AssocRangeAfter, AssocRangeLimit, AssocStorage, AssocType, EntityId, EntityType};

/// Result Alias
///
/// This is a convenience alias for [`std::result::Result`] default specialized
/// to `<T, TeaError>`. It's what's returned from most everything in `tea`.
pub type Result<T, E = TeaError> = std::result::Result<T, E>;

/// A Connection to The Entities and Assocs
///
/// The `TeaConnection` trait defines the operations available on the data store
/// for all our entities and assocs. Typically you're using an impl of this to
/// talk to the database, but for testing or little one-off programs with local
/// data only, you might consider using the tea-sqlite crate instead.
///
/// The interface is modeled after Tao's — entities are subject to CRUD ops,
/// and assocs can be created/updated/deleted by unique (ty, id1, id2) key, or
/// retrieved by one of three query interfaces:
///   * `assoc_get` — fetch the assocs matching (ty, id1) where id2 is in the
///     given set. This is useful for one-off gets as well as queries against a
///     known set of other ids.
///   * `assoc_range` — fetch all assocs matching (ty, id1). Note this interface
///     is paginated, so you may need to hit it repeatedly if you really want
///     _everything_ coming out of id1 of that type.
///   * `assoc_time_range` — fetch all assocs matching (ty, id1) updated within
///     the given time window. This is helpful if you're more interested in
///     recency than completeness, e.g. for activity or recent action lists.
pub trait TeaConnection {
    /// Initialize an empty database.
    ///
    /// NB. you typically only need to do this if you're setting something up
    ///     from scratch. Don't make a habit of running this every time you
    ///     fire up a connection.
    fn initialize(&mut self) -> Result<()>;

    /// Add a new entity of type `ty` with the provided associated `data`
    fn ent_add(&mut self, ty: EntityType, data: &[u8]) -> Result<EntityId>;
    /// Fetch the entity data from the given `id`
    fn ent_get(&mut self, id: EntityId) -> Result<(EntityType, Vec<u8>)>;
    /// Update the entity at the given `id`, replacing its data entirely
    fn ent_update(
        &mut self,
        id: EntityId,
        ty: EntityType,
        data: &[u8],
    ) -> Result<(EntityType, Vec<u8>)>;
    /// Delete the entity at the given `id`
    fn ent_delete(&mut self, id: EntityId) -> Result<(EntityType, Vec<u8>)>;

    /// Add or overwrite the association `(ty, id1, id2)` and its inverse, if
    /// an inverse type is defined for `ty`.
    fn assoc_add(&mut self, ty: AssocType, id1: EntityId, id2: EntityId, data: &[u8])
        -> Result<()>;
    /// Delete the assoc `(ty, id1, id2)` and its inverse, if it exists
    fn assoc_delete(&mut self, ty: AssocType, id1: EntityId, id2: EntityId)
        -> Result<AssocStorage>;
    /// Change the association `(ty, id1, id2)` to `(new_ty, id1, id2)`
    fn assoc_change_type(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        id2: EntityId,
        new_ty: AssocType,
    ) -> Result<AssocStorage>;
    /// Fetch all assocs of type `ty` originating at `id1` where `id2` is in the
    /// `id2_set` and, if specified, last updated in the range `[low, high]`
    fn assoc_get(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        id2_set: &[EntityId],
        high: Option<DateTime<Utc>>,
        low: Option<DateTime<Utc>>,
    ) -> Result<Vec<AssocStorage>>;
    /// Count the number of edges of type `ty` originating at `id1`.
    fn assoc_count(&mut self, ty: AssocType, id1: EntityId) -> Result<usize>;
    /// Retrieve assocs of type `ty` originating at `id1`.
    ///
    /// This interface is paginated — it returns up to `limit` assocs, beginning
    /// with the first entity ID greater than `after`.
    fn assoc_range(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        after: AssocRangeAfter,
        limit: AssocRangeLimit,
    ) -> Result<Vec<AssocStorage>>;
    /// Retrieve up to `limit` assocs of type `ty` originating at `id1`, where
    /// the last update time is in the range `[low, high]`.
    fn assoc_time_range(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        high: DateTime<Utc>,
        low: DateTime<Utc>,
        limit: AssocRangeLimit,
    ) -> Result<Vec<AssocStorage>>;
}

#[derive(Clone)]
/// A `TeaConnection` which can be shared across threads.
///
/// It automatically manages its lifespan and sharing using Arc/Mutex from the
/// standard library, so bear in mind that it can be a source of contention if
/// you're using just the one for your whole program.
pub struct SharedTeaConnection {
    /// The inner `TeaConnection` we are sharing
    conn: Arc<Mutex<dyn TeaConnection + Send>>,
}
impl SharedTeaConnection {
    /// Create a new `SharedTeaConnection` from an owned or static-borrowed
    /// `TeaConnection` instance.
    pub fn new(conn: impl TeaConnection + Send + 'static) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }
}
impl TeaConnection for SharedTeaConnection {
    fn initialize(&mut self) -> Result<()> {
        self.conn.lock()?.initialize()
    }

    fn ent_add(&mut self, ty: EntityType, data: &[u8]) -> Result<EntityId> {
        self.conn.lock()?.ent_add(ty, data)
    }

    fn ent_get(&mut self, id: EntityId) -> Result<(EntityType, Vec<u8>)> {
        self.conn.lock()?.ent_get(id)
    }

    fn ent_update(
        &mut self,
        id: EntityId,
        ty: EntityType,
        data: &[u8],
    ) -> Result<(EntityType, Vec<u8>)> {
        self.conn.lock()?.ent_update(id, ty, data)
    }

    fn ent_delete(&mut self, id: EntityId) -> Result<(EntityType, Vec<u8>)> {
        self.conn.lock()?.ent_delete(id)
    }

    fn assoc_add(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        id2: EntityId,
        data: &[u8],
    ) -> Result<()> {
        self.conn.lock()?.assoc_add(ty, id1, id2, data)
    }

    fn assoc_delete(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        id2: EntityId,
    ) -> Result<AssocStorage> {
        self.conn.lock()?.assoc_delete(ty, id1, id2)
    }

    fn assoc_change_type(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        id2: EntityId,
        new_ty: AssocType,
    ) -> Result<AssocStorage> {
        self.conn.lock()?.assoc_change_type(ty, id1, id2, new_ty)
    }

    fn assoc_get(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        id2_set: &[EntityId],
        high: Option<DateTime<Utc>>,
        low: Option<DateTime<Utc>>,
    ) -> Result<Vec<AssocStorage>> {
        self.conn.lock()?.assoc_get(ty, id1, id2_set, high, low)
    }

    fn assoc_count(&mut self, ty: AssocType, id1: EntityId) -> Result<usize> {
        self.conn.lock()?.assoc_count(ty, id1)
    }

    fn assoc_range(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        after: AssocRangeAfter,
        limit: AssocRangeLimit,
    ) -> Result<Vec<AssocStorage>> {
        self.conn.lock()?.assoc_range(ty, id1, after, limit)
    }

    fn assoc_time_range(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        high: DateTime<Utc>,
        low: DateTime<Utc>,
        limit: AssocRangeLimit,
    ) -> Result<Vec<AssocStorage>> {
        self.conn
            .lock()?
            .assoc_time_range(ty, id1, high, low, limit)
    }
}
