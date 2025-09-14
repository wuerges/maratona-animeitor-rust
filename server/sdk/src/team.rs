use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// A team in the contest.
pub struct Team {
    /// BOCA's login.
    pub login: String,
    /// The school of the team.
    pub escola: String,
    /// The name of the team.
    pub name: String,
}
