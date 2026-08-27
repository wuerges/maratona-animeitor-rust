use std::cmp::Ordering;

use data::{gen_id, Answer, ContestFile, Letter, Problem, RunTuple, Team};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContestError {
    #[error("unmatched team: {}", 0.)]
    UnmatchedTeam(String),
    #[error("unmatched problem: {}", 0.)]
    UnmatchedProblem(Letter),
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

impl ProblemView {
    pub fn is_resolved(&self) -> bool {
        self.solved || self.pending == 0
    }
}

impl PartialEq for ProblemView {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ProblemView {}

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

#[derive(Debug, Clone)]
pub struct RunsPanelItem {
    pub id: i64,
    pub order: u64,
    pub placement: usize,
    pub escola: String,
    pub team_name: String,
    pub team_login: String,
    pub problem: Letter,
    pub problem_view: ProblemView,
}

/// Client-side scoring view of a [`Problem`].
pub trait ProblemExt {
    fn view(&self) -> ProblemView;
}

impl ProblemExt for Problem {
    fn view(&self) -> ProblemView {
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
}

/// Client-side scoring engine for a [`Team`].
pub trait TeamExt {
    fn score(&self) -> Score;

    fn reveal_run_frozen(&mut self) -> bool;
}

impl TeamExt for Team {
    fn score(&self) -> Score {
        let mut solved = 0;
        let mut penalty = 0;
        let mut max_solution_time = 0;
        for value in self.problems.values() {
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

    fn reveal_run_frozen(&mut self) -> bool {
        for p in self.problems.values_mut() {
            if problem_wait(p) && problem_reveal_run_frozen(p) {
                self.id = gen_id();
                return true;
            }
        }
        false
    }
}

/// Client-side scoring engine for a [`ContestFile`].
pub trait ContestFileExt {
    fn apply_run(&mut self, r: &RunTuple);

    fn apply_run_frozen(&mut self, r: &RunTuple);

    fn recalculate_placement(&mut self);

    fn build_panel_item(&self, run: &RunTuple) -> Result<RunsPanelItem, ContestError>;
}

impl ContestFileExt for ContestFile {
    fn apply_run(&mut self, r: &RunTuple) {
        if let Some(t) = self.teams.get_mut(&r.team_login) {
            team_apply_run(t, r);
        }
    }

    fn apply_run_frozen(&mut self, r: &RunTuple) {
        if let Some(t) = self.teams.get_mut(&r.team_login) {
            team_apply_run_frozen(t, r);
        }
    }

    fn recalculate_placement(&mut self) {
        let mut teams = self.teams.iter_mut().map(|(_t, v)| v).collect::<Vec<_>>();
        teams.sort_by_cached_key(|t| t.score());

        for (i, t) in teams.iter_mut().enumerate() {
            if t.placement_global != i + 1 {
                t.placement_global = i + 1;
                t.id = gen_id()
            }
        }
    }

    fn build_panel_item(&self, run: &RunTuple) -> Result<RunsPanelItem, ContestError> {
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
            order: run.order,
            placement: team.placement_global,
            escola: team.escola.clone(),
            team_name: team.name.clone(),
            team_login: run.team_login.clone(),
            problem: run.prob.clone(),
            problem_view: view,
        })
    }
}

fn problem_empty() -> Problem {
    Problem {
        solved: false,
        solved_first: false,
        submissions: 0,
        time_solved: 0,
        penalty: 0,
        answers: Vec::new(),
        id: gen_id(),
        waits: Default::default(),
    }
}

fn problem_add_run(problem: &mut Problem, answer: Answer) {
    if problem.solved {
        return;
    }
    problem.id = gen_id();
    match answer {
        Answer::Yes {
            time,
            is_first,
            run_id,
        } => {
            problem.solved = true;
            problem.submissions += 1;
            problem.penalty += time;
            problem.time_solved = time;
            problem.answers.clear();
            problem.solved_first = is_first;

            problem.waits.remove(&run_id);
        }
        Answer::No { run_id } => {
            problem.submissions += 1;
            problem.penalty += 20;
            problem.waits.remove(&run_id);
        }
        Answer::Wait { run_id } => {
            problem.waits.insert(run_id);
        }
        Answer::Unk { run_id } => {
            problem.waits.remove(&run_id);
        }
    }
}

fn problem_wait(problem: &Problem) -> bool {
    !problem.solved && !problem.answers.is_empty()
}

fn problem_add_run_frozen(problem: &mut Problem, answer: Answer) {
    problem.id = gen_id();
    if !matches!(answer, Answer::Wait { .. }) {
        problem.answers.push(answer)
    }
}

fn problem_reveal_run_frozen(problem: &mut Problem) -> bool {
    if problem_wait(problem) {
        let a = problem.answers.remove(0);
        problem_add_run(problem, a);
        return true;
    }
    false
}

fn team_apply_run(team: &mut Team, run: &RunTuple) {
    team.id = gen_id();
    let problem = team.problems.entry(run.prob.clone()).or_insert_with(problem_empty);
    problem_add_run(problem, run.answer.clone());
}

fn team_apply_run_frozen(team: &mut Team, run: &RunTuple) {
    team.id = gen_id();
    let problem = team.problems.entry(run.prob.clone()).or_insert_with(problem_empty);
    problem_add_run_frozen(problem, run.answer.clone());
}
