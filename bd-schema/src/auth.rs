use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateAccount {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Login {
    pub email: String,
    pub password: String,
}
