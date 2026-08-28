//! Wire types of the event API (`doc/event-api.md` and `doc/public-api.md`).
//!
//! Shared by the server and the client: this crate is wasm-safe.

use serde::{Deserialize, Serialize};

fn one() -> usize {
    1
}
fn two() -> usize {
    2
}
fn three() -> usize {
    3
}

/// A team of the event, as described in `doc/event-api.md`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamInfo {
    pub login: String,
    pub escola: String,
    pub nome: String,
}

/// The full state of an event, as accepted and returned by `/internal`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventState {
    pub name: String,
    pub problems: Vec<String>,
    pub teams: Vec<TeamInfo>,
    pub score_freeze_time_seconds: i64,
    pub penalty_seconds: i64,
    #[serde(default)]
    pub time_seconds: i64,
    #[serde(default)]
    pub salt: Option<String>,
}

/// A contest of an event, as accepted and returned by `/internal`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContestConfig {
    pub name: String,
    pub codes: Vec<String>,
    #[serde(default)]
    pub salt: Option<String>,
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default = "one")]
    pub ouro: usize,
    #[serde(default = "two")]
    pub prata: usize,
    #[serde(default = "three")]
    pub bronze: usize,
    #[serde(default)]
    pub photo_url_format: Option<String>,
    #[serde(default)]
    pub sound_url_format: Option<String>,
}

/// A site of a contest, as accepted and returned by `/internal`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SiteConfig {
    pub name: String,
    pub codes: Vec<String>,
    #[serde(default)]
    pub salt: Option<String>,
}

/// The result of a submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Answer {
    #[serde(rename = "Y")]
    Yes,
    #[serde(rename = "N")]
    No,
    #[serde(rename = "?")]
    Unknown,
    #[serde(rename = "X")]
    Halt,
}

/// A run, as described in `doc/event-api.md`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Run {
    pub id: i64,
    pub team_login: String,
    pub prob: String,
    pub time_seconds: i64,
    pub answer: Answer,
}

/// The timer message sent over the public timer WebSocket.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublicTimer {
    pub current_time_seconds: i64,
    pub score_freeze_time_seconds: i64,
}

/// The public state of a contest, served by `/api`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublicContestState {
    pub event: String,
    pub contest: String,
    /// Omitted while the contest has not started (`time_seconds < 0`).
    pub problems: Option<Vec<String>>,
    pub teams: Vec<TeamInfo>,
    pub time_seconds: i64,
    pub score_freeze_time_seconds: i64,
    pub penalty_seconds: i64,
}

/// A site as exposed by the public config endpoint (no salt).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublicSiteView {
    pub name: String,
    pub codes: Vec<String>,
}

/// The public config of a contest, served by `/api` (no salts).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublicConfig {
    pub name: String,
    pub codes: Vec<String>,
    pub style: Option<String>,
    pub ouro: usize,
    pub prata: usize,
    pub bronze: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sites: Vec<PublicSiteView>,
    pub photo_url_format: Option<String>,
    pub sound_url_format: Option<String>,
}

/// The runs of a site, as returned by the public secret endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RunsData {
    pub runs: Vec<Run>,
}

/// The `{ data, errors, warnings }` response envelope; fields are optional
/// and absent when empty.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Envelope<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<ErrorEntry>>,
    pub warnings: Option<Vec<ErrorEntry>>,
}

/// One entry of `errors`/`warnings` in the envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorEntry {
    pub code: String,
    pub message: String,
}
