use serde::{Deserialize, Serialize};

use crate::{Integer, Text};

#[derive(Serialize, Deserialize)]
pub struct Author {
    pub author_id: Integer,
    pub name: Text,
    pub date_of_birth: Integer,
    pub date_of_death: Option<Integer>,
    pub description: Text,
}

#[derive(Serialize, Deserialize)]
pub struct Book {
    pub book_id: Integer,
    pub title: Text,
    pub author: Author,
    pub publish_date: Integer,
    pub publisher: Text,
    pub count: Integer,
    pub synopsis: Text,
}
