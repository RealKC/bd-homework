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

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct User {
    pub id: Integer,
    pub name: Text,
    pub email: Text,
    pub kind: Integer,
    pub borrowed_book_count: Integer,
}

pub type GetAllUsersReply = Vec<User>;

#[derive(Serialize, Deserialize, Default)]
pub struct PromoteUserRequest {
    pub user_to_be_promoted: Integer,
    pub cookie: session::Cookie,
}

#[derive(Serialize, Deserialize, Default)]
pub struct DeleteUserRequest {
    pub user_to_be_deleted: Integer,
    pub cookie: session::Cookie,
}

#[derive(Serialize, Deserialize, Default)]
pub enum DeleteUserReply {
    #[default]
    Ok,
    UsersStillHadBooks,
    CannotDeleteSelf,
}
