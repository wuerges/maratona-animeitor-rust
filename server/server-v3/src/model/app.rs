use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures::{Stream, StreamExt};
use itertools::Itertools;
use sdk::{ContestParameters, ContestState, Site, SiteConfiguration};
use tokio::sync::RwLock;
use tracing::{Level, debug, instrument};

use crate::components::rejection::{Conflict, NotFound};

use super::runs::Runs;
use super::timer::Timer;

#[derive(Clone)]
pub struct AppV2 {
    pub contests: Arc<RwLock<HashMap<String, Arc<ContestApp>>>>,
    pub server_api_key: Option<String>,
    timeout: Duration,
}

impl AppV2 {
    pub fn new(timeout: Duration, server_api_key: Option<String>) -> Self {
        Self {
            contests: Arc::new(RwLock::new(HashMap::new())),
            timeout,
            server_api_key,
        }
    }

    pub async fn create_contest(&self, contest: sdk::Contest) -> Result<Arc<ContestApp>, Conflict> {
        if self
            .contests
            .read()
            .await
            .contains_key(&contest.contest_name)
        {
            return Err(Conflict);
        }

        let name = contest.contest_name.clone();
        let contest = Arc::new(ContestApp::new(self.timeout, contest).await);

        self.contests
            .write()
            .await
            .insert(name.to_string(), contest.clone());

        Ok(contest)
    }

    #[instrument(skip(self),  err(level = Level::DEBUG))]
    pub async fn get_contest(&self, name: &str) -> Result<Arc<ContestApp>, NotFound> {
        let contests = self.contests.read().await.get(name).cloned();

        contests.ok_or(NotFound)
    }
}

pub struct ContestApp {
    pub runs: Runs,
    pub time: Timer,
    pub sedes: RwLock<HashMap<String, Site>>,
    pub contest_name: String,
    pub contest: RwLock<sdk::ContestParameters>,
}

impl ContestApp {
    async fn new(
        timeout: Duration,
        sdk::Contest {
            contest_name,
            parameters,
        }: sdk::Contest,
    ) -> Self {
        Self {
            runs: Runs::new().await,
            time: Timer::new(timeout),
            contest: RwLock::new(parameters),
            sedes: RwLock::new(HashMap::new()),
            contest_name,
        }
    }

    pub async fn update_state(&self, create_runs: ContestState) {
        let ContestState { runs, time } = create_runs;

        self.runs.push_ordered(runs).await;
        self.time.update(time);
    }

    pub async fn reset_state(&self) {
        self.time.reset();
        self.runs.reset().await;
    }

    pub async fn update_parameters(&self, parameters: ContestParameters) {
        let mut file = self.contest.write().await;

        *file = parameters;
    }

    pub async fn update_site_configuration(
        &self,
        SiteConfiguration { base, sites }: SiteConfiguration,
    ) {
        let new_config = [("".to_string(), base)]
            .into_iter()
            .chain(sites.into_iter().map(|s| (s.name.clone(), s)))
            .collect::<HashMap<_, _>>();

        *self.sedes.write().await = new_config;
    }

    #[instrument(skip_all)]
    pub async fn get_runs(&self) -> impl Stream<Item = impl Future<Output = Vec<sdk::Run>>> {
        self.runs.stream().await.map(async |r| {
            debug!(?r, "batch of runs");
            let hash_map = self.sedes.read().await;
            let sede = hash_map.get("");

            r.into_iter()
                .filter(|r| sede.is_none_or(|s| s.team_belongs(&r.team_login)))
                .collect_vec()
        })
    }
}
