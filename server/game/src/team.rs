use std::collections::{HashMap, VecDeque};

use futures_signals::signal::Mutable;

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
}

impl Problem {
    pub fn new(letter: &str) -> Self {
        Self {
            letter: letter.to_string(),
            judged: Default::default(),
            state_mutable: Default::default(),
            pending: Default::default(),
        }
    }

    fn judge_run(&self, run: sdk::Run) {
        todo!()
    }

    fn push_run(&self, run: sdk::Run) {
        todo!()
    }

    fn pop_run(&self) -> bool {
        todo!()
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum ProblemState {
    #[default]
    Fresh,
    UnderJudgement {
        penalty: u32,
        judged_attempts: u32,
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
        penalty: u32,
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

    pub fn judge_run(&mut self, run: sdk::Run) {
        self.get_problem(&run.problem_letter).judge_run(run)
    }

    pub fn push_run(&mut self, run: sdk::Run) {
        self.get_problem(&run.problem_letter).push_run(run)
    }

    pub fn pop_run(&mut self, run: sdk::Run) -> bool {
        self.get_problem(&run.problem_letter).pop_run()
    }
}

pub trait ContestService {
    fn contest_penalty(&self) -> u32;
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
