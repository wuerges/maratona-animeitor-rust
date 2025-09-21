use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::model::animeitor_v3::{scoreboard::Score, team::ContestService};

use super::team::Team;
use leptos::leptos_dom::logging::console_log;
use sdk::ContestParameters;

pub struct Contest {
    parameters: ContestParameters,
    teams: HashMap<String, Team>,
    service: Service,
    score_queue: ScoreQueue,
}

struct Service {
    penalty: u32,
    was_solved: HashSet<String>,
}

impl ContestService for Service {
    fn contest_penalty(&self) -> u32 {
        self.penalty
    }

    fn problem_was_solved(&mut self, letter: &str) -> bool {
        self.was_solved.insert(letter.to_string())
    }
}

impl Contest {
    pub fn new(parameters: ContestParameters) -> Self {
        Self {
            teams: parameters
                .teams
                .iter()
                .map(|t| {
                    (
                        t.login.clone(),
                        Team::new(t.clone(), &parameters.problem_letters),
                    )
                })
                .collect(),
            service: Service {
                penalty: parameters.penalty_per_wrong_answer,
                was_solved: Default::default(),
            },
            score_queue: Default::default(),
            parameters,
        }
    }

    pub fn parameters(&self) -> &ContestParameters {
        &self.parameters
    }

    pub fn teams(&self) -> impl Iterator<Item = &Team> {
        self.teams.values()
    }

    pub fn judge_run(&mut self, run: &sdk::Run) {
        if let Some(team) = self.teams.get_mut(&run.team_login) {
            team.judge_run(run, &mut self.service)
        }
    }

    pub fn push_run(&mut self, new_run: sdk::Run) {
        if let Some(team) = self.teams.get_mut(&new_run.team_login) {
            team.push_run(new_run);
            self.score_queue.push_team(team);
        }
    }

    pub fn pop_run(&mut self, contest: &mut impl ContestService) {
        if let Some(team_login) = self.score_queue.pop_team() {
            if let Some(team) = self.teams.get_mut(&team_login) {
                if team.pop_run(contest) {
                    self.score_queue.push_team(team);
                }
            }
        }
    }
}

#[derive(Debug, Default)]
struct ScoreQueue {
    score_map: BTreeMap<Score, BTreeSet<String>>,
    team_set: BTreeSet<String>,
}

impl ScoreQueue {
    pub fn push_team(&mut self, team: &Team) {
        let login = team.info().login.clone();

        if self.team_set.insert(login.clone()) {
            self.score_map
                .entry(team.score())
                .or_default()
                .insert(login);
        }
    }

    pub fn pop_team(&mut self) -> Option<String> {
        let mut entry = self.score_map.last_entry()?;

        let teams = entry.get_mut();

        let found = teams.pop_last()?;
        self.team_set.remove(&found);

        if teams.is_empty() {
            entry.remove();
        }

        Some(found)
    }
}
