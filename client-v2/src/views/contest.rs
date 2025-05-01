use std::sync::Arc;

use data::{configdata::Sede, problem_letters, ContestFile, Letter, TimerData};
use itertools::Itertools;
use leptos::{ev, logging::log, prelude::*};

use crate::{
    model::{
        contest_signal::ContestSignal, runs_panel_signal::RunsPanelItemManager,
        team_signal::TeamSignal,
    },
    views::{
        compress_placements::compress_placements,
        runs_panel::RunsPanel,
        team_media::{use_global_photo_state, PhotoState, TeamMedia},
        team_score_line::TeamScoreLine,
        timer::Timer,
    },
};

use super::compress_placements::Compress;

fn nome_sede(sede: &Sede) -> &str {
    sede.entry.name.as_str()
}

fn estilo_sede(sede: &Sede) -> Option<&str> {
    sede.entry.style.as_deref()
}

fn cell_top(i: usize, center: &Option<usize>) -> String {
    let i = i as i64;
    match center {
        None => format!("calc(var(--row-height) * {} + var(--root-top))", i),
        Some(p) => {
            let p = *p as i64;
            if p < 9 {
                format!("calc(var(--row-height) * {} + var(--root-top))", i)
            } else {
                format!("calc(var(--row-height) * {} + var(--root-top))", i + 9 - p)
            }
        }
    }
}

#[component]
fn ContestPanelLine(
    titulo: Signal<Option<Arc<Sede>>>,
    p_center: Signal<Option<usize>>,
    local_placement: Signal<Option<usize>>,
    team: Arc<TeamSignal>,
    sede: Signal<Arc<Sede>>,
    show_photo: RwSignal<PhotoState>,
) -> impl IntoView {
    let memo_placement = Memo::new(move |_| local_placement.get());
    let style = move || {
        memo_placement.with(|t| match t {
            Some(placement) => format!(
                "top: {}; z-index: {};",
                cell_top(*placement, &p_center.get()),
                -(*placement as i32)
            ),
            None => "display: none;".to_string(),
        })
    };

    let team_login_1 = team.login.clone();
    let team_login = team.login.clone();

    let is_center = Signal::derive(move || match (p_center.get(), local_placement.get()) {
        (Some(c), Some(p)) => c == p,
        _ => false,
    });

    view! {
        <div
            class="run_box" id=team_login.clone() style={style}
            on:click={move |_| {
                log!("clicked");
                show_photo.update(|s| s.clicked(&team_login_1))}}
        >
            <TeamScoreLine titulo is_center=is_center team=team.clone() sede local_placement />
        </div>
        <TeamMedia team_login show={show_photo} team titulo local_placement sede />
    }
}

#[component]
fn ContestPanelHeader(sede: Signal<Arc<Sede>>, all_problems: Vec<Letter>) -> impl IntoView {
    log!("header refresh {:?}", all_problems);
    view! {
        <div id="runheader" class="run">
            <div class={move ||
                estilo_sede(&sede.get()).iter().chain(&["cell", "titulo"]).join(" ")}>
                {move || nome_sede(&sede.get()).to_string()}
            </div>
            {all_problems.into_iter().map(|p| view! {
                <div class="cell problema quadrado">{p.to_string()}</div>
            }).collect_view()}
        </div>
    }
}

struct ContestPanelLineWrap {
    titulo: Signal<Option<Arc<Sede>>>,
    team: Arc<TeamSignal>,
    sede: Signal<Arc<Sede>>,
    show_photo: RwSignal<PhotoState>,
}

impl Compress for ContestPanelLineWrap {
    type Key = String;

    fn key(&self) -> Signal<Option<String>> {
        Some(self.team.login.clone()).into()
    }

    fn view_in_position(
        self,
        position: Signal<Option<usize>>,
        center: Signal<Option<usize>>,
    ) -> impl IntoView {
        let Self {
            titulo,
            team,
            sede,
            show_photo,
        } = self;
        view! {
            <ContestPanelLine
                titulo
                p_center=center
                local_placement=position
                team=team.clone()
                sede
                show_photo
            />
        }
    }
}

#[component]
pub fn ContestPanel(
    original_contest: Arc<ContestFile>,
    contest_signal: Arc<ContestSignal>,
    center: Signal<Option<String>>,
    titulo: Signal<Option<Arc<Sede>>>,
    sede: Signal<Arc<Sede>>,
) -> impl IntoView {
    log!("contest panel refresh");

    let all_problems = problem_letters(original_contest.number_problems);

    let show_photo = use_global_photo_state();

    let handle = window_event_listener(ev::keydown, move |ev| match ev.code().as_str() {
        "KeyY" => {
            center.with(|center| match center {
                Some(center) => show_photo.update(|s| s.clicked(center)),
                None => show_photo.update(|s| s.hide()),
            });
        }
        code => log!("ev code: {code}"),
    });
    on_cleanup(move || handle.remove());

    Effect::new(move |_| {
        center.with(|center| match center {
            Some(center) => show_photo.update(|s| match s {
                PhotoState::Hidden => (),
                PhotoState::Show(old) => {
                    if old != center {
                        s.clicked(center)
                    }
                }
            }),
            None => show_photo.update(|s| s.hide()),
        })
    });

    let placements_contest_signal = contest_signal.clone();

    let placements = Signal::derive(move || {
        sede.with(|s| {
            placements_contest_signal.team_global_placements.with(|t| {
                t.into_iter()
                    .filter(|login| s.team_belongs_str(&login))
                    .cloned()
                    .collect_vec()
            })
        })
    });

    let panel_lines = compress_placements(
        contest_signal
            .teams
            .values()
            .map(|team| ContestPanelLineWrap {
                titulo,
                team: team.clone(),
                sede: sede.clone(),
                show_photo,
            })
            .collect_vec(),
        placements,
        center,
    );

    view! {
        <div class="runstable">
            <div class="run_box" style:top={move || {
                log!("center {:?}", center.get());
                cell_top(0, &None)}}>
                <ContestPanelHeader sede all_problems />
            </div>
            {panel_lines}
        </div>
    }
}

#[component]
pub fn Contest(
    original_contest: Arc<ContestFile>,
    contest_signal: Arc<ContestSignal>,
    panel_items: Arc<RunsPanelItemManager>,
    timer: ReadSignal<(TimerData, TimerData)>,
    titulo: Signal<Option<Arc<Sede>>>,
    sede: Signal<Arc<Sede>>,
) -> impl IntoView {
    let (center, _) = signal(None);

    let is_frozen = Signal::derive(move || timer.with(|(current, _)| current.is_frozen()));

    view! {
        <div class="root-container" class:is-frozen=is_frozen>
            <div class="submissions-container">
                <Timer timer />
                <div class="submission-title"> Últimas Submissões </div>
                <RunsPanel items=panel_items sede />
            </div>
            <div class="contest-container">
                <ContestPanel original_contest contest_signal=contest_signal center=center.into() titulo sede />
            </div>
        </div>
    }
}
