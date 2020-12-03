use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Msg {
    Ready,
    Login,
    Logout,
}