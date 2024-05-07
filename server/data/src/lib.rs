pub mod configdata;
pub mod revelation;

use configdata::Sede;
use serde::{Deserialize, Serialize};
use std::cmp::{Eq, Ordering};
use std::collections::{btree_map, BTreeMap, HashMap};
use std::fmt;
use std::sync::atomic::AtomicU64;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, ToSchema)]
/// The judge answer to a submission.
pub enum Answer {
    /// Accepted, with the time of the submission.
    Yes(i64),
    /// Rejected.
    No,
    /// Waiting to be judged.
    Wait,
    /// Unknown.
    Unk,
}

#[derive(Error, Debug)]
pub enum ContestError {
    #[error("unmatched team: {}", 0.)]
    UnmatchedTeam(String),
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
    pub wait: bool,
    pub id: u64,
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
        }
    }

    pub fn view(&self) -> ProblemView {
        let Self {
            solved,
            solved_first,
            submissions,
            penalty,
            time_solved,
            answers: _,
            id,
        } = self;
        ProblemView {
            solved: *solved,
            solved_first: *solved_first,
            submissions: *submissions,
            penalty: *penalty,
            time_solved: *time_solved,
            id: *id,
            wait: self.wait(),
        }
    }

    fn add_run_problem(&mut self, answer: Answer) {
        self.id = gen_id();
        if self.solved {
            return;
        }
        match answer {
            Answer::Yes(tim) => {
                self.solved = true;
                self.submissions += 1;
                self.penalty += tim;
                self.time_solved = tim;
                self.answers.clear();
            }
            Answer::No => {
                self.submissions += 1;
                self.penalty += 20;
            }
            Answer::Wait => {
                self.answers.push(Answer::No) // failsafe
            }
            _ => {}
        }
    }

    pub fn wait(&self) -> bool {
        !self.solved && !self.answers.is_empty()
    }

    fn add_run_frozen(&mut self, answer: Answer) {
        self.id = gen_id();
        if answer != Answer::Wait {
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
        self.name == other.name
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
    /// Time of the first solution for each problem.
    pub first_solution_time: HashMap<String, i64>,
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
            first_solution_time: HashMap::new(),
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

    pub fn placement(&self, team_login: &String) -> Option<usize> {
        self.teams.get(team_login).map(|t| t.placement)
    }

    pub fn recalculate_placement_no_filter(&mut self) -> Result<(), ContestError> {
        self.recalculate_placement(None)
    }

    pub fn recalculate_stars(&mut self) {
        for (_, team) in &self.teams {
            for (l, problem) in &team.problems {
                if problem.solved {
                    match self.first_solution_time.entry(l.clone()) {
                        std::collections::hash_map::Entry::Occupied(mut o) => {
                            *o.get_mut() = *o.get().min(&problem.time_solved)
                        }
                        std::collections::hash_map::Entry::Vacant(v) => {
                            v.insert(problem.time_solved);
                        }
                    }
                }
            }
        }
        for (_, team) in &mut self.teams {
            for (l, problem) in &mut team.problems {
                if problem.time_solved
                    == self.first_solution_time.get(l).copied().unwrap_or_default()
                {
                    problem.solved_first = true
                }
            }
        }
    }

    pub fn recalculate_placement(
        &mut self,
        sede_filter: Option<&Sede>,
    ) -> Result<(), ContestError> {
        let mut score_board = Vec::new();
        for (key, _) in self.teams.iter() {
            score_board.push(key.clone());
        }
        score_board.sort_by(|a, b| {
            let score_a = self.teams.get(a).unwrap().score();
            let score_b = self.teams.get(b).unwrap().score();
            score_a.cmp(&score_b)
        });
        let mut placement = 1;
        let mut placement_global = 1;
        for v in score_board.iter() {
            if let Some(t) = self.teams.get_mut(v) {
                t.placement = placement;
                t.placement_global = placement_global;

                if t.belongs_to_contest(sede_filter) {
                    placement += 1;
                }
                placement_global += 1;
            }
        }

        self.recalculate_stars();

        Ok(())
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

        let first_time = self
            .first_solution_time
            .get(&run.prob)
            .copied()
            .unwrap_or_default();

        Ok(RunsPanelItem {
            id: run.id,
            placement: team.placement_global,
            color: 0,
            escola: team.escola.clone(),
            team_name: team.name.clone(),
            team_login: run.team_login.clone(),
            problem: run.prob.clone(),
            result: run.answer.clone(),
            first_solved: first_time == run.time,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, ToSchema)]
/// A submission being judged.
pub struct RunTuple {
    /// Id of submission.
    pub id: i64,
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

impl RunTuple {
    pub fn new(id: i64, time: i64, team_login: String, prob: String, answer: Answer) -> Self {
        Self {
            id,
            time,
            team_login,
            prob,
            answer,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunsPanelItem {
    pub id: i64,
    pub placement: usize,
    pub color: i64,
    pub escola: String,
    pub team_name: String,
    pub team_login: String,
    pub problem: String,
    pub result: Answer,
    pub first_solved: bool,
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
        r.sort_by(|t1, t2| t1.time.cmp(&t2.time));
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

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::*;

    impl Arbitrary for Answer {
        fn arbitrary(g: &mut Gen) -> Self {
            let r = u32::arbitrary(g) % 3;

            if r == 0 {
                Answer::Yes(i64::arbitrary(g) % 1e18 as i64)
            } else {
                Answer::No
            }
        }
    }

    quickcheck! {
        fn problem_with_runs_is_the_same_as_revealed(answers : Vec<Answer>) -> bool {
            let mut p1 = Problem::empty();
            let mut p2 = Problem::empty();
            println!("------------------------------");
            println!("answers={:?}", answers);
            for a in &answers {
                p1.add_run_problem(a.clone());
                p2.add_run_frozen(a.clone());
            }
            println!("p1={:?}", p1);
            while p2.wait() {
                p2.reveal_run_frozen();

            }
            println!("p2={:?}", p2);

            println!("p2={:?}", p2);
            println!("p1==p2= {}", p1==p2);

            p1 == p2
        }
    }
}
