//! Implementation of `tea` on `sqlite` databases.
//! ==============================================
//!
//! The sqlite implementation provides a relatively self-contained instance of
//! `tea` — you can use it in a network-isolated system to maintain a data store
//! that uses our abstractions, or as an in-memory mock instance for testing.

use super::types::{
    AssocRangeAfter, AssocRangeLimit, AssocStorage, AssocType, EntityId, EntityType,
};
use super::{Result, TeaConnection, TeaError};
use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::{params, Connection, ToSql};
use std::convert::TryInto;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use thiserror::Error;

config::config! {
    /// Maximum number of associations that can be fetched in a single call
    /// to `assoc_range()`, regardless of `limit`
    MAX_ASSOCS_PER_PAGE: usize = 500;
    /// Maximum number of associations that can be fetched in a single call
    /// to `assoc_range()`, regardless of `limit`
    DEFAULT_ASSOCS_PER_PAGE: usize = 100;
}

/// Newtype wrapper over [`rusqlite::Connection`] implementing
/// [`tea::TeaConnection`].
///
/// This wart is required because neither the `Connection` nor `TeaConnection`
/// symbols originate in this crate.
pub struct TeaSqliteConnection(Connection);
impl Deref for TeaSqliteConnection {
    type Target = Connection;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for TeaSqliteConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<Connection> for TeaSqliteConnection {
    fn from(conn: Connection) -> Self {
        Self(conn)
    }
}
impl TeaSqliteConnection {
    /// Open the given sqlite database path and initialize a [`TeaConnection`]
    /// to it. Note that this will create `tea` tables if they're mising.
    pub fn new(db: impl AsRef<Path>) -> Result<Self> {
        let conn = rusqlite::Connection::open(db.as_ref()).map_err(TeaSqliteError::wrap)?;
        let mut tc = Self(conn);
        tc.initialize()?;
        Ok(tc)
    }
    /// Open a new in-memory sqlite database and initialize it for Tea
    pub fn new_in_memory() -> Result<Self> {
        let conn = rusqlite::Connection::open_in_memory().map_err(TeaSqliteError::wrap)?;
        let mut tc = Self(conn);
        tc.initialize()?;
        Ok(tc)
    }
}

impl TeaConnection for TeaSqliteConnection {
    fn initialize(&mut self) -> Result<()> {
        self.execute_batch(
            r#"
            BEGIN TRANSACTION;
            CREATE TABLE IF NOT EXISTS ents (
                id   INTEGER PRIMARY KEY NOT NULL,
                type INTEGER NOT NULL,
                data BLOB
            );
            CREATE TABLE IF NOT EXISTS assocs (
                id1                  INTEGER KEY NOT NULL,
                id2                  INTEGER KEY NOT NULL,
                type                 INTEGER KEY NOT NULL,
                last_change_unixtime INTEGER KEY NOT NULL,
                data                 BLOB,
                PRIMARY KEY (id1, id2, type)
            );
            COMMIT TRANSACTION;
        "#,
        )
        .map_err(TeaSqliteError::wrap)?;
        Ok(())
    }

    fn ent_add(&mut self, ty: EntityType, data: &[u8]) -> Result<EntityId> {
        let id: u64 = self
            .query_row(
                r#"
                INSERT INTO ents (type, data)
                VALUES (?1, ?2)
                RETURNING id
            "#,
                params![ty.as_u64(), data],
                |row| row.get(0),
            )
            .map_err(TeaSqliteError::wrap)?;
        id.try_into()
    }

    fn ent_get(&mut self, id: EntityId) -> Result<(EntityType, Vec<u8>)> {
        let mut stmt = self
            .prepare(
                r#"
            SELECT type, data
            FROM ents
            WHERE id = ?1
            "#,
            )
            .map_err(TeaSqliteError::wrap)?;
        let rows = stmt
            .query_map(params![id.as_u64()], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(TeaSqliteError::wrap)?;
        let rows: std::result::Result<Vec<(u64, Vec<u8>)>, _> = rows.collect();
        let mut rows = rows.map_err(TeaSqliteError::wrap)?;

        match rows.len() {
            0 => Err(TeaError::EntNotFound(id)),
            1 => {
                let (ty, data) = rows.pop().unwrap();
                Ok((ty.try_into()?, data))
            }
            nr_rows => Err(TeaError::EntUpdateModifiedTooManyRows {
                id,
                modified: nr_rows,
                expected: 1,
            }),
        }
    }

    fn ent_update(
        &mut self,
        id: EntityId,
        _ty: EntityType,
        data: &[u8],
    ) -> Result<(EntityType, Vec<u8>)> {
        let mut stmt = self
            .prepare(
                r#"
            UPDATE ents
            SET data = (?2)
            WHERE id = ?1
            RETURNING type
            "#,
            )
            .map_err(TeaSqliteError::wrap)?;
        let rows = stmt
            .query_map(params![id.as_u64(), data], |row| row.get(0))
            .map_err(TeaSqliteError::wrap)?;
        let rows: std::result::Result<Vec<u64>, _> = rows.collect();
        let rows = rows.map_err(TeaSqliteError::wrap)?;

        match rows.len() {
            0 => Err(TeaError::EntNotFound(id)),
            1 => Ok((rows[0].try_into()?, data.to_vec())),
            nr_rows => Err(TeaError::EntUpdateModifiedTooManyRows {
                id,
                modified: nr_rows,
                expected: 1,
            }),
        }
    }

    fn ent_delete(&mut self, id: EntityId) -> Result<(EntityType, Vec<u8>)> {
        let txn = self.transaction().map_err(TeaSqliteError::wrap)?;

        // Find and delete all assocs with this entity on either end of them
        let mut assoc_stmt = txn
            .prepare(
                r#"
            DELETE
            FROM assocs
            WHERE id1 = ?1
               OR id2 = ?1
            "#,
            )
            .map_err(TeaSqliteError::wrap)?;
        let nr_assocs = assoc_stmt
            .execute(params![id.as_u64()])
            .map_err(TeaSqliteError::wrap)?;
        log::debug!(
            "dropping {nr_assocs} assocs referring to {id}",
            nr_assocs = nr_assocs,
            id = id,
        );
        drop(assoc_stmt);

        // Delete the entity itself
        let mut ent_stmt = txn
            .prepare(
                r#"
            DELETE
            FROM ents
            WHERE id = ?1
            RETURNING type, data
            "#,
            )
            .map_err(TeaSqliteError::wrap)?;
        let mut rows = ent_stmt
            .query_map(params![id.as_u64()], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(TeaSqliteError::wrap)?
            .collect::<Result<Vec<_>, rusqlite::Error>>()
            .map_err(TeaSqliteError::wrap)?
            .into_iter()
            .map(|(ty, data)| Ok((EntityType::from_u64(ty)?, data)))
            .collect::<Result<Vec<_>>>()?;

        let result = match rows.len() {
            0 => Err(TeaError::EntNotFound(id)),
            1 => Ok(rows.pop().unwrap()),
            nr_ents => Err(TeaError::EntUpdateModifiedTooManyRows {
                id,
                modified: nr_ents,
                expected: 1,
            }),
        };
        drop(ent_stmt);

        txn.commit().map_err(TeaSqliteError::wrap)?;
        result
    }

    fn assoc_add(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        id2: EntityId,
        data: &[u8],
    ) -> Result<()> {
        let now = Utc::now().timestamp();
        let num_rows = self
            .execute(
                r#"
                INSERT INTO assocs (type, id1, id2, last_change_unixtime, data)
                VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
                params![ty.as_u64(), id1.as_u64(), id2.as_u64(), now, data],
            )
            .map_err(TeaSqliteError::wrap)?;
        debug_assert_eq!(num_rows, 1);
        Ok(())
    }

    fn assoc_delete(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        id2: EntityId,
    ) -> Result<AssocStorage> {
        let mut stmt = self
            .prepare(
                r#"
            DELETE
            FROM assocs
            WHERE type = ?1 AND id1 = ?2 AND id2 = ?3
            RETURNING last_change_unixtime, data
            "#,
            )
            .map_err(TeaSqliteError::wrap)?;
        let rows = stmt
            .query_map(params![ty.as_u64(), id1.as_u64(), id2.as_u64()], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(TeaSqliteError::wrap)?;
        let rows: std::result::Result<Vec<(i64, Vec<u8>)>, _> = rows.collect();
        let mut rows = rows.map_err(TeaSqliteError::wrap)?;

        let (ts, data) = match rows.len() {
            0 => Err(TeaError::AssocNotFound { ty, id1, id2 }),
            1 => Ok(rows.pop().unwrap()),
            nr_rows => Err(TeaError::AssocUpdateModifiedTooManyRows {
                ty,
                id1,
                id2,
                modified: nr_rows,
                expected: 1,
            }),
        }?;

        let last_change: DateTime<Utc> = {
            let ndt = NaiveDateTime::from_timestamp(ts, 0);
            DateTime::from_utc(ndt, Utc)
        };

        let adata = AssocStorage {
            ty,
            id1,
            id2,
            last_change,
            data,
        };

        Ok(adata)
    }

    fn assoc_change_type(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        id2: EntityId,
        new_ty: AssocType,
    ) -> Result<AssocStorage> {
        let now = Utc::now().timestamp();

        let mut stmt = self
            .prepare(
                r#"
            UPDATE assocs
            SET type=?1, last_change_unixtime=?2
            WHERE type=?3 AND id1=?4 AND id2=?5
            RETURNING last_change_unixtime, data
            "#,
            )
            .map_err(TeaSqliteError::wrap)?;
        let rows = stmt
            .query_map(
                params![
                    new_ty.as_u64(),
                    now,
                    ty.as_u64(),
                    id1.as_u64(),
                    id2.as_u64()
                ],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(TeaSqliteError::wrap)?;
        let rows: std::result::Result<Vec<(i64, Vec<u8>)>, _> = rows.collect();
        let mut rows = rows.map_err(TeaSqliteError::wrap)?;

        let (ts, data) = match rows.len() {
            0 => Err(TeaError::AssocNotFound { ty, id1, id2 }),
            1 => Ok(rows.pop().unwrap()),
            nr_rows => Err(TeaError::AssocUpdateModifiedTooManyRows {
                ty,
                id1,
                id2,
                modified: nr_rows,
                expected: 1,
            }),
        }?;

        let last_change: DateTime<Utc> = {
            let ndt = NaiveDateTime::from_timestamp(ts, 0);
            DateTime::from_utc(ndt, Utc)
        };

        let adata = AssocStorage {
            ty: new_ty,
            id1,
            id2,
            last_change,
            data,
        };

        Ok(adata)
    }

    fn assoc_get(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        id2_set: &[EntityId],
        high: Option<DateTime<Utc>>,
        low: Option<DateTime<Utc>>,
    ) -> Result<Vec<AssocStorage>> {
        let max_vars_in_set = self.limit(rusqlite::limits::Limit::SQLITE_LIMIT_VARIABLE_NUMBER);
        debug_assert!(id2_set.len() <= max_vars_in_set as usize);

        let sql = format!(
            r#"
            SELECT type, id1, id2, last_change_unixtime, data
            FROM assocs
            WHERE type = ?1
              AND id1 = ?2
              AND last_change_unixtime <= ?3
              AND last_change_unixtime >= ?4
              AND id2 in ({})
            "#,
            // Make a ? per element in id2_set, joined by commas
            itertools::Itertools::intersperse(std::iter::repeat("?").take(id2_set.len()), ",")
                .collect::<String>()
        );
        let mut stmt = self.prepare(&sql).map_err(TeaSqliteError::wrap)?;
        let ty = ty.as_u64();
        let id1 = id1.as_u64();
        let id2_set: Vec<_> = id2_set.iter().map(EntityId::as_u64).collect();
        let high = high.unwrap_or_else(Utc::now).timestamp();
        let low = low.map_or(0, |dt| dt.timestamp());
        let mut query_params: Vec<&dyn ToSql> = Vec::new();
        query_params.extend(params![ty, id1, high, low]);
        query_params.extend(id2_set.iter().map(|id| id as &dyn ToSql));
        let rows = stmt
            .query_map(query_params.as_slice(), |row| {
                let last_change_unixtime = row.get(3)?;
                let ndt = NaiveDateTime::from_timestamp(last_change_unixtime, 0);
                let last_change = DateTime::from_utc(ndt, Utc);
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    last_change,
                    row.get(4)?,
                ))
            })
            .map_err(TeaSqliteError::wrap)?
            .collect::<Result<Vec<(u64, u64, u64, _, _)>, _>>()
            .map_err(TeaSqliteError::wrap)?
            .into_iter()
            .map(|(ty_, id1_, id2_, changed, data)| -> Result<AssocStorage> {
                Ok(AssocStorage {
                    ty: ty_.try_into()?,
                    id1: id1_.try_into()?,
                    id2: id2_.try_into()?,
                    last_change: changed,
                    data,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn assoc_count(&mut self, ty: AssocType, id1: EntityId) -> Result<usize> {
        let sql = r#"
            SELECT count(*)
            FROM assocs
            WHERE type = ?1
              AND id1 = ?2
        "#;
        let mut stmt = self.prepare(sql).map_err(TeaSqliteError::wrap)?;
        let nr_assocs = stmt
            .query_row(params![ty.as_u64(), id1.as_u64()], |row| row.get(0))
            .map_err(TeaSqliteError::wrap)?;
        Ok(nr_assocs)
    }

    fn assoc_range(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        after: AssocRangeAfter,
        limit: AssocRangeLimit,
    ) -> Result<Vec<AssocStorage>> {
        let maximum_limit = MAX_ASSOCS_PER_PAGE.get();
        let limit = match limit {
            AssocRangeLimit::Default => DEFAULT_ASSOCS_PER_PAGE.get(),
            AssocRangeLimit::Limit(limit) => limit,
            AssocRangeLimit::Maximum => MAX_ASSOCS_PER_PAGE.get(),
        };
        if limit > maximum_limit {
            return Err(TeaError::AssocRangePageTooLarge {
                requested_limit: limit,
                maximum_limit,
            });
        }

        let after = match after {
            AssocRangeAfter::First => 0,
            AssocRangeAfter::ID(id) => id.as_u64(),
        };

        let sql = r#"
            SELECT id2, last_change_unixtime, data
            FROM assocs
            WHERE type = ?1
              AND id1 = ?2
              AND id2 > ?3
            ORDER BY id2 ASC
            LIMIT ?4
        "#;
        let mut stmt = self.prepare(sql).map_err(TeaSqliteError::wrap)?;
        let assocs: Vec<AssocStorage> = stmt
            .query_map(params![ty.as_u64(), id1.as_u64(), after, limit], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(TeaSqliteError::wrap)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(TeaSqliteError::wrap)?
            .into_iter()
            .map(
                |(id2, last_change_unixtime, data)| -> Result<AssocStorage> {
                    let last_change: DateTime<Utc> = {
                        let ndt = NaiveDateTime::from_timestamp(last_change_unixtime, 0);
                        DateTime::from_utc(ndt, Utc)
                    };
                    let id2 = EntityId::from_u64(id2)?;
                    Ok(AssocStorage {
                        ty,
                        id1,
                        id2,
                        last_change,
                        data,
                    })
                },
            )
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(assocs)
    }

    fn assoc_time_range(
        &mut self,
        ty: AssocType,
        id1: EntityId,
        high: DateTime<Utc>,
        low: DateTime<Utc>,
        limit: AssocRangeLimit,
    ) -> Result<Vec<AssocStorage>> {
        let maximum_limit = MAX_ASSOCS_PER_PAGE.get();
        let limit = match limit {
            AssocRangeLimit::Default => DEFAULT_ASSOCS_PER_PAGE.get(),
            AssocRangeLimit::Limit(limit) => limit,
            AssocRangeLimit::Maximum => MAX_ASSOCS_PER_PAGE.get(),
        };
        if limit > maximum_limit {
            return Err(TeaError::AssocRangePageTooLarge {
                requested_limit: limit,
                maximum_limit,
            });
        }

        let low = low.timestamp();
        let high = high.timestamp();

        let sql = r#"
            SELECT id2, last_change_unixtime, data
            FROM assocs
            WHERE type = ?1
              AND id1 = ?2
              AND last_change_unixtime >= ?3
              AND last_change_unixtime <= ?4
            ORDER BY last_change_unixtime DESC
            LIMIT ?5
        "#;
        let mut stmt = self.prepare(sql).map_err(TeaSqliteError::wrap)?;
        let assocs: Vec<AssocStorage> = stmt
            .query_map(
                params![ty.as_u64(), id1.as_u64(), low, high, limit],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(TeaSqliteError::wrap)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(TeaSqliteError::wrap)?
            .into_iter()
            .map(
                |(id2, last_change_unixtime, data)| -> Result<AssocStorage> {
                    let last_change: DateTime<Utc> = {
                        let ndt = NaiveDateTime::from_timestamp(last_change_unixtime, 0);
                        DateTime::from_utc(ndt, Utc)
                    };
                    let id2 = EntityId::from_u64(id2)?;
                    Ok(AssocStorage {
                        ty,
                        id1,
                        id2,
                        last_change,
                        data,
                    })
                },
            )
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(assocs)
    }
}

/// Errors for the `SQLite` Tea backend
#[derive(Error, PartialEq, Debug)]
pub enum TeaSqliteError {
    /// Something in the SQLite layer failed — either we've made some mistake
    /// constructing queries, or blown a limit we didn't know about
    #[error("rusqlite error: {0}")]
    SqliteStorageError(#[from] rusqlite::Error),
}
impl From<TeaSqliteError> for TeaError {
    fn from(val: TeaSqliteError) -> Self {
        TeaError::StorageError(val.into())
    }
}
impl TeaSqliteError {
    /// Wrap a rusqlite error into a `TeaError`
    pub fn wrap(err: rusqlite::Error) -> TeaError {
        TeaSqliteError::SqliteStorageError(err).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unused)]
    fn init_test_db() -> anyhow::Result<TeaSqliteConnection> {
        let mut conn: TeaSqliteConnection = Connection::open_in_memory()?.into();
        conn.initialize()?;
        Ok(conn)
    }

    #[test]
    fn ent_crud() -> anyhow::Result<()> {
        let mut conn = init_test_db()?;
        conn.initialize()?;

        // Create an ent
        let etype = EntityType::from_u64(1)?;
        let id = conn.ent_add(etype, &[])?;

        // Get that ent
        let (etype_, data) = conn.ent_get(id)?;
        assert_eq!(etype, etype_, "fetched etype differs from expected");
        assert_eq!(b"", data.as_slice());

        // Update that ent
        conn.ent_update(id, etype_, b"hello\0")?;

        // Get that ent again and check the data
        let (etype_, data) = conn.ent_get(id)?;
        assert_eq!(etype, etype_, "fetched etype differs from expected");
        assert_eq!(b"hello\0", data.as_slice());

        // Delete that ent
        let (etype_, data) = conn.ent_delete(id)?;
        assert_eq!(etype, etype_, "deleted etype differs from expected");
        assert_eq!(b"hello\0", data.as_slice());

        // Confirm it's not found
        match conn.ent_get(id).unwrap_err() {
            TeaError::EntNotFound(got_id) => {
                assert_eq!(id, got_id, "searched the wrong id?");
            }
            _ => panic!("expected notfound"),
        }

        Ok(())
    }

    #[test]
    fn assoc_count_single() -> anyhow::Result<()> {
        let mut conn = init_test_db()?;

        let etype1 = EntityType::from_u64(1)?;
        let id1 = conn.ent_add(etype1, &[])?;
        let id2 = conn.ent_add(etype1, &[])?;

        let atype1 = AssocType::from_u64(1)?;
        conn.assoc_add(atype1, id1, id2, &[])?;

        let count = conn.assoc_count(atype1, id1)?;
        assert_eq!(count, 1);

        Ok(())
    }

    #[test]
    fn assoc_count_checks_type() -> anyhow::Result<()> {
        let mut conn = init_test_db()?;

        let etype1 = EntityType::from_u64(1)?;
        let id1 = conn.ent_add(etype1, &[])?;
        let id2 = conn.ent_add(etype1, &[])?;

        let atype1 = AssocType::from_u64(1)?;
        let atype2 = AssocType::from_u64(2)?;
        conn.assoc_add(atype1, id1, id2, &[])?;
        conn.assoc_add(atype2, id1, id2, &[])?;

        let count = conn.assoc_count(atype1, id1)?;
        assert_eq!(count, 1);

        Ok(())
    }

    #[test]
    fn assoc_count_multiple() -> anyhow::Result<()> {
        let mut conn = init_test_db()?;

        let etype1 = EntityType::from_u64(1)?;
        let id1 = conn.ent_add(etype1, &[])?;
        let id2 = conn.ent_add(etype1, &[])?;
        let id3 = conn.ent_add(etype1, &[])?;
        let id4 = conn.ent_add(etype1, &[])?;

        let atype1 = AssocType::from_u64(1)?;
        conn.assoc_add(atype1, id1, id2, &[])?;
        conn.assoc_add(atype1, id1, id3, &[])?;
        conn.assoc_add(atype1, id1, id4, &[])?;

        let count = conn.assoc_count(atype1, id1)?;
        assert_eq!(count, 3);

        Ok(())
    }

    #[test]
    fn assoc_delete() -> anyhow::Result<()> {
        let mut conn = init_test_db()?;

        let etype1 = EntityType::from_u64(1)?;
        let id1 = conn.ent_add(etype1, &[])?;
        let id2 = conn.ent_add(etype1, &[])?;

        let atype1 = AssocType::from_u64(1)?;
        conn.assoc_add(atype1, id1, id2, &[])?;
        let count = conn.assoc_count(atype1, id1)?;
        assert_eq!(count, 1);

        conn.assoc_delete(atype1, id1, id2)?;
        let count = conn.assoc_count(atype1, id1)?;
        assert_eq!(count, 0);

        Ok(())
    }

    #[test]
    fn assoc_change_type() -> anyhow::Result<()> {
        let mut conn = init_test_db()?;

        let etype1 = EntityType::from_u64(1)?;
        let id1 = conn.ent_add(etype1, &[])?;
        let id2 = conn.ent_add(etype1, &[])?;

        let atype1 = AssocType::from_u64(1)?;
        conn.assoc_add(atype1, id1, id2, &[])?;
        let fetched = conn.assoc_get(atype1, id1, &[id2], None, None)?;
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].ty, atype1);

        // Change the type and re-query
        let atype2 = AssocType::from_u64(2)?;
        conn.assoc_change_type(atype1, id1, id2, atype2)?;
        let fetched = conn.assoc_get(atype2, id1, &[id2], None, None)?;
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].ty, atype2);

        // Check that the old assoc is gone
        let empty = conn.assoc_get(atype1, id1, &[id2], None, None)?;
        assert!(
            empty.is_empty(),
            "After changing the type, the assoc was still retrieved by the \
             old type"
        );

        Ok(())
    }

    #[test]
    fn assoc_get_smoketest() -> anyhow::Result<()> {
        let mut conn = init_test_db()?;

        // Round the start time to the nearest second, since that's the
        // granularity we keep in the database.
        let start: DateTime<Utc> = {
            let now_ts = Utc::now().timestamp();
            let ndt = NaiveDateTime::from_timestamp(now_ts, 0);
            DateTime::from_utc(ndt, Utc)
        };

        // Insert 3 entities
        let etype = EntityType::from_u64(1)?;
        let id1 = conn.ent_add(etype, &[])?;
        let id2 = conn.ent_add(etype, &[])?;
        let id3 = conn.ent_add(etype, &[])?;

        // Add some assocs
        let atype = AssocType::from_u64(1)?;
        conn.assoc_add(atype, id1, id2, &[])?;
        conn.assoc_add(atype, id3, id2, &[])?;
        conn.assoc_add(atype, id1, id3, &[])?;

        // Query them
        let assocs = conn.assoc_get(atype, id1, &[id3], None, None)?;

        assert_eq!(assocs.len(), 1);
        let assoc = &assocs[0];
        assert_eq!(assoc.ty, atype);
        assert_eq!(assoc.id1, id1);
        assert_eq!(assoc.id2, id3);
        assert!(
            assoc.last_change >= start,
            "{} should be after {}",
            assoc.last_change,
            start
        );
        assert!(
            assoc.data.is_empty(),
            "we did not add any data so none should be returned"
        );

        Ok(())
    }

    /// Check that we delete assocs referring to an ent when we delete that ent.
    /// In other words: no hanging assocs.
    #[test]
    fn ent_delete_includes_references() -> anyhow::Result<()> {
        let mut conn = init_test_db()?;

        // Insert 3 entities
        let etype = EntityType::from_u64(1)?;
        let id1 = conn.ent_add(etype, &[])?;
        let id2 = conn.ent_add(etype, &[])?;
        let id3 = conn.ent_add(etype, &[])?;

        // Add some assocs
        let atype = AssocType::from_u64(1)?;
        conn.assoc_add(atype, id1, id2, &[])?;
        conn.assoc_add(atype, id3, id2, &[])?;
        conn.assoc_add(atype, id1, id3, &[])?;

        // Delete ent 3
        conn.ent_delete(id3)?;

        // Check that the only assoc we can find is the one from 1->2
        let assocs_1_3 = conn.assoc_get(atype, id1, &[id3], None, None)?;
        assert_eq!(assocs_1_3.len(), 0);
        let assocs_3_2 = conn.assoc_get(atype, id3, &[id2], None, None)?;
        assert_eq!(assocs_3_2.len(), 0);

        let assocs_1_2 = conn.assoc_get(atype, id1, &[id2], None, None)?;
        assert_eq!(assocs_1_2.len(), 1);

        Ok(())
    }

    #[test]
    fn assoc_range_all_on_one_page() -> anyhow::Result<()> {
        let mut conn = init_test_db()?;

        // Insert 3 entities
        let etype = EntityType::from_u64(1)?;
        let id1 = conn.ent_add(etype, &[])?;
        let id2 = conn.ent_add(etype, &[])?;
        let id3 = conn.ent_add(etype, &[])?;

        // Add some assocs
        let atype = AssocType::from_u64(1)?;
        conn.assoc_add(atype, id1, id2, &[])?;
        conn.assoc_add(atype, id1, id3, &[])?;

        // Fetch them both
        let assocs =
            conn.assoc_range(atype, id1, AssocRangeAfter::First, AssocRangeLimit::Default)?;

        assert_eq!(assocs.len(), 2);
        for (assoc, id) in assocs.into_iter().zip(&[id2, id3]) {
            assert_eq!(assoc.ty, atype);
            assert_eq!(assoc.id1, id1);
            assert_eq!(assoc.id2, *id);
            assert!(
                assoc.data.is_empty(),
                "we did not add any data so none should be returned"
            );
        }

        Ok(())
    }

    #[test]
    fn assoc_range_pagination() -> anyhow::Result<()> {
        let mut conn = init_test_db()?;

        // Insert 3 entities
        let etype = EntityType::from_u64(1)?;
        let id1 = conn.ent_add(etype, &[])?;
        let id2 = conn.ent_add(etype, &[])?;
        let id3 = conn.ent_add(etype, &[])?;

        // Add some assocs
        let atype = AssocType::from_u64(1)?;
        conn.assoc_add(atype, id1, id2, &[])?;
        conn.assoc_add(atype, id1, id3, &[])?;

        // Fetch both pages
        let assocs_pg1 = conn.assoc_range(
            atype,
            id1,
            AssocRangeAfter::First,
            AssocRangeLimit::Limit(1),
        )?;
        assert_eq!(assocs_pg1.len(), 1);
        let last_id2 = assocs_pg1[0].id2;
        let assocs_pg2 = conn.assoc_range(
            atype,
            id1,
            AssocRangeAfter::ID(last_id2),
            AssocRangeLimit::Default,
        )?;
        assert_eq!(assocs_pg2.len(), 1);

        let assocs = assocs_pg1.into_iter().chain(assocs_pg2.into_iter());
        for (assoc, id) in assocs.zip(&[id2, id3]) {
            assert_eq!(assoc.ty, atype);
            assert_eq!(assoc.id1, id1);
            assert_eq!(assoc.id2, *id);
            assert!(
                assoc.data.is_empty(),
                "we did not add any data so none should be returned"
            );
        }

        Ok(())
    }
}
