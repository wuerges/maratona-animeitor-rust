use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct WindowScroll {
    pub y: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct QueryString {
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum ControlMessage {
    WindowScroll(WindowScroll),
    QueryString(QueryString),
}
