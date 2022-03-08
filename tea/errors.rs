//! Tea Errors
//! ==========
//!
//! Tea can produce a limited set of errors, given it's a pretty simple system.
//!
//! The types in this module enumerate them and provide conversion routines for
//! getting between them and standard library / common dependency equivalents.
//! In general, if you're trying to `?` out of a function and the compiler says
//! no, you probably just need to add a variant to [`TeaError`] below, along
//! with an impl of [`std::convert::From`] for whatever type you're trying
//! to use.

use std::sync::PoisonError;

use thiserror::Error;

use crate::{AssocType, EntityId};

/// Errors for Tea
///
/// This enumeration holds all the ways Tea can fail. In general, these fall in
/// two categories: data model violations and persistence issues. Data model
/// violations are things like trying to fetch an entity that doesn't exist,
/// asking for too many assocs in one page, or somehow modifying the wrong
/// number of records in the database for an update. Persistence issues are
/// problems with the layer below `tea` — sqlite or postgres — things like
/// connection timeouts, missing database files, or failed table migrations.
///
/// As a rule, we expect each variant to be documented (even though rustdoc
/// doesn't handle that properly yet), and a descriptive Display string.
#[derive(Error, Debug)]
pub enum TeaError {
    /// An entity operation failed because the corresponding entity could not
    /// be found in the database
    #[error("couldn't find entity {0}")]
    EntNotFound(EntityId),
    /// An entity operation failed because the corresponding entity could not
    /// be found in the database
    #[error("entity {0} already exists")]
    EntAlreadyExists(EntityId),
    /// An entity operation failed because the corresponding assoc could not
    /// be found in the database
    #[error("couldn't find assoc ({ty}: {id1}->{id2})")]
    AssocNotFound {
        /// The type of assoc we tried to find
        ty: AssocType,
        /// The originating ID of the missing assoc
        id1: EntityId,
        /// The ending ID of the missing assoc
        id2: EntityId,
    },
    /// An entity operation failed because the corresponding entity could not
    /// be found in the database
    #[error("assoc ({ty}:{id1}->{id2}) already exists")]
    AssocAlreadyExists {
        /// The type of assoc we tried to create
        ty: AssocType,
        /// The originating ID
        id1: EntityId,
        /// The destination ID
        id2: EntityId,
    },
    /// We tried to update something in the database, but the number of rows we
    /// modified was wrong
    #[error(
        "CRITICAL DATA MODEL ERROR: we modified {modified} rows updating \
         assoc ({ty}:{id1}->{id2}) but we were expecting to modify {expected}"
    )]
    AssocUpdateModifiedTooManyRows {
        /// Type of assoc
        ty: AssocType,
        /// Originating ID
        id1: EntityId,
        /// End ID
        id2: EntityId,
        /// Number of DB rows actually modified by this action
        modified: usize,
        /// Number of DB rows we expected this action to modify
        expected: usize,
    },
    /// We got a request for a range of assocs with too large a page size
    #[error(
        "cannot return more than {maximum_limit} requests per page of assocs \
         ({requested_limit} was requested)"
    )]
    AssocRangePageTooLarge {
        /// The user-requested page size limit
        requested_limit: usize,
        /// The maximum limit this server is configured to allow
        maximum_limit: usize,
    },
    /// We tried to update something in the database, but the number of rows we
    /// modified was wrong
    #[error(
        "CRITICAL DATA MODEL ERROR: we modified {modified} rows updating id \
         {id} but we were expecting to modify {expected}"
    )]
    EntUpdateModifiedTooManyRows {
        /// The entity ID we tried to modify
        id: EntityId,
        /// The number of DB rows actually modified by this action
        modified: usize,
        /// The number of DB rows we expected this action to modify
        expected: usize,
    },
    /// Something in the stirage layer failed — either we've made some mistake
    /// constructing queries, or blown a limit we didn't know about
    #[error("storage layer error: {0}")]
    StorageError(#[source] anyhow::Error),
    /// The persistence layer returned zero for an ID, which is invalid.
    #[error("got an id with the value zero")]
    ZeroIsNotAValidID,
    /// The persistence layer returned zero for a type, which is invalid.
    #[error("got a type with the value zero")]
    ZeroIsNotAValidType,
    /// The persistence layer returned zero for a type, which is invalid.
    #[error("a thread panicked while holding a shared TeaConnection")]
    SharedResourcePoisoned,
}
impl<T> From<PoisonError<T>> for TeaError {
    fn from(_: PoisonError<T>) -> Self {
        TeaError::SharedResourcePoisoned
    }
}
