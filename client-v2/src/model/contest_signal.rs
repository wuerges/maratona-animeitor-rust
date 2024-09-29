use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use data::{ContestFile, RunTuple};
use leptos::SignalSet;

use super::team_signal::TeamSignal;

pub struct ContestSignal {
    pub teams: HashMap<String, Rc<TeamSignal>>,
}

impl ContestSignal {
    pub fn new(contest_file: &ContestFile) -> Self {
        let letters = data::PROBLEM_LETTERS[..contest_file.number_problems]
            .chars()
            .map(|l| l.to_string())
            .collect::<Vec<_>>();
        ContestSignal {
            teams: contest_file
                .teams
                .iter()
                .map(|(login, team)| (login.clone(), Rc::new(TeamSignal::new(team, &letters))))
                .collect(),
        }
    }

    pub fn update<'a>(
        &self,
        team_logins: impl Iterator<Item = &'a str>,
        fresh_contest: &ContestFile,
    ) {
        let update_set: HashSet<_> = team_logins.collect();

        for team in fresh_contest.teams.values() {
            if let Some(team_signal) = self.teams.get(&team.login) {
                if update_set.contains(team.login.as_str()) {
                    team_signal.update(team);
                } else {
                    team_signal.placement_global.set(team.placement_global)
                }
            }
        }
    }

    pub fn update_tuples(&self, runs: &[RunTuple], fresh_contest: &ContestFile) {
        self.update(
            runs.iter().map(|run| run.team_login.as_str()),
            fresh_contest,
        )
    }
}
