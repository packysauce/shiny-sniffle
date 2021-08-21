use crate::PersistedState;
use crate::RawEntity;
use crate::Save;
use crate::SaveError;
use crate::Saved;

pub use super::Dirty;
use super::Entity;
use super::RawAssoc;
use rusqlite::DatabaseName;
use serde::{Deserialize, Serialize};
use tea::EntityType;
use tea::TeaConnection;

pub struct Authored<S: PersistedState>(RawAssoc, S);
impl<S: PersistedState> PartialEq for Authored<S> {
    fn eq(&self, other: &Self) -> bool {
        self.0.from == other.0.from
    }
}

pub struct AuthoredBy<S: PersistedState>(RawAssoc, S);
impl<S: PersistedState> PartialEq for AuthoredBy<S> {
    fn eq(&self, other: &Self) -> bool {
        self.0.to == other.0.to
    }
}

impl AsRef<RawAssoc> for AuthoredBy<Saved<RawAssoc>> {
    fn as_ref(&self) -> &RawAssoc {
        &self.0
    }
}

pub trait AuthoredAssoc {
    fn authored<Ent: Entity>(&self, what: &Ent) -> Authored<Dirty>;
    fn authored_by<Ent: Entity>(&self, what: &Ent) -> AuthoredBy<Dirty>;
}

// never one, without the other
impl<T> AuthoredAssoc for T
where
    T: Entity,
{
    fn authored<Ent: Entity>(&self, what: &Ent) -> Authored<Dirty> {
        Authored(
            RawAssoc {
                from: self.to_entity(),
                to: what.to_entity(),
                ty: 1,
            },
            Dirty,
        )
    }

    fn authored_by<Ent: Entity>(&self, what: &Ent) -> AuthoredBy<Dirty> {
        AuthoredBy(
            RawAssoc {
                from: self.to_entity(),
                to: what.to_entity(),
                ty: 2,
            },
            Dirty,
        )
    }
}

#[derive(macros::Entity, Debug, Serialize, Deserialize)]
#[entity(id = 11)]
pub struct Book {
    title: String,
    description: String,
}

#[derive(macros::Entity, Debug, Serialize, Deserialize)]
#[entity(id = 12)]
pub struct Play {
    title: String,
    description: String,
}

#[derive(macros::Entity, Debug, Serialize, Deserialize)]
#[entity(id = 13)]
pub struct Comment {
    text: String,
}

#[derive(macros::Entity, Debug, Serialize, Deserialize)]
#[entity(id = 10)]
pub struct Person {
    name: String,
}

impl Person {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

pub trait ToEntity {
    type Entity;

    fn entity_type() -> EntityType;
    fn ent(self) -> Self::Entity;
}

impl Comment {
    pub fn new(s: &str) -> Self {
        Self {
            text: s.to_string(),
        }
    }
}

impl Book {
    pub fn new(title: &str, description: &str) -> Self {
        Book {
            title: title.to_string(),
            description: description.to_string(),
        }
    }
}

impl Play {
    pub fn new(title: &str, description: &str) -> Self {
        Self {
            title: title.to_string(),
            description: description.to_string(),
        }
    }
}

#[test]
fn testing() -> anyhow::Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    db.initialize()?;
    // what a cool dude!
    let person = Person::new("james maxwell").save(&mut db)?;
    // lets make some stuff he did!
    let comment = Comment::new("cant wait to pat your sexy butt later").save(&mut db)?;
    let play = Play::new("so you think you can play", "this time its personal").save(&mut db)?;
    let book = Book::new(
        "magnets!",
        "10 crazy facts about electromagnetism. number 4 will shock you!",
    )
    .save(&mut db)?;

    // set all them assocs up
    let assocs = vec![
        comment.authored_by(&person),
        play.authored_by(&person),
        book.authored_by(&person),
    ];

    let comment_author = &assocs[0];
    let play_author = &assocs[1];
    let book_author = &assocs[2];
    assert!(comment_author == play_author && play_author == book_author);
    db.backup(DatabaseName::Main, "thingy.sqlite", None).unwrap();
    Ok(())
}
