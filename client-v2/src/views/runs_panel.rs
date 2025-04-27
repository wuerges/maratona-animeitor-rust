use std::sync::Arc;

use data::configdata::Sede;
use leptos::prelude::*;

use crate::{
    model::runs_panel_signal::RunsPanelItemManager,
    views::{placement::Placement, problem::Problem, team_name::TeamName},
};

#[component]
pub fn RunsPanel<'cs>(items: &'cs RunsPanelItemManager, sede: Signal<Arc<Sede>>) -> impl IntoView {
    let panel = items
        .items
        .iter()
        .map(|p| {
            let position = p.position.clone();
            let top = move || {
                format!(
                    "calc(var(--row-height) * {} + var(--root-top))",
                    position.get()
                )
            };
            let panel_item = p.panel_item.clone();

            move || panel_item.with(move |p| {
                p.as_ref().map(move |panel_item| {
                    let problem_view = panel_item.problem_view.clone();
                    view! {
                        <div class="run_box" style:top={top} style:z-index={Signal::derive(move || (-(position.get() as i32)).to_string())}>
                            <div class="run">
                                <Placement placement=panel_item.placement sede />
                                <TeamName escola={panel_item.escola.clone()} name={panel_item.team_name.clone()} />
                                <div class="cell quadrado">{panel_item.problem.clone()}</div>
                                <Problem prob=panel_item.problem.chars().next().unwrap_or('Z') problem=Signal::derive(move || Some(problem_view.clone())) />
                            </div>
                        </div>
                    }
                })
            })
        })
        .collect_view();

    view! {
        <div class="runstable">
            {panel}
        </div>
    }
}
