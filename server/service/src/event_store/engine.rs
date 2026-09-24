//! Per-event state store for the internal and public APIs.
//!
//! Implements the resource model of `doc/event-api.md`: events →
//! contests → sites, with optional salts at the three levels, runs and
//! per-event broadcasters.

#[path = "incremental.rs"]
mod incremental;

use std::collections::HashMap;
use std::sync::Arc;

use hmac::{Hmac, Mac};
use rand::distr::{Alphanumeric, SampleString};
use regex::RegexSet;
use sha2::Sha256;
use tokio::sync::{RwLock, broadcast};

use crate::membroadcast;
use crate::remote_control::{ControlSender, create_remote_control};

const SALT_LEN: usize = 32;
const KEY_LEN: usize = 12;
const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

// The wire types live in `data` so the client can consume them too.
pub use data::event::{
    Answer, ContestConfig, Envelope, ErrorEntry, EventState, PublicConfig, PublicContestState,
    PublicSiteView, PublicTimer, Run, RunsData, SiteConfig, TeamInfo,
};

/// Why an internal-API operation failed.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum StoreError {
    #[error(transparent)]
    Storage(#[from] crate::database::DatabaseError),
    #[error("{0}")]
    Conflict(String),
    #[error("{0} já existe")]
    AlreadyExists(String),
    #[error("{0} não existe")]
    NotFound(String),
    #[error("{0}")]
    InvalidValue(String),
    #[error("regex inválida: {0}")]
    InvalidRegex(String),
}

type HmacSha256 = Hmac<Sha256>;

fn hmac_sha256(key: &str, message: &str) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC accepts any key size");
    mac.update(message.as_bytes());
    mac.finalize().into_bytes().into()
}

/// Encodes bytes as a base62 string (most significant digit first).
fn base62(bytes: &[u8]) -> String {
    fn divmod(num: &mut [u8], divisor: u32) -> u32 {
        let mut rem: u32 = 0;
        for byte in num.iter_mut() {
            let cur = (rem << 8) | u32::from(*byte);
            *byte = (cur / divisor) as u8;
            rem = cur % divisor;
        }
        rem
    }

    let mut num: Vec<u8> = bytes.iter().skip_while(|&&b| b == 0).copied().collect();
    if num.is_empty() {
        return "0".to_string();
    }
    let mut out = Vec::new();
    // The array never shrinks: it is zero once the value is exhausted.
    while num.iter().any(|&byte| byte != 0) {
        let rem = divmod(&mut num, 62);
        out.push(ALPHABET[rem as usize]);
    }
    out.reverse();
    String::from_utf8(out).expect("base62 alphabet is valid UTF-8")
}

/// Legacy derivation retained only for regression tests.
///
/// Missing salts contribute empty strings; the site name still provides a
/// deterministic domain, so revelation remains available without salts.
#[cfg(test)]
fn site_key(
    event_salt: Option<&str>,
    contest_salt: Option<&str>,
    site_salt: Option<&str>,
    contest_name: &str,
    site_name: &str,
) -> Option<String> {
    site_key_with_event(
        event_salt,
        contest_salt,
        site_salt,
        "",
        contest_name,
        site_name,
    )
}

/// Legacy event-aware derivation retained only for regression tests.
#[cfg(test)]
fn site_key_with_event(
    event_salt: Option<&str>,
    contest_salt: Option<&str>,
    site_salt: Option<&str>,
    event_name: &str,
    contest_name: &str,
    site_name: &str,
) -> Option<String> {
    let key = format!(
        "{}:{}:{}",
        event_salt.unwrap_or_default(),
        contest_salt.unwrap_or_default(),
        site_salt.unwrap_or_default()
    );
    let message = format!("{}:{}:{}", event_name, contest_name, site_name);
    let digest = hmac_sha256(&key, &message);
    let encoded = base62(&digest);
    Some(encoded.chars().take(KEY_LEN).collect())
}

/// Deployment-bound site key. Event values are public inputs, not HMAC keys.
pub fn deployment_site_key(
    server_salt: &str,
    event_name: &str,
    contest_name: &str,
    site_name: &str,
    event_secret: &str,
    contest_secret: &str,
    site_secret: &str,
) -> String {
    let message = serde_json::to_string(&[
        "animeitor-site-key-v1",
        event_name,
        contest_name,
        site_name,
        event_secret,
        contest_secret,
        site_secret,
    ])
    .expect("strings serialize");
    base62(&hmac_sha256(server_salt, &message))
        .chars()
        .take(KEY_LEN)
        .collect()
}

fn generate_salt() -> String {
    Alphanumeric.sample_string(&mut rand::rng(), SALT_LEN)
}

fn compile_codes(codes: &[String]) -> Result<RegexSet, StoreError> {
    RegexSet::new(codes).map_err(|err| StoreError::InvalidRegex(err.to_string()))
}

fn check_name(body: &str, path: &str, what: &str) -> Result<(), StoreError> {
    if !body.is_empty() && body != path {
        return Err(StoreError::InvalidValue(format!(
            "name do {what} não confere com o caminho"
        )));
    }
    Ok(())
}

/// Contest names are required and non-empty (there is no default contest).
fn check_contest_name(body: &str, path: &str) -> Result<(), StoreError> {
    if body.is_empty() || path.is_empty() {
        return Err(StoreError::InvalidValue(
            "nome do contest não pode ser vazio".into(),
        ));
    }
    check_name(body, path, "contest")
}

struct SiteEntry {
    config: SiteConfig,
    codes: RegexSet,
}

struct ContestEntry {
    config: ContestConfig,
    codes: RegexSet,
    sites: HashMap<String, SiteEntry>,
    remote_control: HashMap<String, ControlSender>,
}

struct Event {
    name: String,
    problems: Vec<String>,
    teams: Vec<TeamInfo>,
    score_freeze_time_seconds: i64,
    penalty_seconds: i64,
    time_seconds: i64,
    salt: Option<String>,
    contests: HashMap<String, ContestEntry>,
    /// Runs in arrival order, indexed by id for corrections.
    runs: Vec<Run>,
    runs_index: HashMap<i64, usize>,
    runs_tx: membroadcast::Sender<Run>,
    timer_tx: broadcast::Sender<PublicTimer>,
}

impl Event {
    fn timer(&self) -> PublicTimer {
        PublicTimer {
            current_time_seconds: self.time_seconds,
            score_freeze_time_seconds: self.score_freeze_time_seconds,
        }
    }

    fn publish_timer(&self) {
        // Lagging receivers are dropped; the timer is re-sent on every change.
        let _ = self.timer_tx.send(self.timer());
    }
}

struct Inner {
    order: Vec<String>,
    events: HashMap<String, Event>,
}

/// The shared store of all events of the server.
#[derive(Clone)]
pub(super) struct Engine {
    channel_capacity: usize,
    revelation_salt: Arc<String>,
    inner: Arc<RwLock<Inner>>,
}

impl Engine {
    pub(super) fn staging(salt: String) -> Self {
        let mut engine = Self::with_revelation_salt(salt);
        engine.channel_capacity = 1;
        engine
    }
    pub fn new() -> Self {
        Self::with_revelation_salt(generate_salt())
    }

    pub fn with_revelation_salt(salt: String) -> Self {
        assert!(!salt.is_empty(), "revelation salt must not be empty");
        Self {
            channel_capacity: 1_000_000,
            revelation_salt: Arc::new(salt),
            inner: Arc::new(RwLock::new(Inner {
                order: Vec::new(),
                events: HashMap::new(),
            })),
        }
    }

    /// Creates a new event. The `name` in the state must match `event_name`.
    pub async fn create_event(
        &self,
        event_name: &str,
        state: EventState,
    ) -> Result<(), StoreError> {
        let mut inner = self.inner.write().await;
        if inner.events.contains_key(event_name) {
            return Err(StoreError::AlreadyExists(format!("evento {event_name}")));
        }
        if state.name != event_name {
            return Err(StoreError::InvalidValue(
                "name do evento não confere com o caminho".into(),
            ));
        }
        let (runs_tx, _) = membroadcast::channel(self.channel_capacity);
        let (timer_tx, _) = broadcast::channel(self.channel_capacity);
        inner.events.insert(
            event_name.to_string(),
            Event {
                name: state.name,
                problems: state.problems,
                teams: state.teams,
                score_freeze_time_seconds: state.score_freeze_time_seconds,
                penalty_seconds: state.penalty_seconds,
                time_seconds: state.time_seconds,
                salt: state.salt,
                contests: HashMap::new(),
                runs: Vec::new(),
                runs_index: HashMap::new(),
                runs_tx,
                timer_tx,
            },
        );
        inner.order.push(event_name.to_string());
        if let Some(event) = inner.events.get(event_name) {
            event.publish_timer();
        }
        Ok(())
    }

    /// Replaces the state fields of an event. Contests, sites and runs are kept.
    pub async fn put_event(&self, event_name: &str, state: EventState) -> Result<(), StoreError> {
        let mut inner = self.inner.write().await;
        let event = inner
            .events
            .get_mut(event_name)
            .ok_or_else(|| StoreError::NotFound(format!("evento {event_name}")))?;
        if state.name != event_name {
            return Err(StoreError::InvalidValue(
                "name do evento não confere com o caminho".into(),
            ));
        }
        event.name = state.name;
        event.problems = state.problems;
        event.teams = state.teams;
        event.score_freeze_time_seconds = state.score_freeze_time_seconds;
        event.penalty_seconds = state.penalty_seconds;
        event.time_seconds = state.time_seconds;
        event.salt = state.salt;
        event.publish_timer();
        Ok(())
    }

    pub async fn get_event(&self, event_name: &str) -> Option<EventState> {
        let inner = self.inner.read().await;
        inner.events.get(event_name).map(|event| EventState {
            name: event.name.clone(),
            problems: event.problems.clone(),
            teams: event.teams.clone(),
            score_freeze_time_seconds: event.score_freeze_time_seconds,
            penalty_seconds: event.penalty_seconds,
            time_seconds: event.time_seconds,
            salt: event.salt.clone(),
        })
    }

    /// Removes an event with its contests, sites and runs.
    pub async fn delete_event(&self, event_name: &str) -> bool {
        let mut inner = self.inner.write().await;
        if inner.events.remove(event_name).is_none() {
            return false;
        }
        inner.order.retain(|name| name != event_name);
        true
    }

    /// Lists event names in creation order.
    #[cfg(test)]
    pub async fn list_events(&self) -> Vec<String> {
        let inner = self.inner.read().await;
        inner.order.clone()
    }

    /// Whether the event has started; `None` when the event does not exist.
    pub async fn is_started(&self, event_name: &str) -> Option<bool> {
        let inner = self.inner.read().await;
        inner
            .events
            .get(event_name)
            .map(|event| event.time_seconds >= 0)
    }

    /// Sets the event time (may be negative: countdown).
    pub async fn patch_time(&self, event_name: &str, seconds: i64) -> Option<i64> {
        let mut inner = self.inner.write().await;
        let event = inner.events.get_mut(event_name)?;
        event.time_seconds = seconds;
        event.publish_timer();
        Some(seconds)
    }

    /// Replaces the event salt; a missing/empty salt generates a random one.
    pub async fn set_event_salt(
        &self,
        event_name: &str,
        salt: Option<String>,
    ) -> Result<String, StoreError> {
        let mut inner = self.inner.write().await;
        let event = inner
            .events
            .get_mut(event_name)
            .ok_or_else(|| StoreError::NotFound(format!("evento {event_name}")))?;
        let salt = salt.filter(|s| !s.is_empty()).unwrap_or_else(generate_salt);
        event.salt = Some(salt.clone());
        Ok(salt)
    }

    /// Normalizes and applies runs by `(time_seconds, id)`: a new `id` is
    /// added, and an existing one corrects the previous result. Returns
    /// `(added, updated, ignored)` where
    /// `ignored` are the runs from teams not in the event (skipped, not
    /// rejected — see below).
    pub async fn add_runs(
        &self,
        event_name: &str,
        runs: Vec<Run>,
    ) -> Result<(usize, usize, Vec<Run>), StoreError> {
        let mut inner = self.inner.write().await;
        let event = inner
            .events
            .get_mut(event_name)
            .ok_or_else(|| StoreError::NotFound(format!("evento {event_name}")))?;
        // Runs from teams that are not part of the event (e.g. the judge
        // users of the MOJ feed) are ignored and logged, not rejected: the
        // feeder re-sends the whole state on every poll, so rejecting would
        // block every update forever.
        let mut ignored = Vec::new();
        let mut runs: Vec<Run> = runs
            .into_iter()
            .filter(|run| {
                let known = event.teams.iter().any(|team| team.login == run.team_login);
                if !known {
                    ignored.push(run.clone());
                }
                known
            })
            .collect();
        runs.sort_by_key(|run| (run.time_seconds, run.id));
        if !ignored.is_empty() {
            tracing::info!(?ignored, "ignoring runs from teams not in the event");
        }
        for run in &runs {
            if !event.problems.iter().any(|prob| prob == &run.prob) {
                return Err(StoreError::InvalidValue(format!(
                    "prob desconhecido: {}",
                    run.prob
                )));
            }
        }
        let mut added = 0;
        let mut updated = 0;
        for run in runs {
            match event.runs_index.get(&run.id) {
                Some(&index) => {
                    // Identical re-sends are no-ops: feeders re-send the full
                    // state on every poll, and only real corrections count.
                    if event.runs[index] == run {
                        continue;
                    }
                    event.runs[index] = run.clone();
                    updated += 1;
                }
                None => {
                    event.runs_index.insert(run.id, event.runs.len());
                    event.runs.push(run.clone());
                    added += 1;
                }
            }
            event.runs_tx.send_memo(run);
        }
        // A correction can change a run's timestamp. Keep the stored replay
        // canonical and rebuild the ID index after sorting.
        event.runs.sort_by_key(|run| (run.time_seconds, run.id));
        event.runs_index.clear();
        for (index, run) in event.runs.iter().enumerate() {
            event.runs_index.insert(run.id, index);
        }
        Ok((added, updated, ignored))
    }

    pub async fn delete_run(&self, event_name: &str, run_id: i64) -> bool {
        let mut inner = self.inner.write().await;
        let Some(event) = inner.events.get_mut(event_name) else {
            return false;
        };
        let Some(index) = event.runs_index.remove(&run_id) else {
            return false;
        };
        event.runs.remove(index);
        event.runs_index = event
            .runs
            .iter()
            .enumerate()
            .map(|(i, run)| (run.id, i))
            .collect();
        true
    }

    pub async fn clear_runs(&self, event_name: &str) -> bool {
        let mut inner = self.inner.write().await;
        let Some(event) = inner.events.get_mut(event_name) else {
            return false;
        };
        event.runs.clear();
        event.runs_index.clear();
        true
    }

    /// Creates a contest. The name in the config, when present, must match `contest_name`.
    pub async fn create_contest(
        &self,
        event_name: &str,
        contest_name: &str,
        config: ContestConfig,
    ) -> Result<(), StoreError> {
        let mut inner = self.inner.write().await;
        let event = inner
            .events
            .get_mut(event_name)
            .ok_or_else(|| StoreError::NotFound(format!("evento {event_name}")))?;
        if event.contests.contains_key(contest_name) {
            return Err(StoreError::AlreadyExists(format!("contest {contest_name}")));
        }
        check_contest_name(&config.name, contest_name)?;
        let codes = compile_codes(&config.codes)?;
        let mut config = config;
        config.name = contest_name.to_string();
        event.contests.insert(
            contest_name.to_string(),
            ContestEntry {
                config,
                codes,
                sites: HashMap::new(),
                remote_control: HashMap::new(),
            },
        );
        Ok(())
    }

    pub async fn get_contest(&self, event_name: &str, contest_name: &str) -> Option<ContestConfig> {
        let inner = self.inner.read().await;
        inner
            .events
            .get(event_name)?
            .contests
            .get(contest_name)
            .map(|entry| entry.config.clone())
    }

    /// Lists the contests of an event; `None` when the event does not exist.
    pub async fn list_contests(&self, event_name: &str) -> Option<Vec<ContestConfig>> {
        let inner = self.inner.read().await;
        inner.events.get(event_name).map(|event| {
            event
                .contests
                .values()
                .map(|entry| entry.config.clone())
                .collect()
        })
    }

    /// Replaces a contest. Its sites are kept.
    pub async fn put_contest(
        &self,
        event_name: &str,
        contest_name: &str,
        config: ContestConfig,
    ) -> Result<(), StoreError> {
        let mut inner = self.inner.write().await;
        let event = inner
            .events
            .get_mut(event_name)
            .ok_or_else(|| StoreError::NotFound(format!("evento {event_name}")))?;
        let entry = event
            .contests
            .get_mut(contest_name)
            .ok_or_else(|| StoreError::NotFound(format!("contest {contest_name}")))?;
        check_contest_name(&config.name, contest_name)?;
        let codes = compile_codes(&config.codes)?;
        let mut config = config;
        config.name = contest_name.to_string();
        entry.config = config;
        entry.codes = codes;
        Ok(())
    }

    /// Removes a contest with its sites.
    pub async fn delete_contest(&self, event_name: &str, contest_name: &str) -> bool {
        let mut inner = self.inner.write().await;
        let Some(event) = inner.events.get_mut(event_name) else {
            return false;
        };
        event.contests.remove(contest_name).is_some()
    }

    /// Replaces the contest salt; a missing/empty salt generates a random one.
    pub async fn set_contest_salt(
        &self,
        event_name: &str,
        contest_name: &str,
        salt: Option<String>,
    ) -> Result<String, StoreError> {
        let mut inner = self.inner.write().await;
        let event = inner
            .events
            .get_mut(event_name)
            .ok_or_else(|| StoreError::NotFound(format!("evento {event_name}")))?;
        let entry = event
            .contests
            .get_mut(contest_name)
            .ok_or_else(|| StoreError::NotFound(format!("contest {contest_name}")))?;
        let salt = salt.filter(|s| !s.is_empty()).unwrap_or_else(generate_salt);
        entry.config.salt = Some(salt.clone());
        Ok(salt)
    }

    /// Creates a site. The name in the config, when present, must match `site_name`.
    pub async fn create_site(
        &self,
        event_name: &str,
        contest_name: &str,
        site_name: &str,
        config: SiteConfig,
    ) -> Result<(), StoreError> {
        let mut inner = self.inner.write().await;
        let event = inner
            .events
            .get_mut(event_name)
            .ok_or_else(|| StoreError::NotFound(format!("evento {event_name}")))?;
        let contest = event
            .contests
            .get_mut(contest_name)
            .ok_or_else(|| StoreError::NotFound(format!("contest {contest_name}")))?;
        if contest.sites.contains_key(site_name) {
            return Err(StoreError::AlreadyExists(format!("site {site_name}")));
        }
        check_name(&config.name, site_name, "site")?;
        let codes = compile_codes(&config.codes)?;
        let mut config = config;
        config.name = site_name.to_string();
        contest
            .sites
            .insert(site_name.to_string(), SiteEntry { config, codes });
        Ok(())
    }

    pub async fn get_site(
        &self,
        event_name: &str,
        contest_name: &str,
        site_name: &str,
    ) -> Option<SiteConfig> {
        let inner = self.inner.read().await;
        inner
            .events
            .get(event_name)?
            .contests
            .get(contest_name)?
            .sites
            .get(site_name)
            .map(|entry| entry.config.clone())
    }

    /// Lists the sites of a contest; `None` when the event or the contest
    /// does not exist.
    pub async fn list_sites(
        &self,
        event_name: &str,
        contest_name: &str,
    ) -> Option<Vec<SiteConfig>> {
        let inner = self.inner.read().await;
        inner
            .events
            .get(event_name)?
            .contests
            .get(contest_name)
            .map(|contest| {
                contest
                    .sites
                    .values()
                    .map(|entry| entry.config.clone())
                    .collect()
            })
    }

    pub async fn put_site(
        &self,
        event_name: &str,
        contest_name: &str,
        site_name: &str,
        config: SiteConfig,
    ) -> Result<(), StoreError> {
        let mut inner = self.inner.write().await;
        let event = inner
            .events
            .get_mut(event_name)
            .ok_or_else(|| StoreError::NotFound(format!("evento {event_name}")))?;
        let contest = event
            .contests
            .get_mut(contest_name)
            .ok_or_else(|| StoreError::NotFound(format!("contest {contest_name}")))?;
        let entry = contest
            .sites
            .get_mut(site_name)
            .ok_or_else(|| StoreError::NotFound(format!("site {site_name}")))?;
        check_name(&config.name, site_name, "site")?;
        let codes = compile_codes(&config.codes)?;
        let mut config = config;
        config.name = site_name.to_string();
        entry.config = config;
        entry.codes = codes;
        Ok(())
    }

    pub async fn delete_site(&self, event_name: &str, contest_name: &str, site_name: &str) -> bool {
        let mut inner = self.inner.write().await;
        let Some(event) = inner.events.get_mut(event_name) else {
            return false;
        };
        let Some(contest) = event.contests.get_mut(contest_name) else {
            return false;
        };
        contest.sites.remove(site_name).is_some()
    }

    /// Replaces the site salt; a missing/empty salt generates a random one.
    pub async fn set_site_salt(
        &self,
        event_name: &str,
        contest_name: &str,
        site_name: &str,
        salt: Option<String>,
    ) -> Result<String, StoreError> {
        let mut inner = self.inner.write().await;
        let event = inner
            .events
            .get_mut(event_name)
            .ok_or_else(|| StoreError::NotFound(format!("evento {event_name}")))?;
        let contest = event
            .contests
            .get_mut(contest_name)
            .ok_or_else(|| StoreError::NotFound(format!("contest {contest_name}")))?;
        let entry = contest
            .sites
            .get_mut(site_name)
            .ok_or_else(|| StoreError::NotFound(format!("site {site_name}")))?;
        let salt = salt.filter(|s| !s.is_empty()).unwrap_or_else(generate_salt);
        entry.config.salt = Some(salt.clone());
        Ok(salt)
    }

    /// Snapshot every site's current revelation URL under one read lock.
    pub async fn revelation_urls(
        &self,
        event_name: &str,
        public_url: &url::Url,
    ) -> Option<Vec<data::event::RevelationUrl>> {
        let inner = self.inner.read().await;
        let event = inner.events.get(event_name)?;
        let mut urls = Vec::new();
        for (contest_name, contest) in &event.contests {
            let scoreboard =
                crate::revelation::scoreboard_url(public_url, event_name, contest_name);
            for (site_name, site) in &contest.sites {
                let key = deployment_site_key(
                    &self.revelation_salt,
                    event_name,
                    contest_name,
                    site_name,
                    event.salt.as_deref().unwrap_or_default(),
                    contest.config.salt.as_deref().unwrap_or_default(),
                    site.config.salt.as_deref().unwrap_or_default(),
                );
                urls.push(data::event::RevelationUrl {
                    contest: contest_name.clone(),
                    site: site_name.clone(),
                    url: crate::revelation::revelation_url(&scoreboard, site_name, &key).into(),
                });
            }
        }
        urls.sort_by(|a, b| (&a.contest, &a.site).cmp(&(&b.contest, &b.site)));
        Some(urls)
    }

    /// The site whose derived key matches `key`, if any.
    pub async fn site_by_key(
        &self,
        event_name: &str,
        contest_name: &str,
        key: &str,
    ) -> Option<(String, SiteConfig)> {
        let inner = self.inner.read().await;
        let event = inner.events.get(event_name)?;
        let contest = event.contests.get(contest_name)?;
        for (site_name, entry) in &contest.sites {
            let derived = deployment_site_key(
                &self.revelation_salt,
                event_name,
                contest_name,
                site_name,
                event.salt.as_deref().unwrap_or_default(),
                contest.config.salt.as_deref().unwrap_or_default(),
                entry.config.salt.as_deref().unwrap_or_default(),
            );
            if derived == key {
                return Some((site_name.clone(), entry.config.clone()));
            }
        }
        None
    }

    /// Public state of a contest; `problems` is omitted before the start.
    pub async fn public_state(
        &self,
        event_name: &str,
        contest_name: &str,
    ) -> Option<PublicContestState> {
        let inner = self.inner.read().await;
        let event = inner.events.get(event_name)?;
        let contest = event.contests.get(contest_name)?;
        let started = event.time_seconds >= 0;
        Some(PublicContestState {
            event: event.name.clone(),
            contest: contest_name.to_string(),
            problems: started.then(|| event.problems.clone()),
            teams: event
                .teams
                .iter()
                .filter(|team| contest.codes.is_match(&team.login))
                .cloned()
                .collect(),
            time_seconds: event.time_seconds,
            score_freeze_time_seconds: event.score_freeze_time_seconds,
            penalty_seconds: event.penalty_seconds,
        })
    }

    /// Public config of a contest (no salts anywhere).
    pub async fn public_config(
        &self,
        event_name: &str,
        contest_name: &str,
    ) -> Option<PublicConfig> {
        let inner = self.inner.read().await;
        let contest = inner.events.get(event_name)?.contests.get(contest_name)?;
        Some(PublicConfig {
            name: contest.config.name.clone(),
            codes: contest.config.codes.clone(),
            style: contest.config.style.clone(),
            ouro: contest.config.ouro,
            prata: contest.config.prata,
            bronze: contest.config.bronze,
            sites: contest
                .sites
                .values()
                .map(|entry| PublicSiteView {
                    name: entry.config.name.clone(),
                    codes: entry.config.codes.clone(),
                })
                .collect(),
            photo_url_format: contest.config.photo_url_format.clone(),
            sound_url_format: contest.config.sound_url_format.clone(),
        })
    }

    /// All runs of the teams of a contest, in arrival order.
    pub async fn contest_runs(&self, event_name: &str, contest_name: &str) -> Option<Vec<Run>> {
        let inner = self.inner.read().await;
        let event = inner.events.get(event_name)?;
        let contest = event.contests.get(contest_name)?;
        Some(
            event
                .runs
                .iter()
                .filter(|run| contest.codes.is_match(&run.team_login))
                .cloned()
                .collect(),
        )
    }

    /// All runs of the teams of a site, in arrival order.
    pub async fn site_runs(
        &self,
        event_name: &str,
        contest_name: &str,
        site_name: &str,
    ) -> Option<Vec<Run>> {
        let inner = self.inner.read().await;
        let event = inner.events.get(event_name)?;
        let contest = event.contests.get(contest_name)?;
        let site = contest.sites.get(site_name)?;
        Some(
            event
                .runs
                .iter()
                .filter(|run| site.codes.is_match(&run.team_login))
                .cloned()
                .collect(),
        )
    }

    /// The compiled team codes of a contest, for filtering streams.
    pub async fn contest_codes(&self, event_name: &str, contest_name: &str) -> Option<RegexSet> {
        let inner = self.inner.read().await;
        Some(
            inner
                .events
                .get(event_name)?
                .contests
                .get(contest_name)?
                .codes
                .clone(),
        )
    }

    /// Subscribes to the run stream of an event (replays since event creation).
    pub async fn subscribe_runs(&self, event_name: &str) -> Option<membroadcast::Receiver<Run>> {
        let inner = self.inner.read().await;
        Some(inner.events.get(event_name)?.runs_tx.subscribe())
    }

    /// Subscribes to the timer of an event.
    pub async fn subscribe_timer(
        &self,
        event_name: &str,
    ) -> Option<broadcast::Receiver<PublicTimer>> {
        let inner = self.inner.read().await;
        Some(inner.events.get(event_name)?.timer_tx.subscribe())
    }

    /// The current timer of an event.
    pub async fn current_timer(&self, event_name: &str) -> Option<PublicTimer> {
        let inner = self.inner.read().await;
        inner.events.get(event_name).map(Event::timer)
    }

    /// The broadcast channel of a remote-control key, creating it if needed.
    pub async fn remote_control_sender(
        &self,
        event_name: &str,
        contest_name: &str,
        key: &str,
    ) -> Option<ControlSender> {
        let mut inner = self.inner.write().await;
        let contest = inner
            .events
            .get_mut(event_name)?
            .contests
            .get_mut(contest_name)?;
        Some(
            contest
                .remote_control
                .entry(key.to_string())
                .or_insert_with(create_remote_control)
                .clone(),
        )
    }

    /// Whether the server holds an event with this name.
    pub async fn has_event(&self, event_name: &str) -> bool {
        let inner = self.inner.read().await;
        inner.events.contains_key(event_name)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts a legacy `ContestState` (webcast/BOCA shape) into the new model:
/// the event state and the full list of runs. Used by the compat feeders.
///
/// The legacy shape has no problem letters, so `A`, `B`, ... are generated
/// from `number_problems`.
pub fn from_legacy_contest_state(
    state: &crate::contest_state::ContestState,
    event_name: &str,
) -> (EventState, Vec<Run>) {
    let problems: Vec<String> = (0..state.contest.number_problems)
        .map(|i| char::from(b'A' + i as u8).to_string())
        .collect();
    let teams: Vec<TeamInfo> = state
        .contest
        .teams
        .values()
        .map(|team| TeamInfo {
            login: team.login.clone(),
            escola: team.escola.clone(),
            nome: team.name.clone(),
        })
        .collect();

    let event = EventState {
        name: event_name.to_string(),
        problems,
        teams,
        score_freeze_time_seconds: state.contest.score_freeze_time,
        penalty_seconds: state.contest.penalty_per_wrong_answer,
        time_seconds: state.time,
        salt: None,
    };
    let runs = state
        .runs
        .iter()
        .map(|run| Run {
            id: run.id,
            team_login: run.team_login.clone(),
            prob: run.prob.to_string(),
            time_seconds: run.time,
            answer: match run.answer {
                data::Answer::Yes { .. } => Answer::Yes,
                data::Answer::No { .. } => Answer::No,
                data::Answer::Wait { .. } => Answer::Unknown,
                data::Answer::Unk { .. } => Answer::Halt,
            },
        })
        .collect();

    (event, runs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base62_encodes_known_values() {
        assert_eq!(base62(&[0]), "0");
        assert_eq!(base62(&[1]), "1");
        assert_eq!(base62(&[61]), "z");
        assert_eq!(base62(&[62]), "10");
        assert_eq!(base62(&[63]), "11");
    }

    #[test]
    fn base62_encodes_full_digest() {
        // Regression: a 32-byte digest keeps the byte array at length 32,
        // and the division loop used to spin forever on it.
        let digest = hmac_sha256("chave", "mensagem");
        let encoded = base62(&digest);
        assert!(!encoded.is_empty());
        assert_eq!(base62(&digest), encoded);
    }

    #[test]
    fn site_key_matches_documented_derivation() {
        // Stable across runs: HMAC-SHA256 + base62 must be deterministic.
        let key = site_key(
            Some("evento"),
            Some("contest"),
            Some("site"),
            "brasil",
            "fiemg",
        );
        let again = site_key(
            Some("evento"),
            Some("contest"),
            Some("site"),
            "brasil",
            "fiemg",
        );
        assert_eq!(key, again);
        let key = key.expect("site has a salt");
        assert_eq!(key.len(), KEY_LEN);
        assert!(key.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn site_key_changes_when_any_salt_changes() {
        let base = site_key(Some("a"), Some("b"), Some("c"), "c", "s").unwrap();
        assert_ne!(
            base,
            site_key(Some("a2"), Some("b"), Some("c"), "c", "s").unwrap()
        );
        assert_ne!(
            base,
            site_key(Some("a"), Some("b2"), Some("c"), "c", "s").unwrap()
        );
        assert_ne!(
            base,
            site_key(Some("a"), Some("b"), Some("c2"), "c", "s").unwrap()
        );
        assert_ne!(
            base,
            site_key(Some("a"), Some("b"), Some("c"), "c2", "s").unwrap()
        );
        assert_ne!(
            base,
            site_key(Some("a"), Some("b"), Some("c"), "c", "s2").unwrap()
        );
    }

    #[test]
    fn site_without_salt_still_has_key() {
        assert!(site_key(Some("a"), Some("b"), None, "c", "s").is_some());
    }

    fn event_state(name: &str) -> EventState {
        EventState {
            name: name.to_string(),
            problems: vec!["A".to_string(), "B".to_string()],
            teams: vec![TeamInfo {
                login: "teambr001".to_string(),
                escola: "FACOM - UFMS".to_string(),
                nome: "Time de Teste".to_string(),
            }],
            score_freeze_time_seconds: 2040,
            penalty_seconds: 1200,
            time_seconds: -60,
            salt: None,
        }
    }

    #[tokio::test]
    async fn event_lifecycle() {
        let store = Engine::new();
        store
            .create_event("ensaio", event_state("ensaio"))
            .await
            .unwrap();
        assert_eq!(store.list_events().await, vec!["ensaio"]);

        // Duplicate creation conflicts.
        assert!(matches!(
            store.create_event("ensaio", event_state("ensaio")).await,
            Err(StoreError::AlreadyExists(_))
        ));

        // Name mismatch is rejected.
        assert!(matches!(
            store.create_event("outro", event_state("ensaio")).await,
            Err(StoreError::InvalidValue(_))
        ));

        store.patch_time("ensaio", 10).await.unwrap();
        assert_eq!(store.get_event("ensaio").await.unwrap().time_seconds, 10);

        assert!(store.delete_event("ensaio").await);
        assert!(!store.delete_event("ensaio").await);
        assert!(store.get_event("ensaio").await.is_none());
    }

    #[tokio::test]
    async fn runs_apply_in_order_and_correct() {
        let store = Engine::new();
        store
            .create_event("ensaio", event_state("ensaio"))
            .await
            .unwrap();

        let run = |id, answer| Run {
            id,
            team_login: "teambr001".to_string(),
            prob: "A".to_string(),
            time_seconds: 100,
            answer,
        };

        let (added, updated, ignored) = store
            .add_runs("ensaio", vec![run(1, Answer::No)])
            .await
            .unwrap();
        assert_eq!((added, updated), (1, 0));
        assert!(ignored.is_empty());

        // Same id corrects the previous result.
        let (added, updated, ignored) = store
            .add_runs("ensaio", vec![run(1, Answer::Yes)])
            .await
            .unwrap();
        assert_eq!((added, updated), (0, 1));
        assert!(ignored.is_empty());

        let runs = store.contest_runs("ensaio", "").await;
        // No contest "" yet: contest_runs returns None.
        assert!(runs.is_none());

        // Unknown teams are ignored (reported); unknown probs are rejected.
        let bad_team = Run {
            id: 2,
            team_login: "desconhecido".to_string(),
            prob: "A".to_string(),
            time_seconds: 100,
            answer: Answer::Yes,
        };
        let (added, updated, ignored) = store.add_runs("ensaio", vec![bad_team]).await.unwrap();
        assert_eq!((added, updated), (0, 0));
        assert_eq!(ignored.len(), 1);

        let bad_prob = Run {
            id: 2,
            team_login: "teambr001".to_string(),
            prob: "Z".to_string(),
            time_seconds: 100,
            answer: Answer::Yes,
        };
        assert!(matches!(
            store.add_runs("ensaio", vec![bad_prob]).await,
            Err(StoreError::InvalidValue(_))
        ));
    }

    #[tokio::test]
    async fn contests_sites_and_salts() {
        let store = Engine::new();
        store
            .create_event("ensaio", event_state("ensaio"))
            .await
            .unwrap();

        let contest = ContestConfig {
            name: "brasil".to_string(),
            codes: vec!["teambr".to_string()],
            salt: None,
            style: None,
            ouro: 4,
            prata: 8,
            bronze: 12,
            photo_url_format: None,
            sound_url_format: None,
        };
        store
            .create_contest("ensaio", "brasil", contest)
            .await
            .unwrap();
        assert!(store.get_contest("ensaio", "brasil").await.is_some());

        // Empty contest names are rejected.
        let empty = ContestConfig {
            name: String::new(),
            codes: vec!["teambr".to_string()],
            salt: None,
            style: None,
            ouro: 1,
            prata: 2,
            bronze: 3,
            photo_url_format: None,
            sound_url_format: None,
        };
        assert!(matches!(
            store.create_contest("ensaio", "vazio", empty).await,
            Err(StoreError::InvalidValue(_))
        ));

        // Invalid regex is rejected.
        let bad = ContestConfig {
            name: "ruim".to_string(),
            codes: vec!["(".to_string()],
            ..store.get_contest("ensaio", "brasil").await.unwrap()
        };
        assert!(matches!(
            store.create_contest("ensaio", "ruim", bad).await,
            Err(StoreError::InvalidRegex(_))
        ));

        let site = SiteConfig {
            name: "fiemg".to_string(),
            codes: vec!["teambr".to_string()],
            salt: None,
        };
        store
            .create_site("ensaio", "brasil", "fiemg", site)
            .await
            .unwrap();

        // A site without its own salt has no key.
        assert!(
            store
                .site_by_key("ensaio", "brasil", "qualquer")
                .await
                .is_none()
        );

        store
            .set_event_salt("ensaio", Some("e".into()))
            .await
            .unwrap();
        store
            .set_contest_salt("ensaio", "brasil", Some("c".into()))
            .await
            .unwrap();
        store
            .set_site_salt("ensaio", "brasil", "fiemg", Some("s".into()))
            .await
            .unwrap();

        let legacy =
            site_key_with_event(Some("e"), Some("c"), Some("s"), "ensaio", "brasil", "fiemg")
                .unwrap();
        assert!(
            store
                .site_by_key("ensaio", "brasil", &legacy)
                .await
                .is_none()
        );
        let legacy = site_key(Some("e"), Some("c"), Some("s"), "brasil", "fiemg").unwrap();
        assert!(
            store
                .site_by_key("ensaio", "brasil", &legacy)
                .await
                .is_none()
        );
        let key = deployment_site_key(
            &store.revelation_salt,
            "ensaio",
            "brasil",
            "fiemg",
            "e",
            "c",
            "s",
        );
        let found = store.site_by_key("ensaio", "brasil", &key).await;
        assert_eq!(found.map(|(name, _)| name).as_deref(), Some("fiemg"));

        // Rotating the site salt changes only that site's key.
        store
            .set_site_salt("ensaio", "brasil", "fiemg", Some("s2".into()))
            .await
            .unwrap();
        assert!(store.site_by_key("ensaio", "brasil", &key).await.is_none());
        let new_key = deployment_site_key(
            &store.revelation_salt,
            "ensaio",
            "brasil",
            "fiemg",
            "e",
            "c",
            "s2",
        );
        assert!(
            store
                .site_by_key("ensaio", "brasil", &new_key)
                .await
                .is_some()
        );

        // Contest deletion removes its sites.
        assert!(store.delete_contest("ensaio", "brasil").await);
        assert!(store.get_site("ensaio", "brasil", "fiemg").await.is_none());
    }

    #[tokio::test]
    async fn public_state_hides_problems_before_start() {
        let store = Engine::new();
        store
            .create_event("ensaio", event_state("ensaio"))
            .await
            .unwrap();
        let contest = ContestConfig {
            name: "brasil".to_string(),
            codes: vec!["teambr".to_string()],
            salt: None,
            style: None,
            ouro: 1,
            prata: 2,
            bronze: 3,
            photo_url_format: None,
            sound_url_format: None,
        };
        store
            .create_contest("ensaio", "brasil", contest)
            .await
            .unwrap();

        // event_state sets time_seconds = -60 (countdown).
        let state = store.public_state("ensaio", "brasil").await.unwrap();
        assert_eq!(state.problems, None);
        assert_eq!(state.teams.len(), 1);

        store.patch_time("ensaio", 0).await;
        let state = store.public_state("ensaio", "brasil").await.unwrap();
        assert_eq!(
            state.problems.as_deref(),
            Some(&["A".to_string(), "B".to_string()][..])
        );
    }
}

#[cfg(test)]
mod deployment_key_tests {
    use super::deployment_site_key as key;
    #[test]
    fn fixed_vectors_and_domains() {
        assert_eq!(key("private", "e", "c", "s", "ev", "", ""), "bhTCntI0yu9R");
        assert_eq!(
            key("private", "e:c", "c", "s:ü", "ev", "c", "s"),
            "ZDvT1wW3ao4y"
        );
        let base = key("private", "e", "c", "s", "ev", "", "");
        for changed in [
            key("other", "e", "c", "s", "ev", "", ""),
            key("private", "e2", "c", "s", "ev", "", ""),
            key("private", "e", "c2", "s", "ev", "", ""),
            key("private", "e", "c", "s2", "ev", "", ""),
            key("private", "e", "c", "s", "ev2", "", ""),
            key("private", "e", "c", "s", "ev", "c", ""),
            key("private", "e", "c", "s", "ev", "", "s"),
        ] {
            assert_ne!(base, changed);
        }
        assert_ne!(
            key("private", "a:b", "c", "s", "", "", ""),
            key("private", "a", "b:c", "s", "", "", "")
        );
    }
}

impl Engine {
    pub(super) async fn snapshot(&self, name: &str) -> Option<crate::database::StoredEvent> {
        let inner = self.inner.read().await;
        let e = inner.events.get(name)?;
        Some(crate::database::StoredEvent {
            state: incremental::state(e),
            contests: e
                .contests
                .iter()
                .map(|(name, c)| {
                    (
                        name.clone(),
                        crate::database::StoredContest {
                            config: c.config.clone(),
                            sites: c
                                .sites
                                .iter()
                                .map(|(name, s)| (name.clone(), s.config.clone()))
                                .collect(),
                        },
                    )
                })
                .collect(),
            runs: e.runs.clone(),
        })
    }

    /// Install an authoritative snapshot. New channels are seeded with current
    /// runs, never historical corrections. Existing channels see committed diffs.
    pub(super) async fn install(
        &self,
        name: &str,
        snapshot: Option<crate::database::StoredEvent>,
    ) -> Result<(), StoreError> {
        let mut inner = self.inner.write().await;
        let Some(snapshot) = snapshot else {
            inner.events.remove(name);
            inner.order.retain(|n| n != name);
            return Ok(());
        };
        if snapshot.state.name != name {
            return Err(StoreError::Storage(
                crate::database::DatabaseError::Corrupt("event identifier mismatch".into()),
            ));
        }
        let mut contests = HashMap::new();
        for (key, c) in snapshot.contests {
            if key != c.config.name || c.sites.iter().any(|(k, s)| k != &s.name) {
                return Err(StoreError::Storage(
                    crate::database::DatabaseError::Corrupt("resource identifier mismatch".into()),
                ));
            }
            let sites = c
                .sites
                .into_iter()
                .map(|(k, config)| {
                    Ok((
                        k,
                        SiteEntry {
                            codes: compile_codes(&config.codes)?,
                            config,
                        },
                    ))
                })
                .collect::<Result<_, StoreError>>()?;
            contests.insert(
                key,
                ContestEntry {
                    codes: compile_codes(&c.config.codes)?,
                    config: c.config,
                    sites,
                    remote_control: HashMap::new(),
                },
            );
        }
        let (runs_tx, timer_tx) = if let Some(old) = inner.events.get(name) {
            let next_ids: std::collections::HashSet<_> =
                snapshot.runs.iter().map(|r| r.id).collect();
            let removed = old.runs.iter().any(|r| !next_ids.contains(&r.id));
            let runs_tx = if removed {
                membroadcast::channel(self.channel_capacity).0
            } else {
                old.runs_tx.clone()
            };
            (runs_tx, old.timer_tx.clone())
        } else {
            (
                membroadcast::channel(self.channel_capacity).0,
                broadcast::channel(self.channel_capacity).0,
            )
        };
        let state = snapshot.state;
        let mut runs = snapshot.runs;
        runs.sort_by_key(|r| (r.time_seconds, r.id));
        let runs_index: HashMap<_, _> = runs.iter().enumerate().map(|(i, r)| (r.id, i)).collect();
        if runs_index.len() != runs.len() {
            return Err(StoreError::Storage(
                crate::database::DatabaseError::Corrupt("duplicate run IDs".into()),
            ));
        }
        let mut next = Event {
            name: state.name,
            problems: state.problems,
            teams: state.teams,
            score_freeze_time_seconds: state.score_freeze_time_seconds,
            penalty_seconds: state.penalty_seconds,
            time_seconds: state.time_seconds,
            salt: state.salt,
            contests,
            runs,
            runs_index,
            runs_tx,
            timer_tx,
        };
        if let Some(old) = inner.events.get_mut(name) {
            next.timer_tx = old.timer_tx.clone();
            if next.timer() != old.timer() {
                next.publish_timer();
            }
            let removed_runs = old
                .runs
                .iter()
                .any(|r| !next.runs_index.contains_key(&r.id));
            if !removed_runs {
                next.runs_tx = old.runs_tx.clone();
                for run in &next.runs {
                    if old.runs_index.get(&run.id).map(|i| &old.runs[*i]) != Some(run) {
                        next.runs_tx.send_memo(run.clone());
                    }
                }
            } else {
                for run in &next.runs {
                    next.runs_tx.send_memo(run.clone());
                }
            }
            for (key, c) in &mut next.contests {
                if let Some(previous) = old.contests.get_mut(key) {
                    c.remote_control = std::mem::take(&mut previous.remote_control);
                }
            }
        } else {
            inner.order.push(name.to_string());
            for run in &next.runs {
                next.runs_tx.send_memo(run.clone());
            }
        }
        next.runs_tx.replace_memo(next.runs.clone());
        inner.events.insert(name.to_string(), next);
        Ok(())
    }
}
