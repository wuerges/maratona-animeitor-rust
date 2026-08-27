use std::{future::Future, sync::OnceLock, sync::RwLock};

use client_model::{poll_runs, ContestProvider, Options, TimerDataExt};
use data::TimerData;
use data::event::PublicConfig;
use futures::{channel::mpsc::UnboundedReceiver, StreamExt};
use leptos::{prelude::*, task::spawn_local};

/// The event and contest a client is showing, parsed from the URL path
/// (`/animeitor/{event}/{contest}`; the empty contest segment is the default).
pub use client_sdk::EventContest;

static CONFIG: OnceLock<client_sdk::SdkConfig> = OnceLock::new();

pub fn init_config(config: client_sdk::SdkConfig) {
    let _ = CONFIG.set(config);
}

fn config() -> &'static client_sdk::SdkConfig {
    CONFIG.get().expect("sdk config not initialized")
}

/// The public config of the contest being shown; its photo/sound formats
/// override the deploy-level defaults (media comes from the contest config).
static MEDIA: RwLock<Option<PublicConfig>> = RwLock::new(None);

/// Extracts the event/contest from `window.location.pathname`; `None` when
/// the page is not an animeitor path (landing).
pub fn event_contest_from_pathname() -> Option<EventContest> {
    let pathname = web_sys::window()?.location().pathname().ok()?;
    let segments: Vec<&str> = pathname.split('/').filter(|s| !s.is_empty()).collect();
    let pos = segments.iter().position(|s| *s == "animeitor")?;
    let event = segments.get(pos + 1)?.to_string();
    if event.is_empty() {
        return None;
    }
    Some(EventContest {
        event,
        contest: segments.get(pos + 2).unwrap_or(&"").to_string(),
    })
}

pub async fn create_events() -> Vec<String> {
    client_sdk::create_events(config()).await
}

pub fn remote_control_url(ec: &EventContest, key: &str) -> String {
    client_sdk::remote_control_url(config(), ec, key)
}

fn media_formats() -> Option<PublicConfig> {
    MEDIA.read().ok().and_then(|media| media.clone())
}

pub fn team_photo_location(team_login: &str) -> String {
    match media_formats() {
        Some(public) => client_sdk::team_photo_location_with(
            config(),
            public.photo_url_format.as_deref(),
            team_login,
        ),
        None => client_sdk::team_photo_location(config(), team_login),
    }
}

pub fn team_sound_location(team_login: &str) -> String {
    match media_formats() {
        Some(public) => client_sdk::team_sound_location_with(
            config(),
            public.sound_url_format.as_deref(),
            team_login,
        ),
        None => client_sdk::team_sound_location(config(), team_login),
    }
}

pub async fn create_secret_runs(key: String, ec: EventContest) -> data::RunsFile {
    let data = client_sdk::create_secret_runs(config(), key, ec).await;
    client_sdk::legacy::to_runs_file(data)
}

fn create_runs(ec: EventContest) -> UnboundedReceiver<data::RunTuple> {
    client_sdk::create_runs(config(), ec)
}

pub fn create_timer(ec: EventContest) -> ReadSignal<(TimerData, TimerData)> {
    let mut timer_stream = client_sdk::create_timer_stream(config(), ec);

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

pub fn provide_contest(ec: EventContest) -> impl Future<Output = ContestProvider> {
    let ec_for_runs = ec.clone();
    async move {
        let public_config = client_sdk::create_public_config(config(), ec.clone()).await;

        *MEDIA.write().expect("media lock poisoned") = Some(public_config.clone());

        let provider = client_model::provide_contest(
            client_sdk::create_contest(config(), ec.clone()),
            async { client_sdk::to_legacy_config(public_config) },
        )
        .await;

        spawn_local(poll_runs(
            provider.starting_contest.clone(),
            create_runs(ec_for_runs),
            provider.new_contest_signal.clone(),
            provider.runs_panel_item_manager.clone(),
            Options::default(),
            || gloo_timers::future::TimeoutFuture::new(1_000),
        ));

        provider
    }
}
