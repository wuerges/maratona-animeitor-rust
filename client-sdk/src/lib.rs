mod config;
pub mod legacy;
mod request;
mod websocket_stream;

pub use config::SdkConfig;

use data::configdata::ConfigContest;
use data::event::{Envelope, PublicConfig, PublicContestState, PublicTimer, Run, RunsData};
use data::{ContestFile, RunTuple, TimerData};
use futures::{
    channel::mpsc::{self, UnboundedReceiver},
    SinkExt, StreamExt,
};
use gloo_timers::future::TimeoutFuture;
use log::warn;
use wasm_bindgen_futures::spawn_local;

use request::{create_request, create_request_with_bearer};
use websocket_stream::create_websocket_stream;

/// The event and contest a client is showing, from the UI path.
#[derive(PartialEq, Eq, Clone, Default)]
pub struct EventContest {
    pub event: String,
    pub contest: String,
}

fn url(config: &SdkConfig, ec: &EventContest, path: &str) -> String {
    format!(
        "{}/events/{}/contests/{}/{}",
        config.api_prefix, ec.event, ec.contest, path
    )
}

fn ws_url(config: &SdkConfig, ec: &EventContest, path: &str) -> String {
    format!(
        "{}/events/{}/contests/{}/{}",
        config.ws_prefix, ec.event, ec.contest, path
    )
}

/// Fetches an enveloped resource; retries forever while the server answers
/// without `data` (e.g. `not_found`).
async fn enveloped<T: for<'a> serde::Deserialize<'a> + serde::Serialize + Clone>(url: &str) -> T {
    loop {
        let envelope: Envelope<T> = create_request(url).await;
        match envelope.data {
            Some(data) => return data,
            None => {
                warn!("response without data, retrying: {url}");
                TimeoutFuture::new(5_000).await;
            }
        }
    }
}

/// Lists the active events (landing page).
pub async fn create_events(config: &SdkConfig) -> Vec<String> {
    enveloped(&format!("{}/events", config.api_prefix)).await
}

/// The public state of the contest, converted to the legacy client shape.
pub async fn create_contest(config: &SdkConfig, ec: EventContest) -> ContestFile {
    let state: PublicContestState = enveloped(&url(config, &ec, "contest")).await;
    legacy::to_contest_file(state)
}

/// The raw public config of the contest (with photo/sound formats).
pub async fn create_public_config(config: &SdkConfig, ec: EventContest) -> PublicConfig {
    enveloped(&url(config, &ec, "config")).await
}

/// The public config converted to the legacy client shape.
pub fn to_legacy_config(config: PublicConfig) -> ConfigContest {
    legacy::to_config_contest(config)
}

/// The live run stream of the contest, converted to the legacy client shape.
pub fn create_runs(config: &SdkConfig, ec: EventContest) -> UnboundedReceiver<RunTuple> {
    let runs = create_websocket_stream::<Run>(&ws_url(config, &ec, "runs_ws"));
    let (mut tx, rx) = mpsc::unbounded::<RunTuple>();

    spawn_local(async move {
        let mut runs = runs;
        let mut order = 0;
        while let Some(run) = runs.next().await {
            let tuple = legacy::to_run_tuple(&run, order);
            order += 1;
            if tx.send(tuple).await.is_err() {
                break;
            }
        }
    });

    rx
}

/// The secret runs of a site, unlocked by its key (Bearer header).
pub async fn create_secret_runs(config: &SdkConfig, key: String, ec: EventContest) -> RunsData {
    let url = url(config, &ec, "runs_secret");
    loop {
        let envelope: Envelope<RunsData> = create_request_with_bearer(&url, &key).await;
        match envelope.data {
            Some(data) => return data,
            None => {
                warn!("response without data, retrying: {url}");
                TimeoutFuture::new(5_000).await;
            }
        }
    }
}

pub fn remote_control_url(config: &SdkConfig, ec: &EventContest, key: &str) -> String {
    format!(
        "{}/events/{}/contests/{}/remote_control/{}",
        config.ws_prefix, ec.event, ec.contest, key
    )
}

/// The timer stream of an event, converted to the legacy client shape.
pub fn create_timer_stream(config: &SdkConfig, ec: EventContest) -> UnboundedReceiver<TimerData> {
    let timers = create_websocket_stream::<PublicTimer>(&format!(
        "{}/events/{}/timer",
        config.ws_prefix, ec.event
    ));
    let (mut tx, rx) = mpsc::unbounded::<TimerData>();

    spawn_local(async move {
        let mut timers = timers;
        while let Some(timer) = timers.next().await {
            let data = legacy::to_timer_data(timer);
            if tx.send(data).await.is_err() {
                break;
            }
        }
    });

    rx
}

pub fn team_photo_location(config: &SdkConfig, team_login: &str) -> String {
    match &config.photo_url_format {
        Some(format) => format.replace("{team_login}", team_login),
        None => format!("{}/{}.webp", config.photo_prefix, team_login),
    }
}

pub fn team_sound_location(config: &SdkConfig, team_login: &str) -> String {
    match &config.sound_url_format {
        Some(format) => format.replace("{team_login}", team_login),
        None => format!("{}/{}.mp3", config.sound_prefix, team_login),
    }
}

/// Builds a photo URL from a contest's own format, falling back to the SDK
/// defaults (the migrated behavior: media comes from the contest config).
pub fn team_photo_location_with(
    config: &SdkConfig,
    format: Option<&str>,
    team_login: &str,
) -> String {
    match format {
        Some(format) => format.replace("{team_login}", team_login),
        None => team_photo_location(config, team_login),
    }
}

/// Builds a sound URL from a contest's own format, falling back to the SDK
/// defaults (the migrated behavior: media comes from the contest config).
pub fn team_sound_location_with(
    config: &SdkConfig,
    format: Option<&str>,
    team_login: &str,
) -> String {
    match format {
        Some(format) => format.replace("{team_login}", team_login),
        None => team_sound_location(config, team_login),
    }
}
