use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use data::configdata::ConfigContest;
use tokio::sync::RwLock;

use crate::components::rejection::{Conflict, NotFound};

use super::runs::Runs;
use super::timer::Timer;

pub struct App {
    pub contests: RwLock<HashMap<String, Arc<ContestApp>>>,
    timeout: Duration,
}

impl App {
    pub fn new(timeout: Duration) -> Self {
        Self {
            contests: RwLock::new(HashMap::new()),
            timeout,
        }
    }

    pub async fn create_contest(&self, name: &str) -> Result<Arc<ContestApp>, Conflict> {
        if self.contests.read().await.contains_key(name) {
            return Err(Conflict);
        }

        let contest = Arc::new(ContestApp::new(self.timeout));

        self.contests
            .write()
            .await
            .insert(name.to_string(), contest.clone());

        Ok(contest)
    }

    pub async fn get_contest(&self, name: &str) -> Result<Arc<ContestApp>, NotFound> {
        let contests = self.contests.read().await.get(name).cloned();

        contests.ok_or(NotFound)
    }
}

pub struct ContestApp {
    pub runs: Runs,
    pub time: Timer,
    pub sedes: RwLock<HashMap<String, ConfigContest>>,
    timeout: Duration,
}

impl ContestApp {
    fn new(timeout: Duration) -> Self {
        Self {
            runs: Runs::new(timeout),
            time: Timer::new(timeout),

            timeout,
            sedes: RwLock::new(HashMap::new()),
        }
    }
}
