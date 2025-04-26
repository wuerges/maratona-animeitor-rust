use std::collections::HashMap;

use data::{ProblemView, Score, Team};
use leptos::prelude::*;

pub struct TeamSignal {
    pub login: String,
    pub name: String,
    pub escola: String,
    pub placement_global: RwSignal<usize>,
    pub score: RwSignal<Score>,
    pub problems: HashMap<String, RwSignal<Option<ProblemView>>>,
}

impl TeamSignal {
    pub fn new(team: &Team, letters: &[String]) -> Self {
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

    pub fn update(&self, team: &Team) {
        let new_score = team.score();
        self.score.update(|x| *x = new_score);
        self.placement_global.update(|p| *p = team.placement_global);

        for (letter, problem_view) in &self.problems {
            problem_view.update(|v| *v = team.problems.get(letter).map(|p| p.view()))
        }
    }
}
