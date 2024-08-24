use data::{configdata::ConfigContest, ContestFile, RunTuple, TimerData};
use futures::{channel::mpsc::UnboundedReceiver, StreamExt};

use leptos::{logging::warn, *};
use leptos_router::Params;

use crate::net::{request_signal::create_request, websocket_stream::create_websocket_stream};

const DEFAULT_URL: &'static str = "http://0.0.0.0";

#[derive(Params, PartialEq, Eq, Clone, Default)]
pub struct ContestQuery {
    pub contest: Option<String>,
}

#[cfg(not(test))]
fn window_origin() -> Option<String> {
    web_sys::window().map(|w| w.origin())
}

#[cfg(test)]
fn window_origin() -> Option<String> {
    None
}

fn guess_prefix() -> String {
    match window_origin() {
        Some(origin) => origin,
        None => {
            warn!("could not guess an origin, using default: {}", DEFAULT_URL);
            DEFAULT_URL.to_string()
        }
    }
}

fn url_prefix() -> String {
    let mut prefix = match option_env!("URL_PREFIX") {
        Some(prefix) => prefix.to_string(),
        None => format!("{}/api", guess_prefix()),
    };

    if window_protocol_is_https() {
        if prefix.starts_with("http:") {
            prefix.replace_range(.."http:".len(), "https:")
        }
    }

    prefix
}

fn window_protocol_is_https() -> bool {
    web_sys::window().is_some_and(|w| {
        w.location()
            .protocol()
            .is_ok_and(|p| p.starts_with("https"))
    })
}

fn ws_url_prefix() -> String {
    let mut prefix = url_prefix();

    if prefix.starts_with("https:") {
        prefix.replace_range(.."https:".len(), "wss:")
    }
    if prefix.starts_with("http:") {
        prefix.replace_range(.."http:".len(), "ws:")
    }

    prefix
}

fn push_contest_query(url: &mut String, query: ContestQuery) {
    if let Some(contest) = query.contest {
        url.push_str(&format!("?contest={contest}"));
    }
}

fn url(path: &str, query: ContestQuery) -> String {
    let mut prefix = url_prefix();
    prefix.push_str("/");
    prefix.push_str(path);
    push_contest_query(&mut prefix, query);
    prefix
}

fn contest_query_ws(path: &str, query: ContestQuery) -> String {
    let mut prefix = ws_url_prefix();
    prefix.push_str("/");
    prefix.push_str(path);
    push_contest_query(&mut prefix, query);
    prefix
}

pub async fn create_contest(query: ContestQuery) -> ContestFile {
    let contest_message = create_request(&url("contest", query)).await;

    contest_message
}

pub async fn create_config(query: ContestQuery) -> ConfigContest {
    let config_message = create_request(&url("config", query)).await;

    config_message
}

pub fn create_runs(query: ContestQuery) -> UnboundedReceiver<RunTuple> {
    create_websocket_stream::<RunTuple>(&contest_query_ws("allruns_ws", query))
}

pub fn remote_control_url(key: &str) -> String {
    let mut prefix = ws_url_prefix();
    prefix.push_str("/remote_control/");
    prefix.push_str(key);
    prefix
}

pub async fn create_secret_runs(secret: String, contest: Option<String>) -> data::RunsFile {
    let mut url = url_prefix();
    url.push_str("/allruns_secret?secret=");
    url.push_str(secret.as_str());

    if let Some(contest) = contest {
        url.push_str("&contest=");
        url.push_str(contest.as_str());
    }

    create_request(&url).await
}

pub fn create_timer() -> ReadSignal<(TimerData, TimerData)> {
    let mut timer_stream = create_websocket_stream::<TimerData>(&contest_query_ws(
        "timer",
        ContestQuery { contest: None },
    ));

    let (timer, set_timer) = create_signal((TimerData::fake(), data::TimerData::new(0, 1)));

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

fn photos_prefix() -> String {
    match option_env!("PHOTO_PREFIX") {
        Some(prefix) => prefix.to_string(),
        None => format!("{}/photos", guess_prefix()),
    }
}

fn sound_prefix() -> String {
    match option_env!("SOUND_PREFIX") {
        Some(prefix) => prefix.to_string(),
        None => format!("{}/sounds", guess_prefix()),
    }
}

pub fn team_photo_location(team_login: &str) -> String {
    format!("{}/{}.webp", photos_prefix(), team_login)
}

pub fn team_sound_location(team_login: &str) -> String {
    format!("{}/{}.mp3", sound_prefix(), team_login)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_url_prefix() {
        let url_prefix = url_prefix();
        assert_eq!(url_prefix, "http://0.0.0.0/api");
    }

    #[test]
    fn check_ws_url_prefix() {
        let url_prefix = ws_url_prefix();
        assert_eq!(url_prefix, "ws://0.0.0.0/api");
    }
}
