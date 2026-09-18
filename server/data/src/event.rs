//! Wire types of the event API (`doc/event-api.md` and `doc/public-api.md`).
//!
//! Shared by the server and the client: this crate is wasm-safe.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct TeamInfo {
    /// Unique team login used by runs and regex filters; for example teambr001.
    pub login: String,
    /// School or institution displayed for the team (Portuguese field name).
    pub escola: String,
    /// Team display name (Portuguese field name).
    pub nome: String,
}

/// The full state of an event, as accepted and returned by `/internal`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct EventState {
    /// Event identifier; must equal event_name in the request path.
    pub name: String,
    /// Problem identifiers in display order, e.g. ["A", "B"]. Run prob values must appear here.
    pub problems: Vec<String>,
    /// Complete team roster. Unknown team logins in run batches are ignored with warnings.
    pub teams: Vec<TeamInfo>,
    /// Elapsed event time at which public run answers become hidden, inclusive, in seconds.
    pub score_freeze_time_seconds: i64,
    /// Penalty per incorrect submission, in seconds; commonly 1200.
    pub penalty_seconds: i64,
    /// Caller-supplied elapsed seconds, default 0. Negative means pre-start countdown. The server does not advance this clock automatically.
    #[serde(default)]
    #[schema(default = 0, example = -60)]
    pub time_seconds: i64,
    /// Optional public input to site-key derivation. Omitted or null means empty input; changing it changes all event revelation keys.
    #[serde(default)]
    pub salt: Option<String>,
}

/// A contest of an event, as accepted and returned by `/internal`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ContestConfig {
    /// Nonempty contest identifier; must equal contest_name in the path.
    pub name: String,
    /// Rust regex patterns matched against team logins. Any matching pattern includes a team; patterns are unanchored unless explicitly anchored. Empty list matches no teams; [".*"] matches all.
    pub codes: Vec<String>,
    /// Optional derivation input. Changing it changes every site key in this contest.
    #[serde(default)]
    pub salt: Option<String>,
    /// Optional frontend visual style name. Null selects the frontend default.
    #[serde(default)]
    pub style: Option<String>,
    /// Last placement awarded gold (1-based, inclusive); default 1.
    #[serde(default = "one")]
    #[schema(default = 1)]
    pub ouro: usize,
    /// Last placement awarded silver (1-based, inclusive); default 2.
    #[serde(default = "two")]
    #[schema(default = 2)]
    pub prata: usize,
    /// Last placement awarded bronze (1-based, inclusive); default 3.
    #[serde(default = "three")]
    #[schema(default = 3)]
    pub bronze: usize,
    /// Optional team photo URL template with {team_login}. Default: photos/{team_login}.webp relative to the API origin.
    #[serde(default)]
    pub photo_url_format: Option<String>,
    /// Optional team audio URL template with {team_login}. Default: sounds/{team_login}.mp3 relative to the API origin.
    #[serde(default)]
    pub sound_url_format: Option<String>,
}

/// A site of a contest, as accepted and returned by `/internal`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct SiteConfig {
    /// Site identifier. Use the path site_name; an empty body name is accepted and normalized in storage.
    pub name: String,
    /// Rust regex patterns matched against event team logins (OR). Configure these to select a subset of the contest teams; the server does not enforce that subset for secret runs.
    pub codes: Vec<String>,
    /// Optional derivation input. Changing it changes only this site key.
    #[serde(default)]
    pub salt: Option<String>,
}

/// Submission result: Y = accepted, N = incorrect, ? = pending/unknown, X = halted/unknown. The frontend displays X as unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Run {
    /// Submission ID, scoped to the event. Resending changed fields corrects this submission; identical resends are no-ops.
    pub id: i64,
    /// Login from the event roster; unknown logins are skipped with unknown_team warnings.
    pub team_login: String,
    /// Problem identifier from the event problems list.
    pub prob: String,
    /// Submission time measured in seconds since event start.
    pub time_seconds: i64,
    /// Y = accepted; N = incorrect; ? = pending/unknown; X = halted/unknown (displayed as unknown by the client).
    pub answer: Answer,
}

/// The timer message sent over the public timer WebSocket.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PublicTimer {
    /// Last supplied event time in seconds; negative values indicate countdown.
    pub current_time_seconds: i64,
    /// Public answer freeze boundary in elapsed seconds, inclusive.
    pub score_freeze_time_seconds: i64,
}

/// The public state of a contest, served by `/api`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PublicContestState {
    /// Event identifier.
    pub event: String,
    /// Nonempty contest identifier.
    pub contest: String,
    /// Problem identifiers. Successful HTTP responses are only available after start, so this field contains the list.
    pub problems: Option<Vec<String>>,
    /// Event teams matching at least one contest codes regex.
    pub teams: Vec<TeamInfo>,
    /// Last supplied event time in seconds.
    pub time_seconds: i64,
    /// Inclusive public answer freeze boundary, in seconds.
    pub score_freeze_time_seconds: i64,
    /// Penalty for an incorrect submission, in seconds.
    pub penalty_seconds: i64,
}

/// A site as exposed by the public config endpoint (no salt).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PublicSiteView {
    /// Site identifier used by the frontend sede query parameter.
    pub name: String,
    /// Team-login regex filters for this site.
    pub codes: Vec<String>,
}

/// The public config of a contest, served by `/api` (no salts).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PublicConfig {
    /// Contest identifier.
    pub name: String,
    /// Contest team-login regex filters.
    pub codes: Vec<String>,
    /// Optional frontend style name; null when unset.
    pub style: Option<String>,
    /// Inclusive gold placement threshold.
    pub ouro: usize,
    /// Inclusive silver placement threshold.
    pub prata: usize,
    /// Inclusive bronze placement threshold.
    pub bronze: usize,
    /// Site choices; omitted when empty. No salts or revelation keys are included.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sites: Vec<PublicSiteView>,
    /// Photo template with {team_login}; null uses photos/{team_login}.webp at the API origin.
    pub photo_url_format: Option<String>,
    /// Audio template with {team_login}; null uses sounds/{team_login}.mp3 at the API origin.
    pub sound_url_format: Option<String>,
}

/// The runs of a site, as returned by the public secret endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, ToSchema)]
pub struct RunsData {
    /// Site submissions, including actual answers after the freeze boundary.
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ErrorEntry {
    /// Stable machine-readable error or warning code; use this rather than matching message text.
    pub code: String,
    /// Human-readable detail; messages may be in Portuguese.
    pub message: String,
}

/// A private frontend revelation link. Anyone holding the URL holds the site key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct RevelationUrl {
    /// Contest identifier within the requested event.
    pub contest: String,
    /// Site identifier within the contest.
    pub site: String,
    /// Absolute frontend URL. Its secret query parameter is also the runs_secret Bearer token.
    #[schema(
        format = "uri",
        example = "https://example.com/animeitor/regional-2026/brasil/?secret=EXAMPLE_KEY&sede=fiemg"
    )]
    pub url: String,
}
