use rusqlite::DatabaseName;
use serde::{Deserialize, Serialize};
use wtf::{Assoc, Ent, EntityTypeID, SaveEnt, TeaConnection};
use wtf_macros::{Assoc, Entity};

#[derive(Assoc)]
#[assoc(id = 1, forward = "Authored", reverse = "AuthoredBy")]
pub struct Author;

#[derive(Entity, Debug, Serialize, Deserialize)]
#[entity(id = 11)]
pub struct Book {
    title: String,
    description: String,
}

#[derive(Entity, Debug, Serialize, Deserialize)]
#[entity(id = 12)]
pub struct Play {
    title: String,
    description: String,
}

#[derive(Entity, Debug, Serialize, Deserialize)]
#[entity(id = 13)]
pub struct Comment {
    text: String,
}

#[derive(Entity, Debug, Serialize, Deserialize)]
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
    let mut db = tea::sqlite::TeaSqliteConnection::new_in_memory()?;
    TeaConnection::initialize(&mut db)?;
    // The generated types aren't all that yucky
    let person: Ent<Person> = Person::new("james maxwell").save(&mut db)?;
    let comment = Comment::new("buzz buzz").save(&mut db)?;
    let play = Play::new("so you think you can play", "this time its personal").save(&mut db)?;
    let book = Book::new(
        "magnets!",
        "10 crazy facts about electromagnetism. number 4 will shock you!",
    )
    .save(&mut db)?;

    // set all them assocs up
    let comment_author = person.authored(&comment).save(&mut db)?;
    let play_author = play.authored_by(&person).save(&mut db)?;
    let book_author = book.authored_by(&person).save(&mut db)?;

    //assert!(comment_author == play_author && play_author == book_author);
    db.backup(DatabaseName::Main, "thingy.sqlite", None)
        .unwrap();
    Ok(())
}
