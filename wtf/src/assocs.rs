use tea::TeaConnection;

use crate::{
    entities::{EntityTypeID, RawEntity},
    state::{Dirty, SaveResult},
    Ent, PersistedState, SaveError, Saved,
};
use std::marker::PhantomData;

pub type AssocType = u64;

/// Got an ID and 2 objects? You're an assoc!
pub trait AssocTypeID {
    const TYPE_ID: u64;
}

pub struct Assoc<A: AssocTypeID, F, T, S: PersistedState = Dirty>
where
    A: AssocTypeID,
    F: EntityTypeID,
    T: EntityTypeID,
    S: PersistedState,
{
    pub from: Ent<F>,
    pub to: Ent<T>,

    state: PhantomData<S>,
    kind: PhantomData<A>,
}

impl<A, F, T> Assoc<A, F, T>
where
    A: AssocTypeID,
    F: EntityTypeID,
    T: EntityTypeID,
{
    pub fn new(from: Ent<F>, to: Ent<T>) -> Self {
        Self {
            from,
            to,
            state: PhantomData::<Dirty>::default(),
            kind: PhantomData::default(),
        }
    }

    fn into_saved(self) -> Assoc<A, F, T, Saved<()>> {
        let Assoc { from, to, kind, .. } = self;
        Assoc {
            from,
            to,
            state: PhantomData::<Saved<()>>::default(),
            kind,
        }
    }
}

/// Storage of an assocation. If you think of an assocation as an arrow,
/// then the base of the arrow is the "from" entity, and the "to" entity
/// is being pointed at by the arrow.
#[derive(Debug, Clone, Copy, PartialEq)]
struct RawAssoc {
    // In TAO parlance, id1
    pub from: RawEntity,
    // In TAO parlance, id2
    pub to: RawEntity,
    // In TAO parlance, atype
    ty: AssocType,
}

impl<A, F, T> Assoc<A, F, T, Dirty>
where
    A: AssocTypeID,
    F: EntityTypeID,
    T: EntityTypeID,
{
    pub fn save(self, db: &mut dyn TeaConnection) -> SaveResult<Assoc<A, F, T, Saved<()>>> {
        let id1 = tea::EntityId::from_u64(self.from.id())?;
        let id2 = tea::EntityId::from_u64(self.to.id())?;
        let a_type = tea::AssocType::from_u64(A::TYPE_ID)?;
        if let Err(e) = db.assoc_add(a_type, id1, id2, &[]) {
            return Err(SaveError::Tea(e));
        }
        let new_assoc = Assoc::new(self.from, self.to);
        Ok(new_assoc.into_saved())
    }
}
