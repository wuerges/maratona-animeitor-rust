use clap::Parser;
use cli::{
    configuration::{ConfiguredContest, EventConfig, EventSecrets, ServerConfig},
    sentry,
};
use data::event::{ContestConfig, EventState, Run, SiteConfig};
use service::{event_store::from_legacy_contest_state, webcast};
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::{debug, error, info};
use url::Url;

#[derive(Parser)]
#[command(
    version,
    about = "Reset one event on startup, then publish its configuration and webcast to the internal HTTPS API"
)]
struct Args {
    #[arg(long)]
    event_config: PathBuf,
    #[arg(long)]
    event_secrets: PathBuf,
    #[arg(long)]
    server_config: PathBuf,
}
/// Encodes one value used as a URL path segment. Contest and site names are
/// display names and commonly contain spaces or accents.
fn url_with_segments(base: &str, segments: &[&str]) -> String {
    let mut url = Url::parse(base).expect("server URL is valid");
    url.path_segments_mut()
        .expect("server URL has a hierarchical path")
        .extend(segments.iter().copied());
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::url_with_segments;

    #[test]
    fn url_segments_use_percent_encoding_and_preserve_hierarchy() {
        assert_eq!(
            url_with_segments(
                "http://localhost:8000/internal/events/nacional-2026",
                &["contests", "South America - South Finals", "sites"]
            ),
            "http://localhost:8000/internal/events/nacional-2026/contests/South%20America%20-%20South%20Finals/sites"
        );
        assert_eq!(
            url_with_segments(
                "http://localhost:8000/internal/sites",
                &["Antigua & Barbuda"]
            ),
            "http://localhost:8000/internal/sites/Antigua%20&%20Barbuda"
        );
    }
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    let _guard = sentry::setup();
    let args = Args::parse();
    let event = EventConfig::load(&args.event_config)?;
    let server = ServerConfig::load(&args.server_config)?;
    let source = EventSecrets::source(&args.event_secrets, &event.event.name)?;
    let token = server.credential()?;
    let mut feeder = Feeder::new(
        cli::http_client::build(Some(&server.tls_ca_cert))?,
        &token.token,
        &token.name,
        &server.server_url,
        &event.event.name,
        event.configured(),
        event.event.score_freeze_time_seconds,
    );
    feeder.photo_url_format = event.event.photo_url_format;
    feeder.sound_url_format = event.event.sound_url_format;
    feeder.event_secret = event.event.secret;
    feeder.db_update_loop(&source).await;
    Ok(())
}

/// The feeder loop: keeps the last-sent state so every poll sends only what
/// changed — event PUTs on static changes, time PATCHes, new/corrected
/// runs, and contests/sites only until the server confirms them.
struct Feeder {
    /// Completed once per feeder process, before publishing the first valid source snapshot.
    reset_complete: bool,
    event_secret: String,
    photo_url_format: Option<String>,
    sound_url_format: Option<String>,
    client: reqwest::Client,
    internal_token: String,
    internal_user: String,
    event: String,
    event_url: String,
    runs_url: String,
    contests_url: String,
    sites_url: String,
    configured: Vec<ConfiguredContest>,
    /// The state the server is known to hold (None until first confirmed).
    known_event: Option<EventState>,
    /// Contests/sites confirmed present on the server (`contest:NAME` and
    /// `site:CONTEST/NAME`), so they are not re-sent on every poll.
    confirmed: HashSet<String>,
    /// Overrides the webcast's score freeze time when set.
    score_freeze_time_seconds: Option<i64>,
}

impl Feeder {
    fn new(
        client: reqwest::Client,
        internal_token: &str,
        internal_user: &str,
        server_url: &str,
        event: &str,
        configured: Vec<ConfiguredContest>,
        score_freeze_time_seconds: Option<i64>,
    ) -> Self {
        Feeder {
            reset_complete: false,
            event_secret: String::new(),
            photo_url_format: None,
            sound_url_format: None,
            client,
            internal_token: internal_token.to_string(),
            internal_user: internal_user.to_string(),
            event: event.to_string(),
            event_url: url_with_segments(server_url, &["internal", "events", event]),
            runs_url: url_with_segments(server_url, &["internal", "events", event, "runs"]),
            contests_url: url_with_segments(server_url, &["internal", "contests", event]),
            sites_url: url_with_segments(server_url, &["internal", "sites", event]),
            configured,
            known_event: None,
            confirmed: HashSet::new(),
            score_freeze_time_seconds,
        }
    }

    async fn send(
        &self,
        method: reqwest::Method,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.client
            .request(method, url)
            .basic_auth(&self.internal_user, Some(&self.internal_token))
            .json(body)
            .send()
            .await
    }

    /// Retry a failed startup reset without publishing into the old event.
    /// Once deletion succeeds (or the event is absent), never repeat it during
    /// this process, even if later setup requests fail.
    async fn reset_event(&mut self) -> bool {
        if self.reset_complete {
            return true;
        }
        match self
            .client
            .delete(&self.event_url)
            .basic_auth(&self.internal_user, Some(&self.internal_token))
            .send()
            .await
        {
            Ok(response)
                if response.status().is_success()
                    || response.status() == reqwest::StatusCode::NOT_FOUND =>
            {
                self.known_event = None;
                self.confirmed.clear();
                self.reset_complete = true;
                info!(event = %self.event, "startup reset complete; rebuilding event");
                true
            }
            Ok(response) => {
                error!(event = %self.event, status = %response.status(), "startup reset failed; will retry before publishing");
                false
            }
            Err(error) => {
                error!(event = %self.event, %error, "startup reset request failed; will retry before publishing");
                false
            }
        }
    }

    async fn publish_snapshot(&mut self, state: EventState, runs: Vec<Run>) {
        if !self.reset_event().await {
            return;
        }
        self.update_event(state).await;
        if self.known_event.is_none() {
            return;
        }
        self.update_contests().await;
        self.update_runs(runs).await;
    }

    async fn get_event(&self) -> Option<EventState> {
        self.get(&self.event_url).await
    }

    /// Reads an enveloped internal-API resource.
    async fn get<T: for<'a> serde::Deserialize<'a>>(&self, url: &str) -> Option<T> {
        let response = match self
            .client
            .get(url)
            .basic_auth(&self.internal_user, Some(&self.internal_token))
            .send()
            .await
        {
            Ok(response) => response,
            Err(err) => {
                error!(%url, ?err, "network error reading internal API resource");
                return None;
            }
        };
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(%url, %status, %body, "status error reading internal API resource");
            return None;
        }
        match response.json::<data::event::Envelope<T>>().await {
            Ok(envelope) => envelope.data,
            Err(err) => {
                error!(%url, ?err, "invalid internal API response");
                None
            }
        }
    }

    /// Whether two event states differ only in the time (and the salt).
    fn same_static(a: &EventState, b: &EventState) -> bool {
        a.photo_url_format == b.photo_url_format
            && a.sound_url_format == b.sound_url_format
            && a.salt == b.salt
            && a.name == b.name
            && a.problems == b.problems
            && a.teams == b.teams
            && a.score_freeze_time_seconds == b.score_freeze_time_seconds
            && a.penalty_seconds == b.penalty_seconds
    }

    /// POSTs or PUTs the event state as needed: PUT only when the static
    /// state (everything except the time) changed; PATCH time when only the
    /// time moved; nothing when both are unchanged. The server's salt is
    /// synchronized from the public event configuration.
    async fn update_event(&mut self, state: EventState) {
        match &self.known_event {
            None => {
                // Create after reset; a conflict can follow a successful POST whose response was lost.
                let body = serde_json::to_value(&state).unwrap();
                match self
                    .send(reqwest::Method::POST, &self.event_url, &body)
                    .await
                {
                    Ok(response) if response.status().is_success() => {
                        info!("event created");
                        self.known_event = Some(state);
                    }
                    Ok(response) if response.status() == reqwest::StatusCode::CONFLICT => {
                        if let Some(existing) = self.get_event().await {
                            self.known_event = Some(existing);
                        }
                    }
                    Ok(response) => {
                        let body = response.text().await.unwrap_or_default();
                        error!(%body, "status error creating event");
                    }
                    Err(err) => error!(?err, "network error creating event"),
                }
            }
            Some(known) => {
                if Self::same_static(known, &state) {
                    if known.time_seconds != state.time_seconds {
                        let body = serde_json::json!({ "time_seconds": state.time_seconds });
                        match self
                            .send(
                                reqwest::Method::PATCH,
                                &format!("{}/time", self.event_url),
                                &body,
                            )
                            .await
                        {
                            Ok(response) if response.status().is_success() => {
                                let mut updated = known.clone();
                                updated.time_seconds = state.time_seconds;
                                self.known_event = Some(updated);
                            }
                            Ok(response) => {
                                let body = response.text().await.unwrap_or_default();
                                error!(%body, "status error patching time");
                            }
                            Err(err) => error!(?err, "network error patching time"),
                        }
                    }
                } else {
                    let body = serde_json::to_value(&state).unwrap();
                    match self
                        .send(reqwest::Method::PUT, &self.event_url, &body)
                        .await
                    {
                        Ok(response) if response.status().is_success() => {
                            info!("event updated");
                            self.known_event = Some(state);
                        }
                        Ok(response) => {
                            let body = response.text().await.unwrap_or_default();
                            error!(%body, "status error updating event");
                        }
                        Err(err) => error!(?err, "network error updating event"),
                    }
                }
            }
        }
    }

    /// Sends the complete webcast run list. Deduplication and correction
    /// handling belong to the server, which indexes runs by their ID.
    async fn update_runs(&mut self, runs: Vec<Run>) {
        if runs.is_empty() {
            return;
        }
        let body = serde_json::json!({ "runs": runs });
        match self
            .send(reqwest::Method::POST, &self.runs_url, &body)
            .await
        {
            Ok(response) => match response.error_for_status_ref() {
                Ok(_) => {
                    debug!(
                        "{} runs sent",
                        body["runs"].as_array().map_or(0, |r| r.len())
                    );
                }
                Err(err) => {
                    let body = response.text().await.unwrap_or_default();
                    error!(?err, %body, "status error sending runs");
                }
            },
            Err(err) => error!(?err, "network error sending runs"),
        }
    }

    /// Ensures the configured contests and their sites exist. Confirmed ones
    /// are skipped after they have been synchronized. Existing salts are
    /// replaced by configured values, including empty optional values.
    async fn update_contests(&mut self) {
        let existing_contests = self
            .get::<Vec<ContestConfig>>(&format!("{}/contests", self.event_url))
            .await
            .unwrap_or_default();

        for contest in &self.configured {
            let name = &contest.config.name;
            let key = format!("contest:{name}");
            if !self.confirmed.contains(&key) {
                let url = url_with_segments(&self.contests_url, &[name]);
                let desired = contest.config.clone();
                let method = if existing_contests
                    .iter()
                    .any(|existing| existing.name == *name)
                {
                    reqwest::Method::PUT
                } else {
                    reqwest::Method::POST
                };
                let body = serde_json::to_value(&desired).unwrap();
                match self.send(method, &url, &body).await {
                    Ok(response) if response.status().is_success() => {
                        debug!("contest {name} synchronized");
                        self.confirmed.insert(key);
                    }
                    Ok(response) => {
                        let body = response.text().await.unwrap_or_default();
                        error!(%body, "status error synchronizing contest {name}");
                    }
                    Err(err) => error!(?err, "network error synchronizing contest {name}"),
                }
            }

            let existing_sites = self
                .get::<Vec<SiteConfig>>(&url_with_segments(
                    &self.event_url,
                    &["contests", name.as_str(), "sites"],
                ))
                .await
                .unwrap_or_default();
            for site in &contest.sites {
                let site_key = format!("site:{name}/{}", site.name);
                if self.confirmed.contains(&site_key) {
                    continue;
                }
                let url = url_with_segments(&self.sites_url, &[name, site.name.as_str()]);
                let desired = site.clone();
                let method = if existing_sites
                    .iter()
                    .any(|existing| existing.name == site.name)
                {
                    reqwest::Method::PUT
                } else {
                    reqwest::Method::POST
                };
                let body = serde_json::to_value(&desired).unwrap();
                match self.send(method, &url, &body).await {
                    Ok(response) if response.status().is_success() => {
                        debug!("site {}/{} synchronized", name, site.name);
                        self.confirmed.insert(site_key);
                    }
                    Ok(response) => {
                        let body = response.text().await.unwrap_or_default();
                        error!(%body, "status error synchronizing site {}/{}", name, site.name);
                    }
                    Err(err) => error!(
                        ?err,
                        "network error synchronizing site {}/{}", name, site.name
                    ),
                }
            }
        }
    }

    pub async fn db_update_loop(&mut self, boca_url: &str) {
        let dur = tokio::time::Duration::new(1, 0);
        let mut interval = tokio::time::interval(dur);

        // The first tick fires immediately, before the server (started in
        // parallel by the Makefile) is listening.
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        loop {
            interval.tick().await;

            match webcast::load_data_from_url_maybe(boca_url).await {
                Ok(contest_state) => {
                    let (mut state, runs) = from_legacy_contest_state(&contest_state, &self.event);
                    if let Some(freeze) = self.score_freeze_time_seconds {
                        state.score_freeze_time_seconds = freeze;
                    }
                    state.salt = Some(self.event_secret.clone());
                    state.photo_url_format = self.photo_url_format.clone();
                    state.sound_url_format = self.sound_url_format.clone();
                    // Load the source successfully before deleting existing data.
                    self.publish_snapshot(state, runs).await;
                }
                Err(_) => error!(
                    "failed loading webcast; check the private source configuration; will retry"
                ),
            }
        }
    }
}

#[cfg(test)]
mod startup_tests {
    use super::*;
    use axum::{
        Router,
        extract::{Request, State},
        http::StatusCode,
        response::IntoResponse,
    };
    use std::{collections::VecDeque, sync::Arc};
    use tokio::sync::Mutex;

    type Calls = Arc<Mutex<Vec<(String, String)>>>;
    #[derive(Clone)]
    struct Mock {
        calls: Calls,
        deletes: Arc<Mutex<VecDeque<StatusCode>>>,
        creates: Arc<Mutex<VecDeque<StatusCode>>>,
    }
    async fn handler(State(mock): State<Mock>, request: Request) -> axum::response::Response {
        let method = request.method().to_string();
        let path = request.uri().path().to_string();
        assert_eq!(request.headers()["authorization"], "Basic dXNlcjp0b2tlbg==");
        mock.calls.lock().await.push((method.clone(), path.clone()));
        if method == "DELETE" {
            return mock
                .deletes
                .lock()
                .await
                .pop_front()
                .unwrap_or(StatusCode::NO_CONTENT)
                .into_response();
        }
        if method == "GET" {
            return axum::Json(serde_json::json!({"data":[]})).into_response();
        }
        if method == "POST" && path == "/internal/events/e" {
            return mock
                .creates
                .lock()
                .await
                .pop_front()
                .unwrap_or(StatusCode::CREATED)
                .into_response();
        }
        StatusCode::OK.into_response()
    }
    async fn setup(
        deletes: Vec<StatusCode>,
        creates: Vec<StatusCode>,
    ) -> (Feeder, Calls, tokio::task::JoinHandle<()>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mock = Mock {
            calls: calls.clone(),
            deletes: Arc::new(Mutex::new(deletes.into())),
            creates: Arc::new(Mutex::new(creates.into())),
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            axum::serve(listener, Router::new().fallback(handler).with_state(mock))
                .await
                .unwrap();
        });
        let contests = vec![ConfiguredContest {
            config: serde_json::from_value(serde_json::json!({"name":"c","codes":["team"]}))
                .unwrap(),
            sites: vec![
                serde_json::from_value(serde_json::json!({"name":"s","codes":["team"]})).unwrap(),
            ],
        }];
        (
            Feeder::new(
                reqwest::Client::new(),
                "token",
                "user",
                &base,
                "e",
                contests,
                None,
            ),
            calls,
            task,
        )
    }
    fn state() -> EventState {
        serde_json::from_value(serde_json::json!({"name":"e","teams":[],"problems":["A"],"score_freeze_time_seconds":100,"penalty_seconds":1200,"time_seconds":-60})).unwrap()
    }
    fn runs() -> Vec<Run> {
        vec![serde_json::from_value(serde_json::json!({"id":1,"team_login":"team1","prob":"A","time_seconds":10,"answer":"Y"})).unwrap()]
    }

    #[tokio::test]
    async fn startup_deletes_before_rebuilding_and_never_resets_on_later_polls() {
        let (mut feeder, calls, server) = setup(vec![StatusCode::NO_CONTENT], vec![]).await;
        feeder.publish_snapshot(state(), runs()).await;
        let first = calls.lock().await.clone();
        assert_eq!(
            first,
            vec![
                ("DELETE".into(), "/internal/events/e".into()),
                ("POST".into(), "/internal/events/e".into()),
                ("GET".into(), "/internal/events/e/contests".into()),
                ("POST".into(), "/internal/contests/e/c".into()),
                ("GET".into(), "/internal/events/e/contests/c/sites".into()),
                ("POST".into(), "/internal/sites/e/c/s".into()),
                ("POST".into(), "/internal/events/e/runs".into()),
            ]
        );
        let mut next = state();
        next.time_seconds = 12;
        feeder.publish_snapshot(next, runs()).await;
        let calls = calls.lock().await;
        assert_eq!(calls.iter().filter(|(m, _)| m == "DELETE").count(), 1);
        assert!(calls.contains(&("PATCH".into(), "/internal/events/e/time".into())));
        server.abort();
    }

    #[tokio::test]
    async fn failed_reset_blocks_publication_and_retries_until_absent() {
        let (mut feeder, calls, server) = setup(
            vec![StatusCode::SERVICE_UNAVAILABLE, StatusCode::NOT_FOUND],
            vec![],
        )
        .await;
        feeder.publish_snapshot(state(), runs()).await;
        assert!(!feeder.reset_complete);
        assert_eq!(calls.lock().await.len(), 1);
        feeder.publish_snapshot(state(), runs()).await;
        assert!(feeder.reset_complete);
        assert!(feeder.known_event.is_some());
        assert_eq!(
            calls.lock().await[1],
            ("DELETE".into(), "/internal/events/e".into())
        );
        server.abort();
    }

    #[tokio::test]
    async fn failed_creation_retries_without_deleting_again_or_sending_children() {
        let (mut feeder, calls, server) = setup(
            vec![StatusCode::NO_CONTENT],
            vec![StatusCode::SERVICE_UNAVAILABLE, StatusCode::CREATED],
        )
        .await;
        feeder.publish_snapshot(state(), runs()).await;
        assert_eq!(calls.lock().await.len(), 2);
        assert!(feeder.reset_complete);
        assert!(feeder.known_event.is_none());
        feeder.publish_snapshot(state(), runs()).await;
        assert_eq!(
            calls
                .lock()
                .await
                .iter()
                .filter(|(m, _)| m == "DELETE")
                .count(),
            1
        );
        assert!(feeder.known_event.is_some());
        server.abort();
    }
}
