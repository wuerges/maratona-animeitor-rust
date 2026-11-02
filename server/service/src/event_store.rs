//! Database-backed event service. Domain rules and live channels are backend independent.
mod engine;
use crate::database::{Database, DatabaseError};
use crate::{membroadcast, remote_control::RemoteConnection};
use data::incremental::*;
pub use engine::{
    Answer, ContestConfig, Envelope, ErrorEntry, EventState, PublicConfig, PublicContestState,
    PublicSiteView, PublicTimer, Run, RunsData, SiteConfig, StoreError, TeamInfo,
    deployment_site_key, from_legacy_contest_state,
};
use regex::RegexSet;
use sentry::SentryFutureExt;
use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::{Mutex, broadcast};
use tracing::Instrument;

#[derive(Clone)]
pub struct EventStore {
    database: Arc<dyn Database>,
    live: engine::Engine,
    salt: Arc<String>,
    // Serialize storage hydration and mutations; public reads use the engine RwLock.
    gate: Arc<Mutex<()>>,
    initialized: Arc<AtomicBool>,
    websockets: Arc<tokio::sync::Semaphore>,
    mutations: Arc<tokio::sync::Semaphore>,
}
impl EventStore {
    pub fn new(database: Arc<dyn Database>, revelation_salt: String) -> Self {
        assert!(
            !revelation_salt.is_empty(),
            "revelation salt must not be empty"
        );
        Self {
            database,
            live: engine::Engine::with_revelation_salt(revelation_salt.clone()),
            salt: Arc::new(revelation_salt),
            gate: Arc::new(Mutex::new(())),
            initialized: Arc::new(AtomicBool::new(false)),
            websockets: Arc::new(tokio::sync::Semaphore::new(2048)),
            mutations: Arc::new(tokio::sync::Semaphore::new(32)),
        }
    }

    /// Hydrate once. One process owns storage; mutations publish committed snapshots.
    /// Public traffic never reopens storage or takes the mutation gate after startup.
    pub async fn initialize(&self) -> Result<(), StoreError> {
        if self.initialized.load(Ordering::Acquire) {
            return Ok(());
        }
        let _guard = self.gate.lock().await;
        if !self.initialized.load(Ordering::Acquire) {
            for name in crate::database::observe("list", self.database.list()).await? {
                self.refresh(&name).await?;
            }
            self.initialized.store(true, Ordering::Release);
        }
        Ok(())
    }

    pub fn websocket_permit(&self) -> Result<tokio::sync::OwnedSemaphorePermit, StoreError> {
        self.websockets
            .clone()
            .try_acquire_owned()
            .map_err(|_| StoreError::LimitExceeded)
    }

    async fn refresh(&self, name: &str) -> Result<(), StoreError> {
        match crate::database::observe("read", self.database.read(name)).await {
            Ok(snapshot) => self
                .live
                .install(name, snapshot)
                .await
                .map_err(|err| match err {
                    StoreError::Storage(_) => err,
                    _ => StoreError::Storage(DatabaseError::Corrupt(
                        "invalid event configuration".into(),
                    )),
                }),
            Err(err) => {
                // Do not leave streams serving uncertain state after storage failure.
                self.initialized.store(false, Ordering::Release);
                self.live.install(name, None).await?;
                Err(err.into())
            }
        }
    }

    async fn mutate<T: Send + 'static>(
        &self,
        name: String,
        operation: impl FnOnce(
            engine::Engine,
        ) -> Pin<Box<dyn Future<Output = Result<T, StoreError>> + Send>>
        + Send
        + 'static,
    ) -> Result<T, StoreError> {
        let permit = self
            .mutations
            .clone()
            .try_acquire_owned()
            .map_err(|_| StoreError::LimitExceeded)?;
        let this = self.clone();
        // Detaching the caller must not cancel persistence between commit and publish.
        let hub = Arc::new(sentry::Hub::new_from_top(sentry::Hub::current()));
        tokio::spawn(
            async move {
                let _permit = permit;
                let result = async {
                    let _guard = this.gate.lock().await;
                    let before =
                        crate::database::observe("read", this.database.read(&name)).await?;
                    let stage = engine::Engine::staging((*this.salt).clone());
                    stage
                        .install(&name, before.clone())
                        .await
                        .map_err(|error| {
                            StoreError::Storage(DatabaseError::Corrupt(format!(
                                "invalid stored event: {error}"
                            )))
                        })?;
                    let result = operation(stage.clone()).await?;
                    let after = stage.snapshot(&name).await;
                    if before != after {
                        let persisted = match (&before, &after) {
                            (None, Some(event)) => {
                                crate::database::observe(
                                    "create",
                                    this.database.create(event.clone()),
                                )
                                .await
                            }
                            (Some(_), Some(event)) => {
                                crate::database::observe(
                                    "replace",
                                    this.database.replace(event.clone()),
                                )
                                .await
                            }
                            (Some(_), None) => {
                                crate::database::observe("delete", this.database.delete(&name))
                                    .await
                                    .map(|_| ())
                            }
                            (None, None) => Ok(()),
                        };
                        if let Err(err) = persisted {
                            // A commit may have succeeded despite a reported I/O failure.
                            // Close old streams and recover from the authoritative database.
                            this.live.install(&name, None).await?;
                            if this.refresh(&name).await.is_err() {
                                this.initialized.store(false, Ordering::Release);
                            }
                            return Err(err.into());
                        }
                    }
                    this.live.install(&name, after).await?;
                    Ok(result)
                }
                .await;
                result.map_err(|error| match error {
                    StoreError::Storage(error) => {
                        crate::database::report_error(&error);
                        StoreError::ReportedStorage(error)
                    }
                    error => error,
                })
            }
            .in_current_span()
            .bind_hub(hub),
        )
        .await
        .map_err(|_| {
            StoreError::Storage(DatabaseError::Unavailable("database task stopped".into()))
        })?
    }
    pub async fn create_event(
        &self,
        event_name: &str,
        state: EventState,
    ) -> Result<(), StoreError> {
        let event_name = event_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move { stage.create_event(&event_name, state).await })
        })
        .await
    }
    pub async fn put_event(&self, event_name: &str, state: EventState) -> Result<(), StoreError> {
        let event_name = event_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move { stage.put_event(&event_name, state).await })
        })
        .await
    }
    pub async fn get_event(&self, event_name: &str) -> Result<Option<EventState>, StoreError> {
        let _guard = self.gate.lock().await;
        self.refresh(event_name).await?;
        Ok(self.live.get_event(event_name).await)
    }
    pub async fn delete_event(&self, event_name: &str) -> Result<bool, StoreError> {
        let event_name = event_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move { Ok(stage.delete_event(&event_name).await) })
        })
        .await
    }
    pub async fn list_events(&self) -> Result<Vec<String>, StoreError> {
        self.initialize().await?;
        Ok(self.live.list_events().await)
    }
    pub async fn is_started(&self, event_name: &str) -> Result<Option<bool>, StoreError> {
        self.initialize().await?;
        Ok(self.live.is_started(event_name).await)
    }
    pub async fn patch_time(
        &self,
        event_name: &str,
        seconds: i64,
    ) -> Result<Option<i64>, StoreError> {
        let event_name = event_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move { Ok(stage.patch_time(&event_name, seconds).await) })
        })
        .await
    }
    pub async fn set_event_salt(
        &self,
        event_name: &str,
        salt: Option<String>,
    ) -> Result<String, StoreError> {
        let event_name = event_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move { stage.set_event_salt(&event_name, salt).await })
        })
        .await
    }
    pub async fn add_runs(
        &self,
        event_name: &str,
        runs: Vec<Run>,
    ) -> Result<(usize, usize, Vec<Run>), StoreError> {
        let event_name = event_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move { stage.add_runs(&event_name, runs).await })
        })
        .await
    }
    pub async fn delete_run(&self, event_name: &str, run_id: i64) -> Result<bool, StoreError> {
        let event_name = event_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move { Ok(stage.delete_run(&event_name, run_id).await) })
        })
        .await
    }

    pub async fn clear_runs(&self, event_name: &str) -> Result<bool, StoreError> {
        let event_name = event_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move { Ok(stage.clear_runs(&event_name).await) })
        })
        .await
    }
    pub async fn create_contest(
        &self,
        event_name: &str,
        contest_name: &str,
        config: ContestConfig,
    ) -> Result<(), StoreError> {
        let event_name = event_name.to_owned();
        let contest_name = contest_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move {
                stage
                    .create_contest(&event_name, &contest_name, config)
                    .await
            })
        })
        .await
    }
    pub async fn get_contest(
        &self,
        event_name: &str,
        contest_name: &str,
    ) -> Result<Option<ContestConfig>, StoreError> {
        let _guard = self.gate.lock().await;
        self.refresh(event_name).await?;
        Ok(self.live.get_contest(event_name, contest_name).await)
    }
    pub async fn list_contests(
        &self,
        event_name: &str,
    ) -> Result<Option<Vec<ContestConfig>>, StoreError> {
        self.initialize().await?;
        Ok(self.live.list_contests(event_name).await)
    }
    pub async fn put_contest(
        &self,
        event_name: &str,
        contest_name: &str,
        config: ContestConfig,
    ) -> Result<(), StoreError> {
        let event_name = event_name.to_owned();
        let contest_name = contest_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move { stage.put_contest(&event_name, &contest_name, config).await })
        })
        .await
    }
    pub async fn delete_contest(
        &self,
        event_name: &str,
        contest_name: &str,
    ) -> Result<bool, StoreError> {
        let event_name = event_name.to_owned();
        let contest_name = contest_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move { Ok(stage.delete_contest(&event_name, &contest_name).await) })
        })
        .await
    }
    pub async fn set_contest_salt(
        &self,
        event_name: &str,
        contest_name: &str,
        salt: Option<String>,
    ) -> Result<String, StoreError> {
        let event_name = event_name.to_owned();
        let contest_name = contest_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move {
                stage
                    .set_contest_salt(&event_name, &contest_name, salt)
                    .await
            })
        })
        .await
    }
    pub async fn create_site(
        &self,
        event_name: &str,
        contest_name: &str,
        site_name: &str,
        config: SiteConfig,
    ) -> Result<(), StoreError> {
        let event_name = event_name.to_owned();
        let contest_name = contest_name.to_owned();
        let site_name = site_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move {
                stage
                    .create_site(&event_name, &contest_name, &site_name, config)
                    .await
            })
        })
        .await
    }
    pub async fn get_site(
        &self,
        event_name: &str,
        contest_name: &str,
        site_name: &str,
    ) -> Result<Option<SiteConfig>, StoreError> {
        let _guard = self.gate.lock().await;
        self.refresh(event_name).await?;
        Ok(self
            .live
            .get_site(event_name, contest_name, site_name)
            .await)
    }
    pub async fn list_sites(
        &self,
        event_name: &str,
        contest_name: &str,
    ) -> Result<Option<Vec<SiteConfig>>, StoreError> {
        let _guard = self.gate.lock().await;
        self.refresh(event_name).await?;
        Ok(self.live.list_sites(event_name, contest_name).await)
    }
    pub async fn put_site(
        &self,
        event_name: &str,
        contest_name: &str,
        site_name: &str,
        config: SiteConfig,
    ) -> Result<(), StoreError> {
        let event_name = event_name.to_owned();
        let contest_name = contest_name.to_owned();
        let site_name = site_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move {
                stage
                    .put_site(&event_name, &contest_name, &site_name, config)
                    .await
            })
        })
        .await
    }
    pub async fn delete_site(
        &self,
        event_name: &str,
        contest_name: &str,
        site_name: &str,
    ) -> Result<bool, StoreError> {
        let event_name = event_name.to_owned();
        let contest_name = contest_name.to_owned();
        let site_name = site_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move {
                Ok(stage
                    .delete_site(&event_name, &contest_name, &site_name)
                    .await)
            })
        })
        .await
    }
    pub async fn set_site_salt(
        &self,
        event_name: &str,
        contest_name: &str,
        site_name: &str,
        salt: Option<String>,
    ) -> Result<String, StoreError> {
        let event_name = event_name.to_owned();
        let contest_name = contest_name.to_owned();
        let site_name = site_name.to_owned();
        self.mutate(event_name.clone(), move |stage| {
            Box::pin(async move {
                stage
                    .set_site_salt(&event_name, &contest_name, &site_name, salt)
                    .await
            })
        })
        .await
    }
    pub async fn revelation_urls(
        &self,
        event_name: &str,
        public_url: &url::Url,
    ) -> Result<Option<Vec<data::event::RevelationUrl>>, StoreError> {
        let _guard = self.gate.lock().await;
        self.refresh(event_name).await?;
        Ok(self.live.revelation_urls(event_name, public_url).await)
    }
    pub async fn site_by_key(
        &self,
        event_name: &str,
        contest_name: &str,
        key: &str,
    ) -> Result<Option<(String, SiteConfig)>, StoreError> {
        self.initialize().await?;
        Ok(self.live.site_by_key(event_name, contest_name, key).await)
    }
    pub async fn public_state(
        &self,
        event_name: &str,
        contest_name: &str,
    ) -> Result<Option<PublicContestState>, StoreError> {
        self.initialize().await?;
        Ok(self.live.public_state(event_name, contest_name).await)
    }
    pub async fn public_config(
        &self,
        event_name: &str,
        contest_name: &str,
    ) -> Result<Option<PublicConfig>, StoreError> {
        self.initialize().await?;
        Ok(self.live.public_config(event_name, contest_name).await)
    }
    pub async fn contest_runs(
        &self,
        event_name: &str,
        contest_name: &str,
    ) -> Result<Option<Vec<Run>>, StoreError> {
        let _guard = self.gate.lock().await;
        self.refresh(event_name).await?;
        Ok(self.live.contest_runs(event_name, contest_name).await)
    }
    pub async fn site_runs(
        &self,
        event_name: &str,
        contest_name: &str,
        site_name: &str,
    ) -> Result<Option<Vec<Run>>, StoreError> {
        self.initialize().await?;
        Ok(self
            .live
            .site_runs(event_name, contest_name, site_name)
            .await)
    }
    pub async fn contest_codes(
        &self,
        event_name: &str,
        contest_name: &str,
    ) -> Result<Option<RegexSet>, StoreError> {
        self.initialize().await?;
        Ok(self.live.contest_codes(event_name, contest_name).await)
    }
    pub async fn subscribe_runs(
        &self,
        event_name: &str,
    ) -> Result<Option<membroadcast::Receiver<Run>>, StoreError> {
        self.initialize().await?;
        Ok(self.live.subscribe_runs(event_name).await)
    }
    pub async fn subscribe_timer(
        &self,
        event_name: &str,
    ) -> Result<Option<broadcast::Receiver<PublicTimer>>, StoreError> {
        self.initialize().await?;
        Ok(self.live.subscribe_timer(event_name).await)
    }
    pub async fn timer_subscription(
        &self,
        name: &str,
    ) -> Result<Option<(PublicTimer, broadcast::Receiver<PublicTimer>)>, StoreError> {
        self.initialize().await?;
        Ok(self.live.timer_subscription(name).await)
    }

    pub async fn current_timer(&self, event_name: &str) -> Result<Option<PublicTimer>, StoreError> {
        self.initialize().await?;
        Ok(self.live.current_timer(event_name).await)
    }
    pub async fn create_remote_session(
        &self,
        event: &str,
        contest: &str,
    ) -> Result<data::remote_control::RemoteSessionInfo, StoreError> {
        self.create_remote_session_with_ttl(
            event,
            contest,
            data::remote_control::DEFAULT_REMOTE_TTL_SECONDS,
        )
        .await
    }
    pub async fn create_remote_session_with_ttl(
        &self,
        event: &str,
        contest: &str,
        ttl: u64,
    ) -> Result<data::remote_control::RemoteSessionInfo, StoreError> {
        if !(1..=data::remote_control::MAX_REMOTE_TTL_SECONDS).contains(&ttl) {
            return Err(StoreError::InvalidValue(
                "expires_in_seconds must be between 1 and 2592000".into(),
            ));
        }
        self.initialize().await?;
        let event = event.to_owned();
        let contest = contest.to_owned();
        let live = self.live.clone();
        self.mutate(event.clone(), move |stage| {
            Box::pin(async move {
                live.check_remote_capacity(&event, &contest).await?;
                stage.create_remote_session(&event, &contest, ttl).await
            })
        })
        .await
    }
    pub async fn revoke_remote_session(
        &self,
        event: &str,
        contest: &str,
        key: &str,
    ) -> Result<bool, StoreError> {
        self.initialize().await?;
        let event = event.to_owned();
        let contest = contest.to_owned();
        let key = key.to_owned();
        self.mutate(event.clone(), move |stage| {
            Box::pin(async move { Ok(stage.revoke_remote_session(&event, &contest, &key).await) })
        })
        .await
    }
    pub async fn remote_control_sender(
        &self,
        event_name: &str,
        contest_name: &str,
        key: &str,
    ) -> Result<Option<RemoteConnection>, StoreError> {
        self.initialize().await?;
        self.live
            .remote_control_sender(event_name, contest_name, key)
            .await
    }
    pub async fn has_event(&self, event_name: &str) -> Result<bool, StoreError> {
        let _guard = self.gate.lock().await;
        self.refresh(event_name).await?;
        Ok(self.live.has_event(event_name).await)
    }
    pub async fn patch_event(
        &self,
        name: &str,
        patch: EventPatch,
        keep_runs: bool,
    ) -> Result<EventState, StoreError> {
        let name = name.to_owned();
        self.mutate(name.clone(), move |stage| {
            Box::pin(async move { stage.patch_event(&name, patch, keep_runs).await })
        })
        .await
    }
    pub async fn patch_contest(
        &self,
        event: &str,
        contest: &str,
        patch: ContestPatch,
    ) -> Result<ContestConfig, StoreError> {
        let event = event.to_owned();
        let contest = contest.to_owned();
        self.mutate(event.clone(), move |stage| {
            Box::pin(async move { stage.patch_contest(&event, &contest, patch).await })
        })
        .await
    }
    pub async fn patch_site(
        &self,
        event: &str,
        contest: &str,
        site: &str,
        patch: SitePatch,
    ) -> Result<SiteConfig, StoreError> {
        let event = event.to_owned();
        let contest = contest.to_owned();
        let site = site.to_owned();
        self.mutate(event.clone(), move |stage| {
            Box::pin(async move { stage.patch_site(&event, &contest, &site, patch).await })
        })
        .await
    }
    pub async fn add_team(&self, event: &str, team: NewTeam) -> Result<TeamInfo, StoreError> {
        let event = event.to_owned();
        self.mutate(event.clone(), move |stage| {
            Box::pin(async move { stage.add_team(&event, team).await })
        })
        .await
    }
    pub async fn get_team(&self, event: &str, login: &str) -> Result<TeamInfo, StoreError> {
        let _guard = self.gate.lock().await;
        self.refresh(event).await?;
        self.live.get_team(event, login).await
    }
    pub async fn patch_team(
        &self,
        event: &str,
        login: &str,
        patch: TeamPatch,
    ) -> Result<TeamInfo, StoreError> {
        let event = event.to_owned();
        let login = login.to_owned();
        self.mutate(event.clone(), move |stage| {
            Box::pin(async move { stage.patch_team(&event, &login, patch).await })
        })
        .await
    }
    pub async fn remove_team(
        &self,
        event: &str,
        login: &str,
        keep_runs: bool,
    ) -> Result<(), StoreError> {
        let event = event.to_owned();
        let login = login.to_owned();
        self.mutate(event.clone(), move |stage| {
            Box::pin(async move { stage.remove_team(&event, &login, keep_runs).await })
        })
        .await
    }
    pub async fn add_problem(
        &self,
        event: &str,
        body: NewProblem,
    ) -> Result<Vec<String>, StoreError> {
        let event = event.to_owned();
        self.mutate(event.clone(), move |stage| {
            Box::pin(async move { stage.add_problem(&event, body).await })
        })
        .await
    }
    pub async fn remove_problem(&self, event: &str, problem: &str) -> Result<(), StoreError> {
        let event = event.to_owned();
        let problem = problem.to_owned();
        self.mutate(event.clone(), move |stage| {
            Box::pin(async move { stage.remove_problem(&event, &problem).await })
        })
        .await
    }
    pub async fn patch_contest_codes(
        &self,
        event: &str,
        contest: &str,
        patch: CodesPatch,
    ) -> Result<ContestConfig, StoreError> {
        let event = event.to_owned();
        let contest = contest.to_owned();
        self.mutate(event.clone(), move |stage| {
            Box::pin(async move { stage.patch_contest_codes(&event, &contest, patch).await })
        })
        .await
    }
    pub async fn patch_site_codes(
        &self,
        event: &str,
        contest: &str,
        site: &str,
        patch: CodesPatch,
    ) -> Result<SiteConfig, StoreError> {
        let event = event.to_owned();
        let contest = contest.to_owned();
        let site = site.to_owned();
        self.mutate(event.clone(), move |stage| {
            Box::pin(async move { stage.patch_site_codes(&event, &contest, &site, patch).await })
        })
        .await
    }
}
