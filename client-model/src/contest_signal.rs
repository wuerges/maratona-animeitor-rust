use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use data::{problem_letters, ContestFile, RunTuple};
use itertools::Itertools;
use reactive_graph::{prelude::*, signal::RwSignal};

use crate::team_signal::TeamSignal;

pub struct ContestSignal {
    pub teams: HashMap<String, Arc<TeamSignal>>,
    pub team_global_placements: RwSignal<Vec<String>>,
}

impl ContestSignal {
    pub fn new(contest_file: &ContestFile) -> Self {
        let letters = problem_letters(contest_file.number_problems);

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

    /// Updates the teams changed by the given runs and rebuilds the global
    /// placement list only when at least one placement actually changed.
    ///
    /// This runs inside the revealitor Effect, so it must not do any
    /// plain signal reads — all comparisons happen inside `maybe_update`.
    pub fn update<'a>(
        &self,
        team_logins: impl Iterator<Item = &'a str>,
        fresh_contest: &ContestFile,
    ) {
        let update_set: HashSet<_> = team_logins.collect();
        let mut placements_changed = false;

        for team in fresh_contest.teams.values() {
            if let Some(team_signal) = self.teams.get(&team.login) {
                if update_set.contains(team.login.as_str()) {
                    placements_changed |= team_signal.update(team);
                } else {
                    team_signal.placement_global.maybe_update(|p| {
                        if *p != team.placement_global {
                            *p = team.placement_global;
                            placements_changed = true;
                            true
                        } else {
                            false
                        }
                    });
                }
            }
        }

        if placements_changed {
            self.team_global_placements.set(
                fresh_contest
                    .teams
                    .values()
                    .sorted_by_cached_key(|team| team.placement_global)
                    .map(|team| team.login.clone())
                    .collect(),
            );
        }
    }

    pub fn update_tuples(&self, runs: &[RunTuple], fresh_contest: &ContestFile) {
        self.update(
            runs.iter().map(|run| run.team_login.as_str()),
            fresh_contest,
        )
    }
}
