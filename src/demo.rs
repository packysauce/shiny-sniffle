use crate::Assoc;

use super::Dirty;
use super::RawAssoc;
use super::{Entity, RawEntity};
use serde::{Deserialize, Serialize};

pub struct Authored(RawAssoc);
pub struct AuthoredBy(RawAssoc);

impl AsRef<RawAssoc> for AuthoredBy {
    fn as_ref(&self) -> &RawAssoc {
        &self.0
    }
}

pub trait AuthoredAssoc {
    fn authored<Ent: Entity>(&self, what: &Ent) -> Dirty<Authored, RawAssoc>;
    fn authored_by<Ent: Entity>(&self, what: &Ent) -> Dirty<AuthoredBy, RawAssoc>;
}

// never one, without the other
impl<T> AuthoredAssoc for T
where
    T: Entity,
{
    fn authored<Ent: Entity>(&self, what: &Ent) -> Dirty<Authored, RawAssoc> {
        Dirty::new(Authored(RawAssoc {
            from: self.to_entity(),
            to: what.to_entity(),
            ty: 1,
        }))
    }

    fn authored_by<Ent: Entity>(&self, what: &Ent) -> Dirty<AuthoredBy, RawAssoc> {
        Dirty::new(AuthoredBy(RawAssoc {
            from: self.to_entity(),
            to: what.to_entity(),
            ty: 2,
        }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Book {
    title: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Play {
    title: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Person {
    name: String,
}

impl Person {
    pub fn new(name: &str) -> super::Dirty<Person, RawEntity> {
        Dirty::new(Self {
            name: name.to_string(),
        })
    }
}

impl Comment {
    pub fn new(s: &str) -> super::Dirty<Comment, RawEntity> {
        Dirty::new(Self {
            text: s.to_string(),
        })
    }
}

impl Book {
    pub fn new(title: &str, description: &str) -> super::Dirty<Book, RawEntity> {
        Dirty::new(Self {
            title: title.to_string(),
            description: description.to_string(),
        })
    }
}

impl Play {
    pub fn new(title: &str, description: &str) -> super::Dirty<Play, RawEntity> {
        Dirty::new(Self {
            title: title.to_string(),
            description: description.to_string(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to save comment")]
pub struct CommentFailure(Comment);

impl super::EntityStorage for Comment {
    type Error = CommentFailure;
    const TYPE_ID: u64 = 10;
}

#[derive(Debug, thiserror::Error)]
pub enum MakeBelieve {
    #[error("uh oh")]
    BadThing,
}

pub struct Db;
impl super::Database for Db {
    type Error = MakeBelieve;
}

#[cfg(test)]
#[test]
fn testing() -> anyhow::Result<()> {
    let db: Db = Db;
    // what a cool dude!
    let person = Person::new("james maxwell").save(&db)?;
    // lets make some stuff he did!
    let comment = Comment::new("cant wait to pat your sexy butt later").save(&db)?;
    let play = Play::new("so you think you can play", "this time its personal").save(&db)?;
    let book = Book::new(
        "magnets!",
        "10 crazy facts about electromagnetism. number 4 will shock you!",
    )
    .save(&db)?;

    // set all them assocs up
    let assocs = vec![
        comment.authored_by(&person),
        play.authored_by(&person),
        book.authored_by(&person),
    ];

    let comment_author = assocs[0].get().obj1();
    let play_author = assocs[1].get().obj1();
    let book_author = assocs[2].get().obj1();

    assert!(comment_author == play_author && play_author == book_author);

    Ok(())
}
