use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures::{Stream, StreamExt};
use itertools::Itertools;
use sdk::{Site, SiteConfiguration};
use tokio::sync::RwLock;

use crate::components::rejection::{Conflict, NotFound};
use crate::endpoints::update_contest::ContestState;

use super::runs::Runs;
use super::timer::Timer;

pub struct AppV2 {
    pub contests: RwLock<HashMap<String, Arc<ContestApp>>>,
    pub server_api_key: Option<String>,
    timeout: Duration,
}

impl AppV2 {
    pub fn new(timeout: Duration, server_api_key: Option<String>) -> Self {
        Self {
            contests: RwLock::new(HashMap::new()),
            timeout,
            server_api_key,
        }
    }

    pub async fn create_contest(&self, name: &str) -> Result<Arc<ContestApp>, Conflict> {
        if self.contests.read().await.contains_key(name) {
            return Err(Conflict);
        }

        let contest = Arc::new(ContestApp::new(self.timeout).await);

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
    pub sedes: RwLock<HashMap<String, Site>>,
    pub file: RwLock<ContestFile>,
}

impl ContestApp {
    async fn new(timeout: Duration) -> Self {
        Self {
            runs: Runs::new(timeout).await,
            time: Timer::new(timeout),
            file: RwLock::new(ContestFile::dummy()),
            sedes: RwLock::new(HashMap::new()),
        }
    }

    pub async fn update_state(&self, create_runs: ContestState) {
        let ContestState { runs, time } = create_runs;

        self.runs.push_ordered(runs).await;
        self.time.update(time);
    }

    pub async fn update_config(&self, contest_file: ContestFile) {
        let mut file = self.file.write().await;

        *file = contest_file;
    }

    pub async fn update_sedes(&self, SiteConfiguration { base, sites }: SiteConfiguration) {
        let new_config = [("".to_string(), base)]
            .into_iter()
            .chain(sites.into_iter().map(|s| (s.name.clone(), s)))
            .collect::<HashMap<_, _>>();

        *self.sedes.write().await = new_config;
    }

    pub async fn get_runs(&self) -> impl Stream<Item = impl Future<Output = Vec<sdk::Run>>> {
        self.runs.stream().await.map(async |r| {
            let hash_map = self.sedes.read().await;
            let sede = hash_map.get("");

            r.into_iter()
                .filter(|r| sede.is_some_and(|s| s.team_belongs_str(&r.team_login)))
                .collect_vec()
        })
    }
}
