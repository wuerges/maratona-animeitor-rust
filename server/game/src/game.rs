use std::collections::BTreeMap;

use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
struct Score {
    solved: u32,
    penalty: u32,
}

pub struct Scoreboard<Key = String> {
    scores: BTreeMap<Key, Score>,
    placements: BTreeMap<Key, u32>,
    placements_are_stale: bool,
}

impl<Key> Scoreboard<Key>
where
    Key: Ord + Clone,
{
    pub fn new() -> Self {
        Self {
            scores: Default::default(),
            placements: Default::default(),
            placements_are_stale: false,
        }
    }

    fn update_team(&mut self, team: Key, solved: u32, penalty: u32) {
        let score = self.scores.entry(team).or_default();
        let new_score = Score { solved, penalty };

        if score != &new_score {
            *score = new_score;
            self.placements_are_stale = true;
        }
    }

    pub fn placement(&mut self, team: &Key) -> Option<u32> {
        self.recalculate();
        self.placements.get(team).copied()
    }

    fn recalculate(&mut self) {
        if !self.placements_are_stale {
            return;
        }

        self.placements = self
            .scores
            .iter()
            .sorted_by_key(|(_, score)| *score)
            .enumerate()
            .map(|(position, (team, _))| (team.clone(), position as u32))
            .collect();
    }
}

impl Default for Scoreboard {
    fn default() -> Self {
        Self::new()
    }
}
