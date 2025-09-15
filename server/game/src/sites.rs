use std::{
    collections::{BTreeMap, HashMap, HashSet},
    hash::Hash,
};

use futures_signals::signal::Signal;

use crate::scoreboard::{Placements, Score};

pub trait TeamSites {
    type Site;
    type Login;
    fn sites(&self) -> impl Iterator<Item = &Self::Site>;
    fn login(&self) -> &Self::Login;
    fn score(&self) -> Score;
}

pub struct Game<Team>
where
    Team: TeamSites,
{
    placements: HashMap<Team::Site, Placements<Team::Login>>,
    scores: BTreeMap<Team::Login, Score>,
}

impl<Team> Game<Team>
where
    Team: TeamSites,
    Team::Site: Hash + Eq + Clone,
    Team::Login: Hash + Eq + Clone + Ord,
{
    pub fn update<'t>(&mut self, teams: impl Iterator<Item = &'t Team>)
    where
        Team: 't,
    {
        let mut updated_teams = vec![];
        let mut update_sites = HashSet::new();

        for team in teams {
            let new_score = team.score();

            if self.update_score(team.login(), new_score) {
                updated_teams.push((team, new_score));

                for site in team.sites() {
                    if let Some(site_placements) = self.placements.get_mut(site)
                        && site_placements.update(team.login(), new_score)
                    {
                        update_sites.insert(site);
                    }
                }
            }
        }

        for site in update_sites {
            if let Some(placements) = self.placements.get_mut(site) {
                placements.recalculate();
            }
        }
    }

    fn update_score(&mut self, team_name: &Team::Login, new_score: Score) -> bool {
        let old_score = self.scores.insert(team_name.clone(), new_score);

        old_score.is_none_or(|old| old != new_score)
    }

    pub fn placement_signal(&mut self, team: &Team, site: &Team::Site) -> impl Signal<Item = u32> {
        self.placements
            .entry(site.clone())
            .or_default()
            .placement_signal(team.login())
    }
}
