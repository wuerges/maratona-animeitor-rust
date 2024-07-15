use std::{collections::HashMap, rc::Rc};

use data::{configdata::Sede, ContestFile, TimerData};
use itertools::Itertools;
use leptos::{logging::log, *};

use crate::{
    model::{runs_panel_signal::RunsPanelItemManager, ContestSignal, TeamSignal},
    views::{
        placement::Placement,
        problem::Problem,
        team_media::{PhotoState, TeamMedia},
        team_name::TeamName,
        team_score_line::TeamScoreLine,
        timer::Timer,
    },
};

#[component]
fn RunsPanel<'cs>(items: &'cs RunsPanelItemManager, sede: Signal<Rc<Sede>>) -> impl IntoView {
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
                        <div class="run_box" style:top={top} style:z-index={move || -(position.get() as i32)}>
                            <div class="run">
                                <Placement placement=panel_item.placement sede />
                                <TeamName escola={panel_item.escola.clone()} name={panel_item.team_name.clone()} />
                                <div class="cell quadrado">{panel_item.problem.clone()}</div>
                                <Problem prob=panel_item.problem.chars().next().unwrap_or('Z') problem=(move || Some(problem_view.clone())).into_signal() />
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
    titulo: Signal<Option<Rc<Sede>>>,
    p_center: Signal<Option<usize>>,
    local_placement: Signal<Option<usize>>,
    team: Rc<TeamSignal>,
    sede: Signal<Rc<Sede>>,
    show_photo: RwSignal<PhotoState>,
) -> impl IntoView {
    let memo_placement = create_memo(move |_| local_placement.get());
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

    let is_center = move || match (p_center.get(), local_placement.get()) {
        (Some(c), Some(p)) => c == p,
        _ => false,
    };

    view! {
        <div
            class="run_box" id=team_login.clone() style={style}
            on:click={move |_| {
                log!("clicked");
                show_photo.update(|s| s.clicked(&team_login_1))}}
        >
            <TeamScoreLine titulo is_center=is_center.into_signal() team=team.clone() sede local_placement />
        </div>
        <TeamMedia team_login show={show_photo} team titulo local_placement sede />
    }
}

#[component]
fn ContestPanelHeader(sede: Signal<Rc<Sede>>, all_problems: Signal<&'static str>) -> impl IntoView {
    log!("header refresh");
    view! {
        <div id="runheader" class="run">
            <div class={move ||
                estilo_sede(&sede.get()).iter().chain(&["cell", "titulo"]).join(" ")}>
                {move || nome_sede(&sede.get()).to_string()}
            </div>
            {move || all_problems.get().chars().map(|p| view! {
                <div class="cell problema quadrado">{p}</div>
            }).collect_view()}
        </div>
    }
}

#[component]
pub fn ContestPanel(
    contest: Signal<ContestFile>,
    contest_signal: Rc<ContestSignal>,
    center: Signal<Option<String>>,
    titulo: Signal<Option<Rc<Sede>>>,
    sede: Signal<Rc<Sede>>,
) -> impl IntoView {
    log!("contest panel refresh");
    let n: Signal<usize> = Signal::derive(move || contest.with(|c| c.number_problems));
    let all_problems = Signal::derive(move || &data::PROBLEM_LETTERS[..n.get()]);

    let show_photo = create_rw_signal(PhotoState::default());

    let placements = create_memo(move |_| {
        sede.with(|sede| {
            contest.with(|c| {
                c.teams
                    .values()
                    .filter(|team| sede.team_belongs(team))
                    .sorted_by_cached_key(|team| team.placement_global)
                    .map(|team| &team.login)
                    .enumerate()
                    .map(|(i, login)| (login.clone(), i + 1))
                    .collect::<HashMap<_, _>>()
            })
        })
    });

    let p_center = (move || {
        placements.with(|placements| {
            center.with(|center| {
                center
                    .as_ref()
                    .and_then(|center| placements.get(center).copied())
            })
        })
    })
    .into_signal();

    let panel_lines = contest_signal
        .teams
        .values()
        .map(move |team| {
            let login = team.login.clone();
            let local_placement =
                (move || placements.with(|ps| ps.get(&login).copied())).into_signal();
            view! {
                <ContestPanelLine
                    titulo
                    p_center=p_center.into()
                    local_placement
                    team=team.clone()
                    sede
                    show_photo
                />
            }
        })
        .collect_view();

    view! {
        <div class="runstable">
            <div class="run_box" style:top={move || {
                log!("center {:?} {:?}", center.get(), p_center.get());
                cell_top(0, &p_center.get())}}>
                <ContestPanelHeader sede all_problems />
            </div>
            {panel_lines}
        </div>
    }
}

#[component]
fn EmptyContestPanel(sede: Signal<Rc<Sede>>) -> impl IntoView {
    view! {
        <div class="runstable">
            <div class="run_box" style:top={cell_top(0, &None)}>
                <ContestPanelHeader sede all_problems=create_signal("").0.into() />
            </div>
        </div>
    }
}

#[component]
pub fn Contest<'cs>(
    contest: Signal<ContestFile>,
    contest_signal: Rc<ContestSignal>,
    panel_items: &'cs RunsPanelItemManager,
    timer: ReadSignal<(TimerData, TimerData)>,
    titulo: Signal<Option<Rc<Sede>>>,
    sede: Signal<Rc<Sede>>,
) -> impl IntoView {
    let (center, _) = create_signal(None);

    let is_frozen = (move || timer.with(|(current, _)| current.is_frozen())).into_signal();

    view! {
        <div class="root-container" class:is-frozen=is_frozen>
            <div class="submissions-container">
                <Timer timer />
                <div class="submission-title"> Últimas Submissões </div>
                <RunsPanel items=panel_items sede />
            </div>
            <div class="contest-container">
                <ContestPanel contest contest_signal=contest_signal center=center.into() titulo sede />
            </div>
        </div>
    }
}
