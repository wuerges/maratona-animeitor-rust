use std::{future::Future, sync::OnceLock};

use client_model::{poll_runs, ContestProvider, Options, TimerDataExt};
use data::TimerData;
use futures::{channel::mpsc::UnboundedReceiver, StreamExt};
use leptos::{prelude::*, task::spawn_local};
use leptos_router::params::{Params, ParamsError, ParamsMap};

#[derive(PartialEq, Eq, Clone, Default)]
pub struct ContestQuery {
    pub contest: Option<String>,
}

impl Params for ContestQuery {
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        let contest = map.get("contest");
        Ok(ContestQuery { contest })
    }
}

impl From<ContestQuery> for client_sdk::ContestQuery {
    fn from(q: ContestQuery) -> Self {
        client_sdk::ContestQuery { contest: q.contest }
    }
}

static CONFIG: OnceLock<client_sdk::SdkConfig> = OnceLock::new();

pub fn init_config(config: client_sdk::SdkConfig) {
    let _ = CONFIG.set(config);
}

fn config() -> &'static client_sdk::SdkConfig {
    CONFIG.get().expect("sdk config not initialized")
}

pub fn remote_control_url(key: &str) -> String {
    client_sdk::remote_control_url(config(), key)
}

pub fn team_photo_location(team_login: &str) -> String {
    client_sdk::team_photo_location(config(), team_login)
}

pub fn team_sound_location(team_login: &str) -> String {
    client_sdk::team_sound_location(config(), team_login)
}

pub async fn create_secret_runs(secret: String, contest: Option<String>) -> data::RunsFile {
    client_sdk::create_secret_runs(config(), secret, contest).await
}

fn create_runs(query: client_sdk::ContestQuery) -> UnboundedReceiver<data::RunTuple> {
    client_sdk::create_runs(config(), query)
}

pub fn create_timer() -> ReadSignal<(TimerData, TimerData)> {
    let mut timer_stream = client_sdk::create_timer_stream(config());

    let (timer, set_timer) = signal((TimerData::fake(), data::TimerData::new(0, 1)));

    spawn_local(async move {
        loop {
            let next = timer_stream.next().await;
            if let Some(next) = next {
                set_timer.update(|(new, old)| {
                    *old = *new;
                    *new = next;
                });
            }
        }
    });

    timer
}

pub fn provide_contest(query: ContestQuery) -> impl Future<Output = ContestProvider> {
    let query = client_sdk::ContestQuery::from(query);
    async move {
        let provider = client_model::provide_contest(
            client_sdk::create_contest(config(), query.clone()),
            client_sdk::create_config(config(), query.clone()),
        )
        .await;

        spawn_local(poll_runs(
            provider.starting_contest.clone(),
            create_runs(query),
            provider.new_contest_signal.clone(),
            provider.runs_panel_item_manager.clone(),
            Options::default(),
            || gloo_timers::future::TimeoutFuture::new(1_000),
        ));

        provider
    }
}
