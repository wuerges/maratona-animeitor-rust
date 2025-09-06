use std::sync::Arc;

use data::{configdata::Sede, RunsPanelItem};
use itertools::Itertools;
use leptos::{logging::log, prelude::*};

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
    type Key = u64;

    fn key(&self) -> Signal<Option<Self::Key>> {
        let sig = self.panel_item;
        Signal::derive(move || sig.with(|item| item.as_ref().map(|p| p.order)))
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
                order: _,
                problem,
                problem_view,
            } = panel_item.get()?;
            let problem_view = problem_view.clone();
            let position = position.get()? as i32;
            let top = format!(
                "calc(var(--row-height) * {} + var(--root-top))",
                position - 1
            );
            let z_index = Signal::derive(move || (-position).to_string());

            Some(view! {
                <div class="run_box" style:top={top} style:z-index={z_index}>
                    <div class="run">
                        <Placement placement=placement sede />
                        <TeamName escola={escola.clone()} name={team_name.clone()} />
                        <div class="cell quadrado">{problem.to_string()}</div>
                        <Problem prob=problem problem=Signal::derive(move || Some(problem_view.clone())) />
                    </div>
                </div>
            })
        }
    }
}

#[component]
pub fn RunsPanel(items: Arc<RunsPanelItemManager>, sede: Signal<Arc<Sede>>) -> impl IntoView {
    move || {
        let placements = items.placements_for_sede(sede);

        log!("runs_panel");

        let wraps = items
            .items
            .iter()
            .map(|item_signal| ItemWrap {
                panel_item: item_signal.panel_item.into(),
                sede,
            })
            .collect_vec();
        let panel = compress_placements(wraps, placements, None.into());

        view! {
            <div class="runstable">
                {panel}
            </div>
        }
    }
}
