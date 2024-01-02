use serde::{Deserialize, Serialize};

use crate::{session, Integer, Text};

#[derive(Serialize, Deserialize, Default)]
pub struct CreateAccount {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Login {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct LoginReply {
    pub id: Integer,
    pub kind: Integer,
}

#[derive(Serialize, Deserialize, Default)]
pub struct GetAllUsersRequest {
    pub cookie: session::Cookie,
}

#[derive(Serialize, Deserialize, Default)]
pub struct User {
    pub id: Integer,
    pub name: Text,
    pub email: Text,
    pub kind: Integer,
}

pub type GetAllUsersReply = Vec<User>;
