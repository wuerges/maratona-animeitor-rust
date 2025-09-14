use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures::{Stream, StreamExt};
use itertools::Itertools;
use sdk::{ContestParameters, ContestState, SiteConfiguration};
use tokio::sync::RwLock;
use tracing::{Level, debug, instrument};

use crate::components::rejection::Conflict;

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

    #[instrument(skip_all)]
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

        debug!(?name);
        self.contests.write().await.insert(name, contest.clone());

        Ok(contest)
    }

    #[instrument(skip(self))]
    pub async fn get_contest(&self, name: &str) -> Option<Arc<ContestApp>> {
        debug!(?name);
        self.contests.read().await.get(name).cloned()
    }

    #[instrument(skip(self), ret(level = Level::DEBUG))]
    pub async fn list_contests(&self) -> Vec<String> {
        self.contests
            .read()
            .await
            .values()
            .map(|c| c.contest_name.clone())
            .collect()
    }
}

pub struct ContestApp {
    runs: Runs,
    time: Timer,
    sedes: RwLock<Option<SiteConfiguration>>,
    contest_name: String,
    contest: RwLock<sdk::ContestParameters>,
    secret: RwLock<Option<String>>,
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
            sedes: RwLock::new(None),
            secret: RwLock::new(None),
            contest_name,
        }
    }

    pub async fn update_state(&self, create_runs: ContestState) {
        let ContestState { runs, time } = create_runs;

        self.runs.push_ordered(runs).await;
        self.time.update(time);
    }

    pub async fn get_time(&self) -> impl Stream<Item = sdk::Time> {
        self.time.stream()
    }

    pub async fn reset_state(&self) {
        self.time.reset();
        self.runs.reset().await;
    }

    pub async fn update_parameters(&self, parameters: ContestParameters) {
        let mut file = self.contest.write().await;

        *file = parameters;
    }

    pub async fn update_site_configuration(&self, sites: SiteConfiguration) {
        *self.sedes.write().await = Some(sites);
    }

    pub async fn get_site_configuration(&self) -> Option<SiteConfiguration> {
        self.sedes.read().await.clone()
    }

    pub async fn get_secret(&self) -> Option<String> {
        self.secret.read().await.clone()
    }

    pub async fn set_secret(&self, secret: &str) {
        *self.secret.write().await = Some(secret.to_string())
    }

    #[instrument(skip_all)]
    pub async fn get_runs_unmasked(
        &self,
    ) -> impl Stream<Item = impl Future<Output = Vec<sdk::Run>>> {
        self.get_runs_for_site().await
    }

    #[instrument(skip_all)]
    async fn get_runs_for_site(&self) -> impl Stream<Item = impl Future<Output = Vec<sdk::Run>>> {
        self.runs.stream().await.map(async |r| {
            debug!(?r, "batch of runs");
            let sites = self.sedes.read().await;

            r.into_iter()
                .filter(|r| {
                    sites
                        .as_ref()
                        .is_none_or(|s| s.base.team_belongs(&r.team_login))
                })
                .collect_vec()
        })
    }

    #[instrument(skip_all)]
    pub async fn get_runs_masked(&self) -> impl Stream<Item = impl Future<Output = Vec<sdk::Run>>> {
        self.get_runs_for_site().await.map(async |r| {
            let batch = r.await;
            let frozen_time = self.contest.read().await.score_freeze_time;

            batch
                .into_iter()
                .map(|mut run| {
                    if run.time_in_seconds >= frozen_time {
                        run.mask_answer()
                    }
                    run
                })
                .collect_vec()
        })
    }
}
