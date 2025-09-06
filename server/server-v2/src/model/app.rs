use std::time::Duration;

use dashmap::DashMap;
use data::configdata::ConfigContest;

use super::runs::Runs;
use super::timer::Timer;

pub struct ContestApp {
    pub runs: Runs,
    pub time: Timer,
    pub sedes: DashMap<String, ConfigContest>,
    timeout: Duration,
}

impl ContestApp {
    pub fn new(timeout: Duration) -> Self {
        Self {
            runs: Runs::new(timeout),
            time: Timer::new(timeout),

            timeout,
            sedes: DashMap::new(),
        }
    }
}
