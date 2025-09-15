use std::{
    collections::{BTreeMap, HashMap, HashSet},
    hash::Hash,
};

use futures_signals::signal::{Mutable, Signal};

use crate::scoreboard::{Placements, Score};

pub trait GetSites {
    type SiteName;
    type TeamName;
    fn sites(&self) -> impl Iterator<Item = &Self::SiteName>;
    fn name(&self) -> &Self::TeamName;
    fn score(&self) -> Score;
}

pub struct Game<Team>
where
    Team: GetSites,
{
    placements: HashMap<Team::SiteName, Placements<Team::TeamName>>,
    score_signals: HashMap<Team::TeamName, Mutable<Score>>,
    scores: BTreeMap<Team::TeamName, Score>,
}

impl<Team> Game<Team>
where
    Team: GetSites,
    Team::SiteName: Hash + Eq + Clone,
    Team::TeamName: Hash + Eq + Clone + Ord,
{
    pub fn update<'t>(&mut self, teams: impl Iterator<Item = &'t Team>)
    where
        Team: 't,
    {
        let mut updated_teams = vec![];
        let mut update_sites = HashSet::new();

        for team in teams {
            let new_score = team.score();

            if self.update_score(team.name(), new_score) {
                updated_teams.push((team, new_score));

                for site in team.sites() {
                    if let Some(site_placements) = self.placements.get_mut(site)
                        && site_placements.update(team.name(), new_score)
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

        for (team, new_score) in updated_teams {
            self.score_mutable(team).set(new_score);
        }
    }

    fn update_score(&mut self, team_name: &Team::TeamName, new_score: Score) -> bool {
        let old_score = self.scores.insert(team_name.clone(), new_score);

        old_score.is_none_or(|old| old != new_score)
    }

    pub fn score_signal(&mut self, team: &Team) -> impl Signal<Item = Score> {
        self.score_mutable(team).signal()
    }

    pub fn placement_signal(
        &mut self,
        team: &Team,
        site: &Team::SiteName,
    ) -> impl Signal<Item = u32> {
        self.placements
            .entry(site.clone())
            .or_default()
            .placement_signal(team.name())
    }

    fn score_mutable(&mut self, team: &Team) -> &Mutable<Score> {
        self.score_signals.entry(team.name().clone()).or_default()
    }
}
