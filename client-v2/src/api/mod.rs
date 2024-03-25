use data::{configdata::ConfigContest, ContestFile, RunTuple, TimerData};
use futures::{channel::mpsc::UnboundedReceiver, StreamExt};

use leptos::{
    logging::{log, warn},
    *,
};

use crate::net::{request_signal::create_request, websocket_stream::create_websocket_stream};

const DEFAULT_URL: &'static str = "http://0.0.0.0/api";

#[cfg(not(test))]
fn window_origin() -> Option<String> {
    web_sys::window().map(|w| w.origin())
}

#[cfg(test)]
fn window_origin() -> Option<String> {
    None
}

fn url_prefix() -> String {
    match option_env!("URL_PREFIX") {
        Some(prefix) => prefix.to_string(),
        None => match window_origin() {
            Some(mut origin) => {
                origin.push_str("/api");
                origin
            }
            None => {
                warn!("could not guess an origin, using default: {}", DEFAULT_URL);
                DEFAULT_URL.to_string()
            }
        },
    }
}

fn ws_url_prefix() -> String {
    let mut prefix = url_prefix();

    let https = "https://";
    let wss = "wss://";

    let http = "http://";
    let ws = "ws://";
    if prefix.starts_with(https) {
        prefix.replace_range(..https.len(), wss)
    } else if prefix.starts_with(http) {
        prefix.replace_range(..http.len(), ws)
    }
    prefix
}

fn url(path: &str) -> String {
    let mut prefix = url_prefix();
    prefix.push_str("/");
    prefix.push_str(path);
    prefix
}

fn ws(path: &str) -> String {
    let mut prefix = ws_url_prefix();
    prefix.push_str("/");
    prefix.push_str(path);
    prefix
}

pub async fn create_contest() -> ContestFile {
    let contest_message = create_request(&url("contest")).await;

    contest_message
}

pub async fn create_config() -> ConfigContest {
    let config_message = create_request(&url("config")).await;

    config_message
}

pub fn create_runs() -> UnboundedReceiver<RunTuple> {
    create_websocket_stream::<RunTuple>(&ws("allruns_ws"))
}

pub async fn create_secret_runs(secret: String) -> data::RunsFile {
    let mut url = url_prefix();
    url.push_str("/allruns_secret?secret=");
    url.push_str(secret.as_str());

    create_request(&url).await
}

pub fn create_timer() -> ReadSignal<(TimerData, TimerData)> {
    let mut timer_stream = create_websocket_stream::<TimerData>(&ws("timer"));

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
