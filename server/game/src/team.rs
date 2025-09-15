use std::collections::{HashMap, VecDeque};

use futures_signals::signal::{Mutable, Signal};

use crate::{TeamSites, scoreboard::Score};

pub struct Team {
    team: sdk::Team,
    score: Mutable<Score>,
    placements: HashMap<String, Mutable<u32>>,
    problems: HashMap<String, Problem>,
}

pub struct Problem {
    letter: String,
    state_mutable: Mutable<ProblemState>,
    judged: Vec<sdk::Run>,
    pending: VecDeque<sdk::Run>,
    is_solved: bool,
    solve_time_in_minutes: u32,
    is_first_solved: bool,
    failed_attempts: u32,
    penalty: u32,
}

impl Problem {
    pub fn new(letter: &str) -> Self {
        Self {
            letter: letter.to_string(),
            judged: Default::default(),
            state_mutable: Default::default(),
            pending: Default::default(),
            is_solved: false,
            is_first_solved: false,
            failed_attempts: 0,
            solve_time_in_minutes: 0,
            penalty: 0,
        }
    }
    pub fn signal(&self) -> impl Signal<Item = ProblemState> {
        self.state_mutable.signal()
    }

    fn mark_solved(&mut self, time_in_minutes: u32, contest: &mut impl ContestService) {
        self.is_solved = true;
        self.is_first_solved = contest.problem_was_solved(&self.letter);
        self.penalty = time_in_minutes + self.failed_attempts * contest.contest_penalty();
        self.pending.clear();
    }

    fn update_state(&self) {
        if self.is_solved {
            self.state_mutable.set(ProblemState::Solved {
                is_first: self.is_first_solved,
                time_in_minutes: self.solve_time_in_minutes,
                penalty: self.penalty,
                attempts: self.failed_attempts,
            });
        } else if self.pending.is_empty() {
            self.state_mutable.set(ProblemState::WrongAnswer {
                judged_attempts: self.failed_attempts,
            });
        } else {
            self.state_mutable.set(ProblemState::UnderJudgement {
                failed_attempts: self.failed_attempts,
                new_attempts: self.pending.len() as u32,
            });
        }
    }

    fn judge_run(
        &mut self,
        sdk::Run {
            id: _,
            time_in_minutes,
            team_login: _,
            problem_letter: _,
            answer,
        }: sdk::Run,
        contest: &mut impl ContestService,
    ) {
        if !self.is_solved {
            match answer {
                sdk::Answer::Yes => {
                    self.mark_solved(time_in_minutes, contest);
                    self.update_state();
                }
                sdk::Answer::No => {
                    self.failed_attempts += 1;
                    self.update_state();
                }
                sdk::Answer::Undecided => (),
                sdk::Answer::NoWithoutPenalty => {
                    self.update_state();
                }
            }
        }
    }

    fn push_run(&mut self, new_run: sdk::Run) {
        if !self.is_solved {
            for run in &mut self.pending {
                if run.id == new_run.id {
                    *run = new_run;
                    return;
                }
            }
            self.pending.push_back(new_run);
            self.update_state();
        }
    }

    fn pop_run(&mut self, contest: &mut impl ContestService) -> bool {
        if let Some(run) = self.pending.pop_back() {
            self.judge_run(run, contest);
        }
        !self.pending.is_empty()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ProblemState {
    #[default]
    Fresh,
    UnderJudgement {
        failed_attempts: u32,
        new_attempts: u32,
    },
    Solved {
        is_first: bool,
        time_in_minutes: u32,
        penalty: u32,
        attempts: u32,
    },
    WrongAnswer {
        judged_attempts: u32,
    },
}

impl Team {
    pub fn new(team: sdk::Team, sites: Vec<String>, letters: &[&str]) -> Self {
        Self {
            team,
            score: Default::default(),
            placements: sites
                .into_iter()
                .map(|site| (site, Default::default()))
                .collect(),
            problems: letters
                .iter()
                .map(|l| (l.to_string(), Problem::new(l)))
                .collect(),
        }
    }

    fn get_problem(&mut self, letter: &str) -> &mut Problem {
        self.problems
            .entry(letter.to_string())
            .or_insert_with(|| Problem::new(letter))
    }

    pub fn judge_run(&mut self, run: sdk::Run, contest: &mut impl ContestService) {
        self.get_problem(&run.problem_letter)
            .judge_run(run, contest)
    }

    pub fn push_run(&mut self, run: sdk::Run) {
        self.get_problem(&run.problem_letter).push_run(run)
    }

    pub fn pop_run(&mut self, run: sdk::Run, contest: &mut impl ContestService) -> bool {
        self.get_problem(&run.problem_letter).pop_run(contest)
    }
}

pub trait ContestService {
    fn contest_penalty(&self) -> u32;
    /// Mark that problem was solved.
    /// return if it was previously solved
    fn problem_was_solved(&mut self, letter: &str) -> bool;
}

impl TeamSites for Team {
    type Site = String;
    type Login = String;

    fn sites(&self) -> impl Iterator<Item = &Self::Site> {
        self.placements.keys()
    }

    fn login(&self) -> &Self::Login {
        &self.team.login
    }

    fn score(&self, contest: &impl ContestService) -> Score {
        let mut total_solved = 0;
        let mut total_penalty = 0;

        for problem in self.problems.values() {
            let mut problem_penalty = 0;

            for sdk::Run {
                id: _,
                time_in_minutes,
                team_login: _,
                problem_letter: _,
                answer,
            } in &problem.judged
            {
                match answer {
                    sdk::Answer::Yes => {
                        total_solved += 1;
                        total_penalty += problem_penalty + *time_in_minutes;
                    }
                    sdk::Answer::No => {
                        problem_penalty += contest.contest_penalty();
                    }
                    sdk::Answer::Undecided | sdk::Answer::NoWithoutPenalty => (),
                }
            }
        }

        Score::new(total_solved, total_penalty)
    }
}
