use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, ToSchema, PartialEq, Eq)]
pub struct Time {
    pub time_in_seconds: i64,
}

impl Time {
    pub fn unknown() -> Self {
        Self {
            time_in_seconds: -1,
        }
    }
}
