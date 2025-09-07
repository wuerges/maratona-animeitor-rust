use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Time(i64);

impl Time {
    pub fn unknown() -> Self {
        Self(-1)
    }
}
