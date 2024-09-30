use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use data::{RunTuple, RunningContest};
use itertools::Itertools;
use leptos::{create_rw_signal, RwSignal, SignalSet};

use super::team_signal::TeamSignal;

pub struct ContestSignal {
    pub teams: HashMap<String, Rc<TeamSignal>>,
    pub team_global_placements: RwSignal<Vec<String>>,
}

impl ContestSignal {
    pub fn new(contest_file: &RunningContest) -> Self {
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
            team_global_placements: create_rw_signal(
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
        fresh_contest: &RunningContest,
    ) {
        let update_set: HashSet<_> = team_logins.collect();

        // FIXME traverse redblack in order
        let new_placements = fresh_contest
            .team_placements
            .keys()
            // .sorted_by_cached_key(|team| team.placement_global)
            //.map(|team| team.login.clone())
            .collect_vec();

        for (i, team) in new_placements.iter().enumerate() {
            let placement = i + 1;
            if let Some(team_signal) = self.teams.get(&team.login) {
                if update_set.contains(team.login.as_str()) {
                    team_signal.update(team, placement);
                } else {
                    team_signal.placement_global.set(placement)
                }
            }
        }

        self.team_global_placements
            .set(new_placements.iter().map(|t| t.login.clone()).collect_vec());
    }

    pub fn update_tuples(&self, runs: &[RunTuple], fresh_contest: &RunningContest) {
        self.update(
            runs.iter().map(|run| run.team_login.as_str()),
            fresh_contest,
        )
    }
}
