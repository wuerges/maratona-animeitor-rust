pub mod configdata;
pub mod event;
pub mod remote_control;

use configdata::Sede;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet, btree_map};
use std::fmt::{self, Display};
use std::str::FromStr;
use std::sync::LazyLock;
use std::sync::atomic::AtomicU64;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq)]
/// The judge answer to a submission.
pub enum Answer {
    /// Accepted, with the time of the submission, and a bool that
    Yes {
        time: i64,
        is_first: bool,
        run_id: i64,
    },
    /// Rejected.
    No { run_id: i64 },
    /// Waiting to be judged.
    Wait { run_id: i64 },
    /// Unknown.
    Unk { run_id: i64 },
}

pub type TimeFile = i64;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
/// A problem in the scoreboard.
pub struct Problem {
    /// Was the problem solved?
    pub solved: bool,
    /// Was the problem solved first?
    pub solved_first: bool,
    /// How many submissions?
    pub submissions: usize,
    /// How much penalty in total?
    pub penalty: i64,
    /// When was it solved?
    pub time_solved: i64,
    /// What were the judges answers to this problem for this team?
    pub answers: Vec<Answer>,

    /// The run ids of the waits
    pub waits: HashSet<i64>,

    pub id: u64,
}

#[derive(Copy, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Timer state
pub struct TimerData {
    /// Current time.
    pub current_time: TimeFile,
    /// Scoreboard freeze time.
    pub score_freeze_time: TimeFile,
}

impl TimerData {
    pub fn new(current_time: TimeFile, score_freeze_time: TimeFile) -> Self {
        Self {
            current_time,
            score_freeze_time,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A team in the contest.
pub struct Team {
    /// BOCA's login.
    pub login: String,
    /// The school of the team.
    pub escola: String,
    /// The name of the team.
    pub name: String,
    /// Placement in the site.
    pub placement: usize,
    /// Global placement across all sites.
    pub placement_global: usize,
    /// State of the problems that the team is solving.
    pub problems: BTreeMap<Letter, Problem>,

    pub id: u64,
}

impl PartialEq for Team {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.id == other.id
    }
}

static SEED: AtomicU64 = AtomicU64::new(0);

/// Mints a fresh id for state mutations.
///
/// Every mutation of [`Team`] or [`Problem`] state (server-side dump
/// construction included) must bump this shared counter so ids never collide
/// between the initial contest state shipped by the server and the mutations
/// applied by clients. `client-model`'s change detection compares [`Team`]
/// by `name + id` and problem views by `id`, so ids must be strictly
/// increasing across the wire.
pub fn gen_id() -> u64 {
    SEED.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A contest serialized in the api response.
pub struct ContestFile {
    /// Name of the contest.
    pub contest_name: String,
    /// Map of the teams.
    pub teams: BTreeMap<String, Team>,
    /// Current contest time.
    pub current_time: i64,
    /// Maximum time (contest ends).
    pub maximum_time: i64,
    /// Time that score gets frozen.
    pub score_freeze_time: i64,
    /// Penalty per wrong answer.
    pub penalty_per_wrong_answer: i64,
    /// Number of problems in the contest.
    pub number_problems: usize,
}

impl ContestFile {
    pub fn filter_sede(self, sede: &Sede) -> Self {
        Self {
            teams: self
                .teams
                .into_iter()
                .filter(|(login, _t)| sede.team_belongs_str(login))
                .collect(),
            ..self
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Letter(String);

#[derive(Debug, Error)]
#[error("bad letter: {}", .0)]
pub struct BadLetter(String);

impl FromStr for Letter {
    type Err = BadLetter;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_empty() && s.chars().all(|c| ALPHABET.contains(&c)) {
            Ok(Letter(s.to_string()))
        } else {
            Err(BadLetter(s.to_string()))
        }
    }
}

impl Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialOrd for Letter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Letter {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.len().cmp(&other.0.len()) {
            Ordering::Equal => self.0.cmp(&other.0),
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        }
    }
}

static ALPHABET: LazyLock<Vec<char>> =
    LazyLock::new(|| "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect());

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
/// A submission being judged.
pub struct RunTuple {
    /// Id of submission.
    pub id: i64,
    /// Order in input.
    pub order: u64,
    /// Time of the submission.
    pub time: i64,
    /// The team login.
    pub team_login: String,
    /// The problem letter.
    pub prob: Letter,
    /// The answer for this submission.
    pub answer: Answer,
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for RunTuple {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.time.cmp(&other.time))
    }
}

impl Ord for RunTuple {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunsFile {
    runs: BTreeMap<i64, RunTuple>,
}

// `is_empty` was removed with the client-only helpers; `len` only serves
// server-side dump assertions.
#[allow(clippy::len_without_is_empty)]
impl RunsFile {
    pub fn empty() -> Self {
        Self {
            runs: BTreeMap::new(),
        }
    }

    /// Builds a runs file from a run list; later ids win.
    pub fn from_runs(runs: Vec<RunTuple>) -> Self {
        let mut map = BTreeMap::new();
        for run in runs {
            map.insert(run.id, run);
        }
        Self { runs: map }
    }

    pub fn len(&self) -> usize {
        self.runs.len()
    }

    pub fn sorted(&self) -> Vec<RunTuple> {
        let mut r: Vec<_> = self.runs.values().cloned().collect();
        r.sort_by_key(|r| (r.time, r.order));
        r
    }

    pub fn filter_teams(&mut self, contest: &ContestFile) {
        let runs = &mut self.runs;
        runs.retain(|&_, run| contest.teams.contains_key(&run.team_login));
    }

    pub fn refresh_1(&mut self, t: &RunTuple) -> bool {
        let ent = self.runs.entry(t.id);
        match ent {
            btree_map::Entry::Vacant(v) => {
                v.insert(t.clone());
                true
            }
            btree_map::Entry::Occupied(mut o) => {
                if o.get() != t {
                    *o.get_mut() = t.clone();
                    return true;
                }
                false
            }
        }
    }
}
