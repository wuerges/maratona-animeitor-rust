//! Operator-facing commands; each produces one internal API request.
use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "animeitor-admin",
    version,
    about = "Manage Animeitor events through the internal HTTPS API"
)]
pub struct AdminArgs {
    /// Server configuration; relative paths inside it are resolved from this file.
    #[arg(long, global = true, default_value = "server.toml")]
    pub server_config: PathBuf,
    /// Emit API JSON envelopes for automation (errors go to stderr).
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Command,
}
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create, inspect, update, replace, or delete events.
    #[command(subcommand)]
    Events(EventCommand),
    /// Manage scoreboard group configurations within an event.
    #[command(subcommand)]
    Contests(ContestCommand),
    /// Manage site configurations within a contest.
    #[command(subcommand)]
    Sites(SiteCommand),
    /// Manage event roster entries; referenced deletions require --keep-runs.
    #[command(subcommand)]
    Teams(TeamCommand),
    /// Manage the ordered event problem list.
    #[command(subcommand)]
    Problems(ProblemCommand),
    /// Add/correct submissions, import batches, or clear stored runs.
    #[command(subcommand)]
    Runs(RunCommand),
    /// Set elapsed event time once, including negative countdowns.
    #[command(subcommand)]
    Timer(TimerCommand),
    /// List private frontend links for every site of an event.
    RevelationUrls(EventId),
    /// Read process-wide Prometheus metrics.
    Metrics,
}
#[derive(Debug, Args)]
pub struct EventId {
    pub event: String,
}
#[derive(Debug, Args)]
pub struct ContestId {
    pub event: String,
    pub contest: String,
}
#[derive(Debug, Args)]
pub struct SiteId {
    pub event: String,
    pub contest: String,
    pub site: String,
}
#[derive(Debug, Args)]
pub struct TeamId {
    pub event: String,
    pub login: String,
}
#[derive(Debug, Args, Default)]
pub struct Input {
    /// Bare resource JSON file, or - for stdin; mutually exclusive with field flags.
    #[arg(long)]
    pub file: Option<PathBuf>,
}
#[derive(Debug, Args)]
pub struct FileInput {
    /// JSON file, or - for stdin. Requests do not use an envelope.
    #[arg(long)]
    pub file: PathBuf,
}
#[derive(Debug, Args, Default)]
pub struct ClearFields {
    /// Clear a nullable field (salt, style, photo_url_format, sound_url_format as applicable).
    #[arg(long)]
    pub unset: Vec<String>,
}
#[derive(Debug, Args, Default)]
pub struct CodeFlags {
    /// Append this exact regex pattern; repeat for multiple patterns.
    #[arg(long)]
    pub add: Vec<String>,
    /// Remove this exact regex pattern; repeat for multiple patterns.
    #[arg(long)]
    pub remove: Vec<String>,
}

#[derive(Debug, Args, Serialize, Default)]
pub struct EventFields {
    #[arg(long)]
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "problems")]
    /// Problem identifier; repeat to supply an ordered list.
    pub problem: Vec<String>,
    #[arg(long)]
    #[serde(skip)]
    /// Roster JSON array file, or - for stdin.
    pub teams_file: Option<PathBuf>,
    #[arg(long, allow_negative_numbers = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Inclusive freeze boundary in elapsed seconds.
    pub score_freeze_time_seconds: Option<i64>,
    #[arg(long, allow_negative_numbers = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Penalty for an incorrect submission, in seconds.
    pub penalty_seconds: Option<i64>,
    #[arg(long, allow_negative_numbers = true)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Elapsed seconds; negative values keep contest details closed.
    pub time_seconds: Option<i64>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<String>,
}

#[derive(Debug, Args, Serialize, Default)]
pub struct ContestFields {
    #[arg(long)]
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "codes")]
    /// Team-login regex; repeat for OR matching. Arrays replace all filters on update.
    pub code: Vec<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none", rename = "ouro")]
    /// Inclusive gold placement threshold (API ouro).
    pub gold: Option<usize>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none", rename = "prata")]
    /// Inclusive silver placement threshold (API prata).
    pub silver: Option<usize>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none", rename = "bronze")]
    /// Inclusive bronze placement threshold (API bronze).
    pub bronze: Option<usize>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Photo URL template containing {team_login}.
    pub photo_url_format: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Audio URL template containing {team_login}.
    pub sound_url_format: Option<String>,
}

#[derive(Debug, Args, Serialize, Default)]
pub struct SiteFields {
    #[arg(long)]
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "codes")]
    /// Team-login regex; repeat for OR matching. Arrays replace all filters on update.
    pub code: Vec<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<String>,
}

#[derive(Debug, Args, Serialize, Default)]
pub struct TeamFields {
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escola: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nome: Option<String>,
}

#[derive(Debug, Args, Serialize, Default)]
pub struct TeamChanges {
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escola: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nome: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum EventCommand {
    List,
    Get(EventId),
    /// Create a resource; required fields follow the API schema.
    Create {
        #[command(flatten)]
        id: EventId,
        #[command(flatten)]
        input: Input,
        #[command(flatten)]
        fields: EventFields,
    },
    /// Atomically change only supplied fields; arrays replace the entire list.
    Update {
        #[command(flatten)]
        id: EventId,
        #[command(flatten)]
        input: Input,
        #[command(flatten)]
        fields: EventFields,
        #[command(flatten)]
        clear: ClearFields,
        /// Allow removing teams while retaining their runs; never applies to problems.
        #[arg(long)]
        keep_runs: bool,
    },
    /// Full replacement: omitted optional fields reset to API defaults.
    Replace {
        #[command(flatten)]
        id: EventId,
        #[command(flatten)]
        input: FileInput,
    },
    /// Delete this resource and its children; no interactive confirmation.
    Delete(EventId),
    /// Set or generate a salt; existing revelation links for this scope may change.
    Salt {
        #[command(flatten)]
        id: EventId,
        #[arg(long)]
        salt: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ContestCommand {
    List(EventId),
    Get(ContestId),
    /// Create a resource; required fields follow the API schema.
    Create {
        #[command(flatten)]
        id: ContestId,
        #[command(flatten)]
        input: Input,
        #[command(flatten)]
        fields: ContestFields,
    },
    /// Atomically change only supplied fields; arrays replace the entire list.
    Update {
        #[command(flatten)]
        id: ContestId,
        #[command(flatten)]
        input: Input,
        #[command(flatten)]
        fields: ContestFields,
        #[command(flatten)]
        clear: ClearFields,
    },
    /// Full replacement: omitted optional fields reset to API defaults.
    Replace {
        #[command(flatten)]
        id: ContestId,
        #[command(flatten)]
        input: FileInput,
    },
    /// Delete this resource and its children; no interactive confirmation.
    Delete(ContestId),
    /// Set or generate a salt; existing revelation links for this scope may change.
    Salt {
        #[command(flatten)]
        id: ContestId,
        #[arg(long)]
        salt: Option<String>,
    },
    /// Atomically add/remove exact regex filter strings.
    Codes {
        #[command(flatten)]
        id: ContestId,
        #[command(flatten)]
        codes: CodeFlags,
    },
}

#[derive(Debug, Subcommand)]
pub enum SiteCommand {
    List(ContestId),
    Get(SiteId),
    /// Create a resource; required fields follow the API schema.
    Create {
        #[command(flatten)]
        id: SiteId,
        #[command(flatten)]
        input: Input,
        #[command(flatten)]
        fields: SiteFields,
    },
    /// Atomically change only supplied fields; arrays replace the entire list.
    Update {
        #[command(flatten)]
        id: SiteId,
        #[command(flatten)]
        input: Input,
        #[command(flatten)]
        fields: SiteFields,
        #[command(flatten)]
        clear: ClearFields,
    },
    /// Full replacement: omitted optional fields reset to API defaults.
    Replace {
        #[command(flatten)]
        id: SiteId,
        #[command(flatten)]
        input: FileInput,
    },
    /// Delete this resource and its children; no interactive confirmation.
    Delete(SiteId),
    /// Set or generate a salt; existing revelation links for this scope may change.
    Salt {
        #[command(flatten)]
        id: SiteId,
        #[arg(long)]
        salt: Option<String>,
    },
    /// Atomically add/remove exact regex filter strings.
    Codes {
        #[command(flatten)]
        id: SiteId,
        #[command(flatten)]
        codes: CodeFlags,
    },
}

#[derive(Debug, Subcommand)]
pub enum TeamCommand {
    List(EventId),
    Get(TeamId),
    Add {
        #[command(flatten)]
        id: EventId,
        #[command(flatten)]
        input: Input,
        #[command(flatten)]
        fields: TeamFields,
    },
    Update {
        #[command(flatten)]
        id: TeamId,
        #[command(flatten)]
        input: Input,
        #[command(flatten)]
        fields: TeamChanges,
    },
    /// Remove a team; referenced runs cause a conflict unless --keep-runs is supplied.
    Delete {
        #[command(flatten)]
        id: TeamId,
        #[arg(long)]
        keep_runs: bool,
    },
}
#[derive(Debug, Subcommand)]
pub enum ProblemCommand {
    List(EventId),
    Add {
        event: String,
        problem: String,
    },
    /// Removing a problem with stored runs returns a conflict.
    Delete {
        event: String,
        problem: String,
    },
}
#[derive(Debug, Subcommand)]
pub enum TimerCommand {
    /// Set once; the server does not advance the clock automatically.
    Set {
        event: String,
        #[arg(long, allow_negative_numbers = true)]
        seconds: i64,
    },
}
#[derive(Debug, Subcommand)]
pub enum RunCommand {
    /// Add a submission or correct an existing ID; identical resends are no-ops.
    Add {
        event: String,
        #[arg(long, allow_negative_numbers = true)]
        id: i64,
        #[arg(long)]
        team_login: String,
        #[arg(long)]
        problem: String,
        #[arg(long, allow_negative_numbers = true)]
        time_seconds: i64,
        #[arg(long, value_parser = ["Y", "N", "?", "X"])]
        answer: String,
    },
    /// Submit a JSON object containing a runs array; unknown teams produce warnings.
    Import {
        event: String,
        #[command(flatten)]
        input: FileInput,
    },
    /// Delete a single stored submission by ID.
    Delete {
        event: String,
        #[arg(allow_negative_numbers = true)]
        id: i64,
    },
    /// Delete stored runs. WebSocket replay history is retained; recreate the event for a clean history.
    Clear(EventId),
}
