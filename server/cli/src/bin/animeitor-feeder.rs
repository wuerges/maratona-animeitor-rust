use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;
use url::Url;

use cli::sentry;
use data::configdata::{ConfigContest, SedeEntry};
use data::event::{ContestConfig, EventState, Run, SiteConfig};
use service::event_store::from_legacy_contest_state;
use service::webcast;
use tracing::{debug, error, info};

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// Feeder: polls the webcast and publishes the event state, runs and the
/// configured contests/sites into the internal API of an animeitor server.
/// Only what changed since the last poll is sent.
struct SimpleParser {
    /// Token for the internal API (/internal).
    #[clap(short = 't', long)]
    internal_token: String,

    /// The webcast url from BOCA (an URL or a local zip path).
    #[clap(short = 'i')]
    boca_url: String,

    /// The animeitor server url.
    #[clap(short = 's')]
    server_url: String,

    /// The event fed by this loop.
    #[clap(long, default_value = "default")]
    event: String,

    /// Old-format contest config (repeatable): each file's `[titulo]` and
    /// `[[sedes]]` become a contest and its sites in the internal API.
    #[clap(short = 'c', long = "contest")]
    contests: Vec<PathBuf>,

    /// Secrets file (`[[secrets]]` name/secret entries) matching the salt
    /// of each contest and site.
    #[clap(long)]
    secrets: Option<PathBuf>,

    /// Photo URL format set on every contest (the client substitutes
    /// `{team_login}`); served by the contest's public config.
    #[clap(long)]
    photo_url_format: Option<String>,

    /// Sound URL format set on every contest (the client substitutes
    /// `{team_login}`).
    #[clap(long)]
    sound_url_format: Option<String>,

    /// Overrides the score freeze time of the webcast (seconds; the client
    /// freezes the scoreboard at this time).
    #[clap(long)]
    score_freeze_time_seconds: Option<i64>,
}

#[derive(Deserialize)]
struct SecretsFile {
    secrets: Vec<SecretEntry>,
}

#[derive(Deserialize)]
struct SecretEntry {
    name: String,
    secret: String,
}

/// One contest and its sites, translated from the old config format to the
/// internal API shapes.
struct ConfiguredContest {
    config: ContestConfig,
    sites: Vec<SiteConfig>,
}

impl ConfiguredContest {
    /// The catch-all contest used when no config files are given: its empty
    /// regex matches every team login.
    fn default(media: &MediaFormats) -> Self {
        ConfiguredContest {
            config: ContestConfig {
                name: "default".to_string(),
                codes: vec!["".to_string()],
                salt: None,
                style: None,
                ouro: 1,
                prata: 2,
                bronze: 3,
                photo_url_format: media.photo.clone(),
                sound_url_format: media.sound.clone(),
            },
            sites: Vec::new(),
        }
    }
}

/// The photo/sound URL formats set on every contest (the client substitutes
/// `{team_login}`); `None` leaves the client's deploy-level defaults.
#[derive(Clone)]
struct MediaFormats {
    photo: Option<String>,
    sound: Option<String>,
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
            url_with_segments("http://localhost:8000/internal/sites", &["Antigua & Barbuda"]),
            "http://localhost:8000/internal/sites/Antigua%20&%20Barbuda"
        );
    }
}

fn load_contests(
    files: &[PathBuf],
    secrets: &HashMap<String, String>,
    media: &MediaFormats,
) -> color_eyre::eyre::Result<Vec<ConfiguredContest>> {
    let mut contests = Vec::new();
    for file in files {
        let raw = std::fs::read_to_string(file)?;
        let legacy: ConfigContest = toml::from_str(&raw)?;
        let name = legacy.titulo.name.clone();
        let config = ContestConfig {
            name: name.clone(),
            codes: legacy.titulo.codes.codes().to_vec(),
            salt: secrets.get(&name).cloned(),
            style: legacy.titulo.style.clone(),
            ouro: legacy.titulo.ouro,
            prata: legacy.titulo.prata,
            bronze: legacy.titulo.bronze,
            photo_url_format: media.photo.clone(),
            sound_url_format: media.sound.clone(),
        };
        let sites = legacy
            .sedes
            .unwrap_or_default()
            .into_iter()
            .map(|sede: SedeEntry| SiteConfig {
                name: sede.name.clone(),
                codes: sede.codes.codes().to_vec(),
                salt: secrets.get(&sede.name).cloned(),
            })
            .collect();
        contests.push(ConfiguredContest { config, sites });
    }
    Ok(contests)
}

fn load_secrets(path: Option<&PathBuf>) -> color_eyre::eyre::Result<HashMap<String, String>> {
    let Some(path) = path else {
        return Ok(HashMap::new());
    };
    let raw = std::fs::read_to_string(path)?;
    let parsed: SecretsFile = toml::from_str(&raw)?;
    Ok(parsed
        .secrets
        .into_iter()
        .map(|entry| (entry.name, entry.secret))
        .collect())
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    tracing::info!("\nSetting up sentry guard");
    let _guard = sentry::setup();

    let SimpleParser {
        internal_token,
        boca_url,
        server_url,
        event,
        contests,
        secrets,
        photo_url_format,
        sound_url_format,
        score_freeze_time_seconds,
    } = SimpleParser::parse();

    let media = MediaFormats {
        photo: photo_url_format,
        sound: sound_url_format,
    };
    let secrets = load_secrets(secrets.as_ref())?;
    let mut contests = load_contests(&contests, &secrets, &media)?;
    if contests.is_empty() {
        // No config files: the standalone flow uses the catch-all contest.
        contests.push(ConfiguredContest::default(&media));
    }

    let mut feeder = Feeder::new(
        &internal_token,
        &server_url,
        &event,
        contests,
        score_freeze_time_seconds,
    );

    feeder.db_update_loop(&boca_url).await;

    Ok(())
}

/// The feeder loop: keeps the last-sent state so every poll sends only what
/// changed — event PUTs on static changes, time PATCHes, new/corrected
/// runs, and contests/sites only until the server confirms them.
struct Feeder {
    client: reqwest::Client,
    internal_token: String,
    event: String,
    event_url: String,
    runs_url: String,
    contests_url: String,
    sites_url: String,
    configured: Vec<ConfiguredContest>,
    /// The state the server is known to hold (None until first confirmed).
    known_event: Option<EventState>,
    /// Runs already accepted by the server, by id.
    sent_runs: HashMap<i64, Run>,
    /// Contests/sites confirmed present on the server (`contest:NAME` and
    /// `site:CONTEST/NAME`), so they are not re-sent on every poll.
    confirmed: HashSet<String>,
    /// Overrides the webcast's score freeze time when set.
    score_freeze_time_seconds: Option<i64>,
}

impl Feeder {
    fn new(
        internal_token: &str,
        server_url: &str,
        event: &str,
        configured: Vec<ConfiguredContest>,
        score_freeze_time_seconds: Option<i64>,
    ) -> Self {
        Feeder {
            client: reqwest::Client::new(),
            internal_token: internal_token.to_string(),
            event: event.to_string(),
            event_url: url_with_segments(server_url, &["internal", "events", event]),
            runs_url: url_with_segments(server_url, &["internal", "events", event, "runs"]),
            contests_url: url_with_segments(server_url, &["internal", "contests", event]),
            sites_url: url_with_segments(server_url, &["internal", "sites", event]),
            configured,
            known_event: None,
            sent_runs: HashMap::new(),
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
            .basic_auth("usuario", Some(&self.internal_token))
            .json(body)
            .send()
            .await
    }

    async fn get_event(&self) -> Option<EventState> {
        self.get(&self.event_url).await
    }

    /// Reads an enveloped internal-API resource.
    async fn get<T: for<'a> serde::Deserialize<'a>>(&self, url: &str) -> Option<T> {
        let response = match self
            .client
            .get(url)
            .basic_auth("usuario", Some(&self.internal_token))
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
        a.name == b.name
            && a.problems == b.problems
            && a.teams == b.teams
            && a.score_freeze_time_seconds == b.score_freeze_time_seconds
            && a.penalty_seconds == b.penalty_seconds
    }

    /// POSTs or PUTs the event state as needed: PUT only when the static
    /// state (everything except the time) changed; PATCH time when only the
    /// time moved; nothing when both are unchanged. The server's salt is
    /// preserved across PUTs and generated once when missing.
    async fn update_event(&mut self, state: EventState) {
        match &self.known_event {
            None => {
                // First poll: create, or adopt the existing event.
                let body = serde_json::to_value(&state).unwrap();
                match self.send(reqwest::Method::POST, &self.event_url, &body).await {
                    Ok(response) if response.status().is_success() => {
                        info!("event created");
                        self.known_event = Some(state);
                        self.sent_runs.clear();
                    }
                    Ok(response) if response.status() == reqwest::StatusCode::CONFLICT => {
                        if let Some(existing) = self.get_event().await {
                            self.known_event = Some(existing);
                            self.sent_runs.clear();
                        }
                    }
                    Ok(response) => {
                        let body = response.text().await.unwrap_or_default();
                        error!(%body, "status error creating event");
                    }
                    Err(err) => error!(?err, "network error creating event"),
                }
                self.ensure_event_salt().await;
            }
            Some(known) => {
                if Self::same_static(known, &state) {
                    if known.time_seconds != state.time_seconds {
                        let body = serde_json::json!({ "time_seconds": state.time_seconds });
                        match self
                            .send(reqwest::Method::PATCH, &format!("{}/time", self.event_url), &body)
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
                    // An existing event keeps its salt: PUT replaces every
                    // field, and the webcast shape carries no salt.
                    let mut state = state;
                    state.salt = known.salt.clone();
                    let body = serde_json::to_value(&state).unwrap();
                    match self.send(reqwest::Method::PUT, &self.event_url, &body).await {
                        Ok(response) if response.status().is_success() => {
                            info!("event updated");
                            self.known_event = Some(state);
                            // Teams/problems changed: re-send every run so
                            // runs of newly-known teams get applied.
                            self.sent_runs.clear();
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

    /// Generates the event salt once (the server derives site keys from it;
    /// regenerating it would invalidate every site key).
    async fn ensure_event_salt(&mut self) {
        if self
            .known_event
            .as_ref()
            .is_none_or(|event| event.salt.is_some())
        {
            return;
        }
        match self
            .send(
                reqwest::Method::POST,
                &format!("{}/salt", self.event_url),
                &serde_json::json!({}),
            )
            .await
        {
            Ok(response) if response.status().is_success() => {
                if let Ok(envelope) = response
                    .json::<data::event::Envelope<serde_json::Value>>()
                    .await
                {
                    if let Some(salt) = envelope
                        .data
                        .and_then(|data| data["salt"].as_str().map(String::from))
                    {
                        if let Some(event) = &mut self.known_event {
                            event.salt = Some(salt);
                        }
                    }
                }
            }
            Ok(response) => {
                let body = response.text().await.unwrap_or_default();
                error!(%body, "status error setting event salt");
            }
            Err(err) => error!(?err, "network error setting event salt"),
        }
    }

    /// When the event has a salt, every contest and site fed by this process
    /// must have one too so each site has a usable reveal key. Existing salts
    /// (including values from `--secrets`) are deliberately preserved: POST
    /// is only used for resources whose internal representation has no salt.
    async fn ensure_reveal_salts(&self) {
        if self
            .known_event
            .as_ref()
            .is_none_or(|event| event.salt.is_none())
        {
            return;
        }

        let Some(contests) = self
            .get::<Vec<ContestConfig>>(&format!("{}/contests", self.event_url))
            .await
        else {
            error!("could not list contests while enabling reveals");
            return;
        };

        for contest in contests {
            if contest.salt.is_none() {
                self.generate_salt(
                    &url_with_segments(
                        &self.contests_url,
                        &[contest.name.as_str(), "salt"],
                    ),
                    "contest",
                )
                .await;
            }

            let sites_url = url_with_segments(
                &self.event_url,
                &["contests", contest.name.as_str(), "sites"],
            );
            let Some(sites) = self.get::<Vec<SiteConfig>>(&sites_url).await else {
                error!(contest = %contest.name, "could not list sites while enabling reveals");
                continue;
            };
            for site in sites {
                if site.salt.is_none() {
                    self.generate_salt(
                        &url_with_segments(
                            &self.sites_url,
                            &[contest.name.as_str(), site.name.as_str(), "salt"]
                        ),
                        "site",
                    )
                    .await;
                }
            }
        }
    }

    /// Asks the server to generate one salt. This is never called for a
    /// resource that already has a salt, because it would rotate its URL key.
    async fn generate_salt(&self, url: &str, resource: &str) {
        match self
            .send(reqwest::Method::POST, url, &serde_json::json!({}))
            .await
        {
            Ok(response) if response.status().is_success() => {
                debug!(%resource, "reveal salt enabled");
            }
            Ok(response) => {
                let body = response.text().await.unwrap_or_default();
                error!(%resource, %body, "status error enabling reveal salt");
            }
            Err(err) => error!(%resource, ?err, "network error enabling reveal salt"),
        }
    }

    /// Sends only the runs that are new or changed since the last poll.
    async fn update_runs(&mut self, runs: Vec<Run>) {
        let fresh: Vec<Run> = runs
            .into_iter()
            .filter(|run| self.sent_runs.get(&run.id) != Some(run))
            .collect();
        if fresh.is_empty() {
            return;
        }
        let body = serde_json::json!({ "runs": fresh });
        match self.send(reqwest::Method::POST, &self.runs_url, &body).await {
            Ok(response) => match response.error_for_status_ref() {
                Ok(_) => {
                    debug!("{} runs sent", body["runs"].as_array().map_or(0, |r| r.len()));
                    if let Some(runs) = body["runs"].as_array() {
                        for run in runs {
                            if let Ok(run) = serde_json::from_value::<Run>(run.clone()) {
                                self.sent_runs.insert(run.id, run);
                            }
                        }
                    }
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
    /// carried forward when a config file does not specify one.
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
                let mut desired = contest.config.clone();
                let method = if let Some(existing) = existing_contests
                    .iter()
                    .find(|existing| existing.name == *name)
                {
                    if desired.salt.is_none() {
                        desired.salt = existing.salt.clone();
                    }
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
                let mut desired = site.clone();
                let method = if let Some(existing) = existing_sites
                    .iter()
                    .find(|existing| existing.name == site.name)
                {
                    if desired.salt.is_none() {
                        desired.salt = existing.salt.clone();
                    }
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
                    Err(err) => error!(?err, "network error synchronizing site {}/{}", name, site.name),
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
                    self.update_event(state).await;
                    self.update_contests().await;
                    self.ensure_reveal_salts().await;
                    self.update_runs(runs).await;
                }
                Err(err) => error!(?err, "failed loading contest state from BOCA, will retry"),
            }
        }
    }
}
