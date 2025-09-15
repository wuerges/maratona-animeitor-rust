use std::collections::HashMap;

use crate::{
    Sites,
    sites::TeamSites,
    team::{ContestService, Team},
};

pub struct Contest {
    teams: HashMap<String, Team>,
    sites: Sites<Team>,
}

impl Contest {
    fn push_run(&mut self, new_run: sdk::Run) {
        if let Some(team) = self.teams.get_mut(&new_run.team_login) {
            team.push_run(new_run);
        }
    }

    fn pop_run(&mut self, contest: &mut impl ContestService) -> bool {
        todo!()
    }

    pub fn judge_run_batch(&mut self, runs: &[sdk::Run], contest: &mut impl ContestService) {
        let mut teams: HashMap<String, &Team> = HashMap::new();
        for run in runs {
            if let Some(team) = self.teams.get_mut(&run.team_login) {
                team.judge_run(run, contest);
            }
        }
        for run in runs {
            if let Some(team) = self.teams.get(&run.team_login) {
                teams.insert(team.login().to_string(), team);
            }
        }
        self.sites.update(teams.values().copied());
    }

    pub fn new(
        sdk::SiteConfiguration { base, sites }: &sdk::SiteConfiguration,
        sdk::ContestParameters {
            teams,
            maximum_time_in_minutes,
            score_freeze_time_in_minutes,
            penalty_per_wrong_answer,
            problem_letters,
        }: &sdk::ContestParameters,
    ) -> Self {
        Self {
            teams: teams
                .iter()
                .map(|t| {
                    (
                        t.login.clone(),
                        Team::new(t.clone(), sites, problem_letters),
                    )
                })
                .collect(),
            sites: (),
        }
    }
}
