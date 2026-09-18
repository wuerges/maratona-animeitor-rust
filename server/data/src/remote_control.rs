use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, utoipa::ToSchema)]
/// Bare remote-control message: {"y":120}.
pub struct WindowScroll {
    /// Vertical browser scroll position.
    pub y: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, utoipa::ToSchema)]
/// Bare JSON "Hidden" or {"Show":"team-login"}.
pub enum PhotoState {
    Hidden,
    Show(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, utoipa::ToSchema)]
/// Bare remote-control message: {"query":"sede=fiemg"}.
pub struct QueryString {
    /// Frontend query string to synchronize.
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, utoipa::ToSchema)]
#[serde(untagged)]
/// Untagged union: no outer WindowScroll, QueryString, or PhotoState wrapper.
pub enum ControlMessage {
    WindowScroll(WindowScroll),
    QueryString(QueryString),
    PhotoState(PhotoState),
}
