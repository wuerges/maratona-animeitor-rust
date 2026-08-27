use std::collections::HashMap;

use data::{Letter, Team};
use itertools::Itertools;
use reactive_graph::{computed::Memo, prelude::*, signal::RwSignal};

use crate::scoring::{ProblemExt, ProblemView, Score, TeamExt};

pub struct TeamSignal {
    pub login: String,
    pub name: String,
    pub escola: String,
    pub placement_global: RwSignal<usize>,
    pub score: RwSignal<Score>,
    pub problems: HashMap<Letter, RwSignal<Option<ProblemView>>>,
}

impl TeamSignal {
    pub fn new(team: &Team, letters: &[Letter]) -> Self {
        let Team {
            login,
            escola,
            name,
            placement: _,
            placement_global,
            problems,
            id: _,
        } = team;

        Self {
            login: login.clone(),
            name: name.clone(),
            escola: escola.clone(),
            placement_global: RwSignal::new(*placement_global),
            score: RwSignal::new(team.score()),
            problems: letters
                .iter()
                .map(|l| {
                    let view = problems.get(l).map(|p| p.view());
                    (l.clone(), RwSignal::new(view))
                })
                .collect(),
        }
    }

    /// Updates the signals of this team from a fresh Team.
    /// Returns whether the placement changed.
    ///
    /// Only signals whose value actually changed are notified: the score
    /// and problem views compare via the id-based equality that `data`
    /// maintains (ids bump on every state change). Callers running inside
    /// a reactive owner must not do plain reads here.
    pub fn update(&self, team: &Team) -> bool {
        let mut placement_changed = false;
        let new_score = team.score();
        self.score.maybe_update(|x| {
            if *x != new_score {
                *x = new_score;
                true
            } else {
                false
            }
        });
        self.placement_global.maybe_update(|p| {
            if *p != team.placement_global {
                *p = team.placement_global;
                placement_changed = true;
                true
            } else {
                false
            }
        });

        for (letter, problem_view) in &self.problems {
            let new_view = team.problems.get(letter).map(|p| p.view());
            problem_view.maybe_update(|v| {
                if *v != new_view {
                    *v = new_view;
                    true
                } else {
                    false
                }
            });
        }

        placement_changed
    }

    pub fn is_resolved(&self) -> Memo<bool> {
        let signals = self.problems.values().cloned().collect_vec();

        Memo::new(move |_| {
            signals
                .iter()
                .all(|p| p.with(move |p| p.as_ref().is_none_or(|p| p.is_resolved())))
        })
    }
}
