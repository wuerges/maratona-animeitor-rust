use data::configdata::Sede;
use itertools::Itertools;
use leptos::prelude::*;
use std::sync::Arc;

use crate::{
    model::team_signal::TeamSignal,
    views::{placement::Placement, problem::Problem, team_name::TeamName},
};

#[component]
pub fn TeamScoreLine(
    team: Arc<TeamSignal>,
    is_center: Signal<bool>,
    titulo: Signal<Option<Arc<Sede>>>,
    local_placement: Signal<Option<usize>>,
    sede: Signal<Arc<Sede>>,
) -> impl IntoView {
    let escola = team.escola.clone();
    let name = team.name.clone();
    let score = team.score.clone();

    let problems = team.problems.clone();
    let problems = problems
        .into_iter()
        .sorted_by_cached_key(|(letter, _problem)| letter.clone())
        .map(|(letter, problem)| {
            let memo_problem = Memo::new(move |_| problem.get());
            move || view! { <Problem prob=letter.chars().next().unwrap() problem=memo_problem.into() /> }
        })
        .collect_view();

    let placement_global = team.placement_global;

    view! {
        <div class="run">
            <div class:run_prefix=true class:center=is_center >
                {move || {
                    let placement = placement_global.get();
                    titulo.with(move |t| t.clone().map(move |t| view! {
                        <Placement placement sede=t.clone().into() />
                    }))
                }}
                {move || local_placement.get().map(|placement|
                    view!{ <Placement placement sede /> }
                )}
                <TeamName escola name />
                <div class="cell problema quadrado">
                    <div class="cima">{move || score.with(|s| s.solved)}</div>
                    <div class="baixo">{move || score.with(|s| s.penalty)}</div>
                </div>
            </div>
            {problems}
        </div>
    }
}
