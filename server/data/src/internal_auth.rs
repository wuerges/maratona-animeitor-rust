//! Public descriptions of internal credential permissions; never includes secrets.
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum InternalRole {
    ReadOnly,
    ReadWrite,
}

#[derive(Serialize, ToSchema)]
pub struct TokenCapabilities {
    pub name: String,
    pub role: InternalRole,
    /// Rust regular expressions matched against the entire event name.
    pub events: Vec<String>,
}
