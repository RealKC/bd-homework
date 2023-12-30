use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Cookie {
    pub id: i64,
    pub password: String,
}
