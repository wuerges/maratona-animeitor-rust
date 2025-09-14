use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, ToSchema)]
pub struct Time(i64);

impl Time {
    pub fn unknown() -> Self {
        Self(-1)
    }
}
