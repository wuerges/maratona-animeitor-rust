use serde::{Deserialize, Serialize};

use crate::{ContestFile, RunTuple, TimeFile};

#[derive(Deserialize, Serialize, Debug)]
pub struct ContestState {
    pub runs: Vec<RunTuple>,
    pub time: TimeFile,
    pub contest: ContestFile,
}
