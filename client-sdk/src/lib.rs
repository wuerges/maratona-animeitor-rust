mod config;
mod request;
mod websocket_stream;

pub use config::SdkConfig;

use data::{configdata::ConfigContest, ContestFile, RunTuple, RunsFile, TimerData};
use futures::channel::mpsc::UnboundedReceiver;

use request::create_request;
use websocket_stream::create_websocket_stream;

#[derive(PartialEq, Eq, Clone, Default)]
pub struct ContestQuery {
    pub contest: Option<String>,
}

fn push_contest_query(url: &mut String, query: &ContestQuery) {
    if let Some(contest) = &query.contest {
        url.push_str(&format!("?contest={contest}"));
    }
}

fn url(config: &SdkConfig, path: &str, query: &ContestQuery) -> String {
    let mut prefix = config.api_prefix.clone();
    prefix.push('/');
    prefix.push_str(path);
    push_contest_query(&mut prefix, query);
    prefix
}

fn contest_query_ws(config: &SdkConfig, path: &str, query: &ContestQuery) -> String {
    let mut prefix = config.ws_prefix.clone();
    prefix.push('/');
    prefix.push_str(path);
    push_contest_query(&mut prefix, query);
    prefix
}

pub async fn create_contest(config: &SdkConfig, query: ContestQuery) -> ContestFile {
    create_request(&url(config, "contest", &query)).await
}

pub async fn create_config(config: &SdkConfig, query: ContestQuery) -> ConfigContest {
    create_request(&url(config, "config", &query)).await
}

pub fn create_runs(config: &SdkConfig, query: ContestQuery) -> UnboundedReceiver<RunTuple> {
    create_websocket_stream::<RunTuple>(&contest_query_ws(config, "allruns_ws", &query))
}

pub fn remote_control_url(config: &SdkConfig, key: &str) -> String {
    let mut prefix = config.ws_prefix.clone();
    prefix.push_str("/remote_control/");
    prefix.push_str(key);
    prefix
}

pub async fn create_secret_runs(
    config: &SdkConfig,
    secret: String,
    contest: Option<String>,
) -> RunsFile {
    let mut url = config.api_prefix.clone();
    url.push_str("/allruns_secret?secret=");
    url.push_str(secret.as_str());

    if let Some(contest) = contest {
        url.push_str("&contest=");
        url.push_str(contest.as_str());
    }

    create_request(&url).await
}

pub fn create_timer_stream(config: &SdkConfig) -> UnboundedReceiver<TimerData> {
    create_websocket_stream::<TimerData>(&contest_query_ws(
        config,
        "timer",
        &ContestQuery { contest: None },
    ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_url_prefix() {
        let config = SdkConfig::from_env();
        assert_eq!(config.api_prefix, "http://0.0.0.0/api");
    }

    #[test]
    fn check_ws_url_prefix() {
        let config = SdkConfig::from_env();
        assert_eq!(config.ws_prefix, "ws://0.0.0.0/api");
    }
}
