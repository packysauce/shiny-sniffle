use macros::{Assoc, Entity};
use rusqlite::DatabaseName;
use serde::{Deserialize, Serialize};
use wtf::Save;
use wtf::TeaConnection;
use wtf::ToEntity;
use wtf::{PersistedState, RawAssoc};

#[derive(Assoc, Debug)]
#[assoc(id = 1)]
pub struct Authored<S: PersistedState>(RawAssoc, S);
impl<S: PersistedState> PartialEq for Authored<S> {
    fn eq(&self, other: &Self) -> bool {
        self.0.from == other.0.from
    }
}

#[derive(Assoc, Debug)]
#[assoc(id = 2)]
pub struct AuthoredBy<S: PersistedState>(RawAssoc, S);
impl<S: PersistedState> PartialEq for AuthoredBy<S> {
    fn eq(&self, other: &Self) -> bool {
        self.0.to == other.0.to
    }
}
#[derive(Entity, Debug, Serialize, Deserialize)]
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

fn main() -> anyhow::Result<()> {
    let mut db = rusqlite::Connection::open_in_memory()?;
    db.initialize()?;
    // what a cool dude!
    let person = Person::new("james maxwell").save(&mut db)?;
    // lets make some stuff he did!
    let comment = Comment::new("buzz buzz").save(&mut db)?;
    let play = Play::new("so you think you can play", "this time its personal").save(&mut db)?;
    let book = Book::new(
        "magnets!",
        "10 crazy facts about electromagnetism. number 4 will shock you!",
    )
    .save(&mut db)?;

    // set all them assocs up
    let comment_author = comment.authored_by(&person).save(&mut db)?;
    let play_author = play.authored_by(&person).save(&mut db)?;
    let book_author = book.authored_by(&person).save(&mut db)?;

    assert!(comment_author == play_author && play_author == book_author);
    db.backup(DatabaseName::Main, "thingy.sqlite", None)
        .unwrap();
    Ok(())
}
