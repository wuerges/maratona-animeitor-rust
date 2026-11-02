use super::virtual_rows::{Viewport, WindowRows};
use leptos_use::{use_interval_fn, use_resize_observer};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use data::{configdata::Sede, ContestFile, Letter, TimerData};
use itertools::Itertools;
use leptos::{ev, logging::log, prelude::*};

use client_model::{
    contest_signal::ContestSignal, problem_letters, runs_panel_signal::RunsPanelItemManager,
    team_signal::TeamSignal, TimerDataExt,
};

use crate::views::{
    runs_panel::RunsPanel,
    team_media::{use_global_photo_state, PhotoState, TeamMedia},
    team_score_line::TeamScoreLine,
    timer::Timer,
};

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
    y: ArcRwSignal<f64>,
    initial_y: f64,
) -> impl IntoView {
    let style = move || {
        format!(
            "top: var(--root-top); transform: translateY({}px); --entry-y: {}px; z-index: {};",
            y.get(),
            initial_y,
            -(local_placement.get().unwrap_or(0) as i32)
        )
    };

    let team_login_1 = team.login.clone();
    let team_login = team.login.clone();

    let is_center = Signal::derive(move || match (p_center.get(), local_placement.get()) {
        (Some(c), Some(p)) => c == p,
        _ => false,
    });

    view! {
        <div
            class="run_box scoreboard-row" id=team_login.clone() style={style}
            on:click={move |_| {
                log!("clicked");
                show_photo.update(|s| s.clicked(&team_login_1))}}
        >
            <TeamScoreLine titulo is_center=is_center team=team.clone() sede local_placement />
        </div>
    }
}

#[component]
fn ContestPanelHeader(sede: Signal<Arc<Sede>>, all_problems: Vec<Letter>) -> impl IntoView {
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

    let placements = Memo::new(move |_| {
        sede.with(|s| {
            placements_contest_signal.team_global_placements.with(|t| {
                t.iter()
                    .filter(|login| s.team_belongs_str(login))
                    .cloned()
                    .collect_vec()
            })
        })
    });

    let positions = Memo::new(move |_| {
        Arc::new(placements.with(|order| {
            order
                .iter()
                .enumerate()
                .map(|(i, login)| (login.clone(), i + 1))
                .collect::<HashMap<_, _>>()
        }))
    });
    let p_center = Memo::new(move |_| {
        center.with(|c| {
            c.as_ref()
                .and_then(|login| positions.with(|p| p.get(login).copied()))
        })
    });
    let offset = Memo::new(move |_| p_center.get().unwrap_or(0).saturating_sub(9));
    let probe = NodeRef::<leptos::html::Div>::new();
    let viewport = RwSignal::new(Viewport {
        top: 0.0,
        height: 768.0,
        row_height: 48.0,
    });
    let measure = move || {
        if let Some(probe) = probe.get_untracked() {
            let rect = probe.get_bounding_client_rect();
            let next = Viewport {
                top: -rect.top(),
                height: window()
                    .inner_height()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(768.0),
                row_height: rect.height().max(1.0),
            };
            viewport.maybe_update(|old| {
                if *old == next {
                    false
                } else {
                    *old = next;
                    true
                }
            });
        }
    };
    Effect::new(move |_| {
        probe.get();
        measure();
    });
    let scroll = window_event_listener(ev::scroll, move |_| measure());
    let resize = window_event_listener(ev::resize, move |_| measure());
    let _observer = use_resize_observer(probe, move |_, _| measure());
    on_cleanup(move || {
        scroll.remove();
        resize.remove();
    });

    // Poll only transition expiry and geometry; unchanged values do not notify rows.
    let clock = RwSignal::new(js_sys::Date::now());
    let _interval = use_interval_fn(
        move || {
            measure();
            clock.set(js_sys::Date::now());
        },
        250,
    );
    let rows = RwSignal::new(Vec::<(String, ArcRwSignal<f64>, f64)>::new());
    let state = Rc::new(RefCell::new(WindowRows::default()));
    let mut previous = Arc::new(HashMap::new());
    let mut previous_offset = 0;
    Effect::new(move |_| {
        let current = positions.get();
        let offset = offset.get();
        let ranking_changed =
            !previous.is_empty() && (current != previous || offset != previous_offset);
        let mut state = state.borrow_mut();
        placements.with(|order| {
            state.update(
                order,
                &current,
                &previous,
                viewport.get(),
                (offset, previous_offset),
                ranking_changed,
                clock.get(),
            )
        });
        let mut membership_changed = false;
        rows.update_untracked(|mounted| {
            let old_len = mounted.len();
            mounted.retain(|(login, _, _)| state.rows.contains_key(login));
            membership_changed = mounted.len() != old_len;
            for (login, row) in &state.rows {
                if let Some((_, signal, _)) = mounted.iter().find(|(key, _, _)| key == login) {
                    if signal.get_untracked() != row.y {
                        signal.set(row.y);
                    }
                } else {
                    membership_changed = true;
                    mounted.push((login.clone(), ArcRwSignal::new(row.y), row.initial_y));
                }
            }
        });
        // For's keys preserve row owners across ranking updates.
        if membership_changed {
            rows.notify();
        }
        previous = current;
        previous_offset = offset;
    });
    let row_teams = contest_signal.clone();
    let panel_lines = view! {
        <For each=move || rows.get() key=|(login, _, _)| login.clone()
            children=move |(login, y, initial_y)| {
                let team = row_teams.teams[&login].clone();
                let local_placement = Memo::new(move |_| positions.with(|p| p.get(&login).copied()));
                view! { <ContestPanelLine titulo p_center=p_center.into() local_placement=local_placement.into()
                    team sede show_photo y initial_y /> }
            }
        />
    };
    // One overlay, keyed by selected login, survives scrolling and rank changes.
    let media = move || {
        let selected = match show_photo.get() {
            PhotoState::Hidden => None,
            PhotoState::Show(login) => Some(login),
        };
        selected.and_then(|login| {
            let team = contest_signal.teams.get(&login)?.clone();
            let key = login.clone();
            let local_placement = Memo::new(move |_| positions.with(|p| p.get(&key).copied()));
            Some(view! { <TeamMedia team_login=login show=show_photo team titulo local_placement=local_placement.into() sede /> })
        })
    };

    view! {
        <div class="runstable scoreboard" style:height=move || format!("calc(var(--root-top) + {}px)",
            (placements.with(Vec::len).saturating_sub(offset.get()) + 1) as f64 * viewport.get().row_height)>
            <div node_ref=probe aria-hidden="true" style="position:absolute;top:var(--root-top);height:var(--row-height);width:1px;visibility:hidden;pointer-events:none"></div>
            <div class="run_box" style:top={move || {
                log!("center {:?}", center.get());
                cell_top(0, &None)}}>
                <ContestPanelHeader sede all_problems />
            </div>
            {panel_lines}
            {media}
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
