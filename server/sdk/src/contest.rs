use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{Run, Team, Time};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// A contest serialized in the api response.
pub struct ContestParameters {
    /// Map of the teams.
    pub teams: Vec<Team>,
    /// Maximum time (contest ends).
    pub maximum_time_in_minutes: u32,
    /// Time that score gets frozen.
    pub score_freeze_time_in_minutes: u32,
    /// Penalty per wrong answer.
    pub penalty_per_wrong_answer: u32,
    /// Number of problems in the contest.
    pub number_problems: usize,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct ContestState {
    pub runs: Vec<Run>,
    pub time: Time,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct ContestSecret {
    pub secret: String,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct Contest {
    /// Name of the contest.
    pub contest_name: String,

    /// Contest configuration parameters.
    pub parameters: ContestParameters,
}
