pub mod auth;
// pub mod config;
pub mod configdata;
pub mod revelation;
pub mod turb;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use serde::{Deserialize, Serialize};
use std::collections::{btree_map, BTreeMap};
use std::fmt;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq)]
pub enum Answer {
    Yes(i64),
    No,
    Wait,
    Unk,
}

#[derive(Debug)]
pub enum ContestError {
    UnmatchedTeam(String),
}

impl std::error::Error for ContestError {}

impl fmt::Display for ContestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ContestError: {:?}", self)
    }
}

impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type TimeFile = i64;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Problem {
    pub solved: bool,
    pub submissions: usize,
    pub penalty: i64,
    pub time_solved: i64,
    pub answers: Vec<Answer>,
}

#[derive(Copy, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimerData {
    pub current_time: TimeFile,
    pub score_freeze_time: i64,
}

impl TimerData {
    pub fn new(current_time: TimeFile, score_freeze_time: i64) -> Self {
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
    pub fn empty() -> Self {
        Problem {
            solved: false,
            submissions: 0,
            time_solved: 0,
            penalty: 0,
            answers: Vec::new(),
        }
    }
    pub fn add_run_problem(&mut self, answer: Answer) {
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
        !self.solved && self.answers.len() > 0
    }

    pub fn add_run_frozen(&mut self, answer: Answer) {
        if answer != Answer::Wait {
            self.answers.push(answer)
        }
    }

    pub fn reveal_run_frozen(&mut self) {
        if self.wait() {
            let a = self.answers.remove(0);
            self.add_run_problem(a);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub login: String,
    pub escola: String,
    pub name: String,
    pub placement: usize,
    pub problems: BTreeMap<String, Problem>,
}

impl Eq for Team {}
impl PartialEq for Team {
    fn eq(&self, other: &Self) -> bool {
        self.score() == other.score()
    }
}

impl PartialOrd for Team {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.score().cmp(&other.score()))
    }
}

impl Ord for Team {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score().cmp(&other.score())
    }
}

use std::cmp::{Eq, Ord, Ordering};

#[derive(PartialEq, Eq)]
pub struct Score {
    pub solved: usize,
    pub penalty: i64,
    pub max_solution_time: i64,
    pub team_login: String,
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(if self.solved != other.solved {
            other.solved.cmp(&self.solved)
        } else if self.penalty != other.penalty {
            self.penalty.cmp(&other.penalty)
        } else if self.max_solution_time != other.max_solution_time {
            self.max_solution_time.cmp(&other.max_solution_time)
        } else {
            self.team_login.cmp(&other.team_login)
        })
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Team {
    pub fn new(login: &str, escola: &str, name: &str) -> Self {
        Self {
            login: login.to_string(),
            escola: escola.to_string(),
            name: name.to_string(),
            placement: 0,
            problems: BTreeMap::new(),
        }
    }

    pub fn dummy() -> Self {
        Self::new("<login>", "<escola>", "<nome>")
    }

    fn apply_run(&mut self, run: &RunTuple) {
        self.problems
            .entry(run.prob.clone())
            .or_insert(Problem::empty())
            .add_run_problem(run.answer.clone());
    }

    fn apply_run_frozen(&mut self, run: &RunTuple) {
        self.problems
            .entry(run.prob.clone())
            .or_insert(Problem::empty())
            .add_run_frozen(run.answer.clone());
    }

    pub fn wait(&self) -> bool {
        self.problems
            .values()
            .map(|p| p.wait())
            .fold(false, |t, e| t || e)
    }

    pub fn reveal_run_frozen(&mut self) {
        for p in self.problems.values_mut() {
            if p.wait() {
                p.reveal_run_frozen();
                return;
            }
        }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContestFile {
    pub contest_name: String,
    pub teams: BTreeMap<String, Team>,
    pub current_time: i64,
    pub maximum_time: i64,
    pub score_freeze_time: i64,
    pub penalty_per_wrong_answer: i64,
    pub score_board: Vec<String>,
    pub number_problems: usize,
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
            score_board: Vec::new(),
            number_problems: number_problems,
        }
    }

    pub fn placement(&self, team_login: &String) -> Option<usize> {
        self.teams.get(team_login).map(|t| t.placement)
    }

    pub fn recalculate_placement(&mut self) -> Result<(), ContestError> {
        let mut score_board = Vec::new();
        for (key, _) in self.teams.iter() {
            score_board.push(key.clone());
        }
        score_board.sort_by(|a, b| {
            let score_a = self.teams.get(a).unwrap().score();
            let score_b = self.teams.get(b).unwrap().score();
            score_a.cmp(&score_b)
        });
        for (i, v) in score_board.iter().enumerate() {
            match self.teams.get_mut(v) {
                None => return Err(ContestError::UnmatchedTeam(v.clone())),
                Some(t) => t.placement = i + 1,
            }
        }

        Ok(())
    }

    pub fn reload_score(&mut self) -> Result<(), ContestError> {
        let mut score_board = Vec::new();
        for (key, _) in self.teams.iter() {
            score_board.push(key.clone());
        }
        score_board.sort_by(|a, b| {
            let score_a = self.teams.get(a).unwrap().score();
            let score_b = self.teams.get(b).unwrap().score();
            score_a.cmp(&score_b)
        });
        for (i, v) in score_board.iter().enumerate() {
            match self.teams.get_mut(v) {
                None => return Err(ContestError::UnmatchedTeam(v.clone())),
                Some(t) => t.placement = i + 1,
            }
        }

        self.score_board = score_board;
        Ok(())
    }

    pub fn dummy() -> Self {
        Self::new("Dummy Contest".to_string(), Vec::new(), 0, 0, 0, 0, 0)
    }

    pub fn apply_run(&mut self, r: &RunTuple) -> Result<(), ContestError> {
        match self.teams.get_mut(&r.team_login) {
            None => Err(ContestError::UnmatchedTeam(
                "Could not apply run to team".to_string(),
            )),
            Some(t) => {
                t.apply_run(&r);
                Ok(())
            }
        }
    }

    pub fn apply_run_frozen(&mut self, r: &RunTuple) -> Result<Score, ContestError> {
        match self.teams.get_mut(&r.team_login) {
            None => Err(ContestError::UnmatchedTeam(
                "Could not apply run to team".to_string(),
            )),
            Some(t) => {
                t.apply_run_frozen(&r);
                Ok(t.score())
            }
        }
    }

    pub fn build_panel_item(&self, run: &RunTuple) -> Result<RunsPanelItem, ContestError> {
        let team = self
            .teams
            .get(&run.team_login)
            .ok_or(ContestError::UnmatchedTeam(run.team_login.clone()))?;

        Ok(RunsPanelItem {
            id: run.id,
            placement: team.placement,
            color: 0,
            escola: team.escola.clone(),
            team_name: team.name.clone(),
            team_login: run.team_login.clone(),
            problem: run.prob.clone(),
            result: run.answer.clone(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct RunTuple {
    pub id: i64,
    pub time: i64,
    pub team_login: String,
    pub prob: String,
    pub answer: Answer,
}

impl PartialOrd for RunTuple {
    fn partial_cmp(&self, other : &Self) -> Option<Ordering> {
        Some(self.time.cmp(&other.time))
    }
}

impl Ord for RunTuple {
    fn cmp(&self, other : &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct RunsPanelItem {
    pub id: i64,
    pub placement: usize,
    pub color: i64,
    pub escola: String,
    pub team_name: String,
    pub team_login: String,
    pub problem: String,
    pub result: Answer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunsFile {
    runs: BTreeMap<i64, RunTuple>,
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

    pub fn refresh_1(&mut self, t :&RunTuple) -> bool {
        let ent = self.runs.entry(t.id);
        match ent {
            btree_map::Entry::Vacant(v) => {
                v.insert(t.clone());
                true
            }
            btree_map::Entry::Occupied(mut o) => {
                if o.get() != t {
                    *o.get_mut() = t.clone();
                    return true
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
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let r = g.next_u32() % 3;

            if r == 0 {
                Answer::Yes(g.next_u64() as i64)
            }
            // else if r == 1{
            //     Answer::Wait
            // }
            else {
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

            // p2.answers.clear();

            println!("p2={:?}", p2);
            println!("p1==p2= {}", p1==p2);

            p1 == p2
        }
    }
}
