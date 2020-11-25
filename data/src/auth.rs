use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserKey {
    pub contest_number : i32,
    pub site_number : i32,
    pub user_number : i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub login : String,
    pub password :String,
}