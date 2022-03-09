use tea::TeaConnection;

use crate::{
    entities::EntityTypeID,
    state::{Dirty, SaveResult},
    Ent, PersistedState, SaveError, Saved,
};
use std::marker::PhantomData;

pub type AssocType = u64;

/// Got an ID and 2 objects? You're an assoc!
pub trait AssocTypeID {
    const TYPE_ID: u64;
}

pub struct Assoc<'from, 'to, F, A: AssocTypeID, T, S: PersistedState = Dirty>
where
    A: AssocTypeID,
    F: EntityTypeID,
    T: EntityTypeID,
    S: PersistedState,
{
    pub from: &'from Ent<F>,
    pub to: &'to Ent<T>,

    state: PhantomData<S>,
    kind: PhantomData<A>,
}

impl<'from, 'to, F, A, T> Assoc<'from, 'to, F, A, T, Dirty>
where
    A: AssocTypeID,
    F: EntityTypeID,
    T: EntityTypeID,
{
    pub fn new(from: &'from Ent<F>, to: &'to Ent<T>) -> Self {
        Self {
            from,
            to,
            state: PhantomData::<Dirty>::default(),
            kind: PhantomData::default(),
        }
    }

    fn into_saved(self) -> Assoc<'from, 'to, F, A, T, Saved<()>> {
        let Assoc { from, to, kind, .. } = self;
        Assoc {
            from,
            to,
            state: PhantomData::<Saved<()>>::default(),
            kind,
        }
    }
}

impl<'db, 'from, 'to, F, A, T> Assoc<'from, 'to, F, A, T, Dirty>
where
    A: AssocTypeID,
    F: EntityTypeID,
    T: EntityTypeID,
{
    pub fn save(
        self,
        db: &'db mut dyn TeaConnection,
    ) -> SaveResult<Assoc<'from, 'to, F, A, T, Saved<()>>> {
        let Assoc { from, to, .. } = self;
        let id1 = tea::EntityId::from_u64(from.id())?;
        let id2 = tea::EntityId::from_u64(to.id())?;
        let a_type = tea::AssocType::from_u64(A::TYPE_ID)?;
        if let Err(e) = db.assoc_add(a_type, id1, id2, &[]) {
            return Err(SaveError::Tea(e));
        }
        let new_assoc = Assoc::new(from, to);
        Ok(new_assoc.into_saved())
    }
}
