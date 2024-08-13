pub mod annotate_first_solved;
pub mod configdata;
pub mod remote_control;
pub mod revelation;

use configdata::Sede;
use serde::{Deserialize, Serialize};
use std::cmp::{Eq, Ordering};
use std::collections::{btree_map, BTreeMap, HashSet};
use std::fmt;
use std::sync::atomic::AtomicU64;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, ToSchema)]
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

impl Answer {
    pub fn is_wait(&self) -> bool {
        match self {
            Answer::Wait { .. } => true,
            _ => false,
        }
    }
}

#[derive(Error, Debug)]
pub enum ContestError {
    #[error("unmatched team: {}", 0.)]
    UnmatchedTeam(String),
    #[error("unmatched problem: {}", 0.)]
    UnmatchedProblem(String),
}

impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type TimeFile = i64;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, ToSchema)]
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

#[derive(Debug, Clone)]
/// A problem in the scoreboard.
pub struct ProblemView {
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
    pub id: u64,
    pub pending: usize,
}

impl PartialEq for ProblemView {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ProblemView {}

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

    pub fn is_frozen(&self) -> bool {
        self.current_time >= self.score_freeze_time * 60
    }

    pub fn fake() -> Self {
        Self::new(86399, 86399 + 1)
    }
}

impl Problem {
    fn empty() -> Self {
        Problem {
            solved: false,
            solved_first: false,
            submissions: 0,
            time_solved: 0,
            penalty: 0,
            answers: Vec::new(),
            id: gen_id(),
            waits: HashSet::new(),
        }
    }

    pub fn view(&self) -> ProblemView {
        let Self {
            solved,
            solved_first,
            submissions,
            penalty,
            time_solved,
            answers,
            id,
            waits,
        } = self;
        ProblemView {
            solved: *solved,
            solved_first: *solved_first,
            submissions: *submissions,
            penalty: *penalty,
            time_solved: *time_solved,
            id: *id,
            pending: answers.len() + waits.len(),
        }
    }

    fn add_run_problem(&mut self, answer: Answer) {
        self.id = gen_id();
        if self.solved {
            return;
        }
        match answer {
            Answer::Yes {
                time,
                is_first,
                run_id,
            } => {
                self.solved = true;
                self.submissions += 1;
                self.penalty += time;
                self.time_solved = time;
                self.answers.clear();
                self.solved_first = is_first;

                self.waits.remove(&run_id);
            }
            Answer::No { run_id } => {
                self.submissions += 1;
                self.penalty += 20;
                self.waits.remove(&run_id);
            }
            Answer::Wait { run_id } => {
                self.waits.insert(run_id);
            }
            Answer::Unk { run_id } => {
                self.waits.remove(&run_id);
            }
        }
    }

    fn wait(&self) -> bool {
        !self.solved && !self.answers.is_empty()
    }

    fn add_run_frozen(&mut self, answer: Answer) {
        self.id = gen_id();
        if !answer.is_wait() {
            self.answers.push(answer)
        }
    }

    fn reveal_run_frozen(&mut self) -> bool {
        self.id = gen_id();
        if self.wait() {
            let a = self.answers.remove(0);
            self.add_run_problem(a);
            // if !self.wait() {
            //     self.answers.clear();
            // }
            return true;
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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
    pub problems: BTreeMap<String, Problem>,

    pub id: u64,
}

impl PartialEq for Team {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.id == other.id
    }
}

impl PartialOrd for Team {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score().partial_cmp(&other.score())
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Score {
    pub solved: usize,
    pub penalty: i64,
    pub max_solution_time: i64,
    pub team_login: String,
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.solved != other.solved {
            other.solved.cmp(&self.solved)
        } else if self.penalty != other.penalty {
            self.penalty.cmp(&other.penalty)
        } else if self.max_solution_time != other.max_solution_time {
            self.max_solution_time.cmp(&other.max_solution_time)
        } else {
            self.team_login.cmp(&other.team_login)
        }
    }
}

static SEED: AtomicU64 = AtomicU64::new(0);

fn gen_id() -> u64 {
    SEED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    SEED.load(std::sync::atomic::Ordering::SeqCst)
}

impl Team {
    pub fn new(login: &str, escola: &str, name: String) -> Self {
        Self {
            login: login.to_string(),
            escola: escola.to_string(),
            name,
            placement: 0,
            placement_global: 0,
            problems: BTreeMap::new(),
            id: gen_id(),
        }
    }

    pub fn dummy() -> Self {
        Self::new("<login>", "<escola>", "<nome>".to_string())
    }

    fn apply_run(&mut self, run: &RunTuple) {
        self.id = gen_id();
        self.problems
            .entry(run.prob.clone())
            .or_insert(Problem::empty())
            .add_run_problem(run.answer.clone());
    }

    fn apply_run_frozen(&mut self, run: &RunTuple) {
        self.id = gen_id();
        self.problems
            .entry(run.prob.clone())
            .or_insert(Problem::empty())
            .add_run_frozen(run.answer.clone());
    }

    pub fn wait(&self) -> bool {
        self.problems.values().map(|p| p.wait()).any(|e| e)
    }

    pub fn reveal_run_frozen(&mut self) -> bool {
        for p in self.problems.values_mut() {
            if p.wait() && p.reveal_run_frozen() {
                self.id = gen_id();
                return true;
            }
        }
        false
    }

    pub fn score(&self) -> Score {
        let mut solved = 0;
        let mut penalty = 0;
        let mut max_solution_time = 0;
        for (_, value) in self.problems.iter() {
            if value.solved {
                solved += 1;
                penalty += value.penalty;
                max_solution_time = max_solution_time.max(value.time_solved);
            }
        }
        Score {
            solved,
            penalty,
            max_solution_time,
            team_login: self.login.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

pub const PROBLEM_LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub trait BelongsToContest {
    fn belongs_to_contest(&self, sede: Option<&Sede>) -> bool;
}

impl BelongsToContest for Team {
    fn belongs_to_contest(&self, sede: Option<&Sede>) -> bool {
        match sede {
            Some(sede) => sede.team_belongs(self),
            None => true,
        }
    }
}

impl ContestFile {
    pub fn new(
        contest_name: String,
        teams: Vec<Team>,
        current_time: i64,
        maximum_time: i64,
        score_freeze_time: i64,
        penalty: i64,
        number_problems: usize,
    ) -> Self {
        let mut m = BTreeMap::new();
        for t in teams {
            m.insert(t.login.clone(), t);
        }
        Self {
            contest_name,
            teams: m,
            current_time,
            maximum_time,
            score_freeze_time,
            penalty_per_wrong_answer: penalty,
            number_problems,
        }
    }

    pub fn remove_ccl(self) -> Self {
        Self {
            teams: self
                .teams
                .into_iter()
                .filter(|(_k, v)| !v.login.contains("ccl"))
                .collect(),
            ..self
        }
    }

    pub fn filter_sede(self, sede: &Sede) -> Self {
        Self {
            teams: self
                .teams
                .into_iter()
                .filter(|(login, _t)| sede.team_belongs_str(&login))
                .collect(),
            ..self
        }
    }

    pub fn recalculate_placement(&mut self) {
        let mut teams = self.teams.iter_mut().map(|(_t, v)| v).collect::<Vec<_>>();
        teams.sort_by_cached_key(|t| t.score());

        for (i, t) in teams.iter_mut().enumerate() {
            if t.placement_global != i + 1 {
                t.placement_global = i + 1;
                t.id = gen_id()
            }
        }
    }

    pub fn dummy() -> Self {
        Self::new("Dummy Contest".to_string(), Vec::new(), 0, 0, 0, 0, 0)
    }

    pub fn apply_run(&mut self, r: &RunTuple) {
        if let Some(t) = self.teams.get_mut(&r.team_login) {
            t.apply_run(r);
        }
    }

    pub fn apply_run_frozen(&mut self, r: &RunTuple) {
        if let Some(t) = self.teams.get_mut(&r.team_login) {
            t.apply_run_frozen(r);
        }
    }

    pub fn build_panel_item(&self, run: &RunTuple) -> Result<RunsPanelItem, ContestError> {
        let team = self
            .teams
            .get(&run.team_login)
            .ok_or(ContestError::UnmatchedTeam(run.team_login.clone()))?;

        let problem = team
            .problems
            .get(&run.prob)
            .ok_or(ContestError::UnmatchedProblem(run.prob.clone()))?;

        let view = problem.view();

        Ok(RunsPanelItem {
            id: run.id,
            placement: team.placement_global,
            escola: team.escola.clone(),
            team_name: team.name.clone(),
            team_login: run.team_login.clone(),
            problem: run.prob.clone(),
            problem_view: view,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, ToSchema)]
/// A submission being judged.
pub struct RunTuple {
    /// Id of submission.
    pub id: i64,
    /// Order in input.
    pub order: u64,
    /// Time of the submision.
    pub time: i64,
    /// The team login.
    pub team_login: String,
    /// The problem letter.
    pub prob: String,
    /// The answer for this submission.
    pub answer: Answer,
}

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

impl fmt::Display for RunTuple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct RunsPanelItem {
    pub id: i64,
    pub placement: usize,
    pub escola: String,
    pub team_name: String,
    pub team_login: String,
    pub problem: String,
    pub problem_view: ProblemView,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunsFile {
    runs: BTreeMap<i64, RunTuple>,
}

#[derive(Debug, Clone)]
pub struct RunsFileContest(RunsFile);

impl AsRef<RunsFile> for RunsFileContest {
    fn as_ref(&self) -> &RunsFile {
        &self.0
    }
}

impl RunsFile {
    pub fn empty() -> Self {
        Self {
            runs: BTreeMap::new(),
        }
    }

    pub fn new(runs: Vec<RunTuple>) -> Self {
        let mut t = Self::empty();
        for r in runs {
            t.runs.insert(r.id, r);
        }
        t
    }

    pub fn len(&self) -> usize {
        self.runs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }

    pub fn sorted(&self) -> Vec<RunTuple> {
        let mut r: Vec<_> = self.runs.values().cloned().collect();
        r.sort_by_key(|r| (r.time, r.order));
        r
    }

    pub fn filter_frozen(&self, frozen_time: i64) -> Self {
        Self::new(
            self.sorted()
                .into_iter()
                .filter(|r| r.time < frozen_time)
                .collect(),
        )
    }

    pub fn filter_sede(&self, sede: &Sede) -> Self {
        Self {
            runs: self
                .runs
                .iter()
                .filter_map(|(key, value)| {
                    sede.team_belongs_str(&value.team_login)
                        .then_some((*key, value.clone()))
                })
                .collect(),
        }
    }

    pub fn filter_teams(&mut self, contest: &ContestFile) {
        let runs = &mut self.runs;
        runs.retain(|&_, run| contest.teams.contains_key(&run.team_login));
    }

    pub fn into_runs_sede(&self, contest: &ContestFile) -> RunsFileContest {
        let mut runs = self.clone();
        runs.filter_teams(contest);
        RunsFileContest(runs)
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

    pub fn refresh(&mut self, fresh: Vec<RunTuple>) -> Vec<RunTuple> {
        let mut rec = Vec::new();

        for t in fresh {
            if self.refresh_1(&t) {
                rec.push(t);
            }
        }

        rec
    }
}
