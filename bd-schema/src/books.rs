use serde::{Deserialize, Serialize};

use crate::{session, Integer, Text};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Author {
    pub author_id: Integer,
    pub name: Text,
    pub date_of_birth: Integer,
    pub date_of_death: Option<Integer>,
    pub description: Text,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Book {
    pub book_id: Integer,
    pub title: Text,
    pub author: Author,
    pub publish_date: Integer,
    pub publisher: Text,
    pub count: Integer,
    pub synopsis: Text,
    pub can_be_borrowed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BorrowRequest {
    pub cookie: session::Cookie,
    pub book_id: Integer,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BorrowReply {
    pub already_borrowed: bool,
}
