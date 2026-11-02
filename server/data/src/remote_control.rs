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

/// Capability issued by the authenticated internal API.
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RemoteSessionInfo {
    pub key: String,
    /// Unix timestamp in seconds. Credentials survive restart with SQLite; defaults to seven days.
    pub expires_at: u64,
}

pub const DEFAULT_REMOTE_TTL_SECONDS: u64 = 7 * 24 * 3600;
pub const MAX_REMOTE_TTL_SECONDS: u64 = 30 * 24 * 3600;
fn default_remote_ttl() -> u64 {
    DEFAULT_REMOTE_TTL_SECONDS
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RemoteSessionRequest {
    /// Credential lifetime, 1 second through 30 days; default seven days.
    #[serde(default = "default_remote_ttl")]
    #[schema(default = 604800, minimum = 1, maximum = 2592000)]
    pub expires_in_seconds: u64,
}
impl Default for RemoteSessionRequest {
    fn default() -> Self {
        Self {
            expires_in_seconds: DEFAULT_REMOTE_TTL_SECONDS,
        }
    }
}
