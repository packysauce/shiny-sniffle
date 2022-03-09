use std::fmt::Debug;
use std::ops::Deref;
use tea::TeaError;

/// Implementation of `PersistedState` indicating the data is unsaved
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dirty<T = ()>(T);

impl Dirty {
    fn new() -> Dirty {
        Dirty(())
    }
}

impl Default for Dirty {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of `PersistedState` indicating the data is commited.
#[derive(Debug)]
pub struct Saved<Id: std::fmt::Debug>(pub(crate) Id);

impl<Id: std::fmt::Debug> Saved<Id> {
    pub fn new(id: Id) -> Self {
        Self(id)
    }
}

impl<Id: Debug> Deref for Saved<Id> {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Marker trait for Ent typestates
pub trait PersistedState {}
impl PersistedState for Dirty {}
impl<Id: std::fmt::Debug> PersistedState for Saved<Id> {}

pub type SaveResult<T> = std::result::Result<T, SaveError>;
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("Problem serializing to json")]
    Serde(#[from] serde_json::Error),
    #[error("Problem communicating with TEA backend")]
    Tea(#[from] TeaError),
}

impl<Id: std::fmt::Debug> AsRef<Id> for Saved<Id> {
    fn as_ref(&self) -> &Id {
        &self.0
    }
}
