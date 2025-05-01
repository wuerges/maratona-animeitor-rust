use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use data::{ContestFile, RunTuple};
use itertools::Itertools;
use leptos::prelude::*;

use super::team_signal::TeamSignal;

pub struct ContestSignal {
    pub teams: HashMap<String, Arc<TeamSignal>>,
    pub team_global_placements: RwSignal<Vec<String>>,
}

impl ContestSignal {
    pub fn new(contest_file: &ContestFile) -> Self {
        let letters = data::PROBLEM_LETTERS
            .chars()
            .take(contest_file.number_problems)
            .map(|l| l.to_string())
            .collect::<Vec<_>>();

        ContestSignal {
            teams: contest_file
                .teams
                .iter()
                .map(|(login, team)| (login.clone(), Arc::new(TeamSignal::new(team, &letters))))
                .collect(),
            team_global_placements: RwSignal::new(
                contest_file
                    .teams
                    .values()
                    .map(|team| team.login.clone())
                    .collect(),
            ),
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

        self.team_global_placements.set(
            fresh_contest
                .teams
                .values()
                .sorted_by_cached_key(|team| team.placement_global)
                .map(|team| team.login.clone())
                .collect(),
        );
    }

    pub fn update_tuples(&self, runs: &[RunTuple], fresh_contest: &ContestFile) {
        self.update(
            runs.iter().map(|run| run.team_login.as_str()),
            fresh_contest,
        )
    }
}
