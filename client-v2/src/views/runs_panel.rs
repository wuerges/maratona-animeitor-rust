use std::sync::Arc;

use data::{configdata::Sede, RunsPanelItem};
use itertools::Itertools;
use leptos::prelude::*;

use crate::{
    model::runs_panel_signal::RunsPanelItemManager,
    views::{
        compress_placements::compress_placements, placement::Placement, problem::Problem,
        team_name::TeamName,
    },
};

use super::compress_placements::Compress;

struct ItemWrap {
    panel_item: Signal<Option<RunsPanelItem>>,
    sede: Signal<Arc<Sede>>,
}

impl Compress for ItemWrap {
    type Key = i64;

    fn key(&self) -> Self::Key {
        // FIXME
        self.panel_item
            .with_untracked(|p| p.as_ref().map(|p| p.id).unwrap_or_default())
    }

    fn view_in_position(
        self,
        position: Signal<Option<usize>>,
        _center: Signal<Option<usize>>,
    ) -> impl IntoView {
        let ItemWrap { panel_item, sede } = self;
        move || {
            let RunsPanelItem {
                id: _,
                placement,
                escola,
                team_name,
                team_login: _,
                problem,
                problem_view,
            } = panel_item.get()?;
            let problem_view = problem_view.clone();
            let position = position.get()?;
            let top = format!("calc(var(--row-height) * {} + var(--root-top))", position);
            let z_index = Signal::derive(move || (-(position as i32)).to_string());

            Some(view! {
                <div class="run_box" style:top={top} style:z-index={z_index}>
                    <div class="run">
                        <Placement placement=placement sede />
                        <TeamName escola={escola.clone()} name={team_name.clone()} />
                        <div class="cell quadrado">{problem.clone()}</div>
                        <Problem prob=problem.chars().next().unwrap_or('Z') problem=Signal::derive(move || Some(problem_view.clone())) />
                    </div>
                </div>
            })
        }
    }
}

#[component]
pub fn RunsPanel<'cs>(items: &'cs RunsPanelItemManager, sede: Signal<Arc<Sede>>) -> impl IntoView {
    let placements = items.placements_for_sede(sede);
    let wraps = items
        .items
        .iter()
        .filter_map(|item| {
            Some(ItemWrap {
                panel_item: item.panel_item.into(),
                sede: sede.clone(),
            })
        })
        .collect_vec();

    let panel = compress_placements(wraps, placements, None.into());

    view! {
        <div class="runstable">
            {panel}
        </div>
    }
}
