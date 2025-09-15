use std::{collections::HashMap, hash::Hash};

use futures_signals::signal::{Mutable, Signal};
use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Score {
    solved: u32,
    penalty: u32,
}

impl Score {
    pub fn new(solved: u32, penalty: u32) -> Self {
        Self { solved, penalty }
    }
}

#[derive(Debug)]
pub struct Placements<Key = String> {
    scores: HashMap<Key, Score>,
    placements: HashMap<Key, u32>,
    placements_are_stale: bool,
    placement_signals: HashMap<Key, Mutable<u32>>,
}

impl<Key> Default for Placements<Key> {
    fn default() -> Self {
        Self {
            scores: HashMap::new(),
            placements: HashMap::new(),
            placements_are_stale: false,
            placement_signals: HashMap::new(),
        }
    }
}

impl<Key> Placements<Key>
where
    Key: Ord + Clone + Hash + Eq,
{
    pub fn update(&mut self, team: &Key, Score { solved, penalty }: Score) -> bool {
        let score = self.scores.entry(team.clone()).or_default();
        let new_score = Score { solved, penalty };

        if score != &new_score {
            *score = new_score;
            self.placements_are_stale = true;
            true
        } else {
            false
        }
    }

    pub fn recalculate(&mut self) {
        if !self.placements_are_stale {
            return;
        }

        let mut updated = vec![];

        for (position, (team, _)) in self
            .scores
            .iter()
            .sorted_by_key(|(_, score)| *score)
            .enumerate()
        {
            let new_placement = position as u32 + 1;
            let old_placement = self
                .placements
                .insert(team.clone(), new_placement)
                .unwrap_or_default();

            if old_placement != new_placement {
                updated.push((team.clone(), new_placement));
            }
        }

        for (team, new_placement) in updated {
            self.placement_mutable(&team).set(new_placement);
        }
    }

    pub fn placement_signal(&mut self, team: &Key) -> impl Signal<Item = u32> {
        self.placement_mutable(team).signal()
    }

    fn placement_mutable(&mut self, team: &Key) -> &Mutable<u32> {
        self.placement_signals.entry(team.clone()).or_default()
    }
}
