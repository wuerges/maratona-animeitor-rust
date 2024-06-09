use std::{collections::HashMap, rc::Rc};

use data::{
    configdata::{Color, Sede},
    ContestFile, RunsPanelItem, Team, TimerData,
};
use itertools::Itertools;
use leptos::{logging::log, *};

use crate::{
    model::{ContestSignal, TeamSignal},
    views::{
        photos::{PhotoState, TeamPhoto},
        timer::Timer,
    },
};

pub fn get_color(n: usize, sede: &Sede) -> Option<Color> {
    sede.premio(n)
}

fn get_class(color: Color) -> &'static str {
    match color {
        Color::Red => "vermelho",
        Color::Gold => "ouro",
        Color::Silver => "prata",
        Color::Bronze => "bronze",
        Color::Green => "verde",
        Color::Yellow => "amarelo",
    }
}

#[component]
fn Placement(placement: usize, sede: Signal<Rc<Sede>>) -> impl IntoView {
    let background_color = (move || sede.with(|sede| get_color(placement, sede).map(get_class).unwrap_or_default())).into_signal();

    view! {
        <div
        // style:background-color=background_color
        class=move || format!("cell quadrado colocacao {}", background_color.get())
        >
        {placement}
        </div>
    }
}

#[component]
fn TeamName(escola: String, name: String) -> impl IntoView {
    view! {
        <div class="cell time">
            <div class="nomeEscola">{escola}</div>
            <div class="nomeTIme">{name}</div>
        </div>
    }
}

fn take_30(
    items: Vec<RunsPanelItem>,
    sede: Rc<Sede>,
) -> impl IntoIterator<Item = (usize, RunsPanelItem)> {
    items
        .into_iter()
        .filter(move |p| sede.team_belongs_str(&p.team_login))
        .take(30)
        .enumerate()
}

#[component]
fn RunsPanel(items: Signal<Vec<RunsPanelItem>>, sede: Signal<Rc<Sede>>) -> impl IntoView {
    view! {
        <div class="runstable">
        <For
        each=move || take_30(items.get(), sede.get())
        key=|(_, r)| r.id
        children={move |(i, o)| {
                let sede = sede.clone();
                let top = format!("calc(var(--row-height) * {} + var(--root-top))", i);

                view! {
                    <div class="run" style:top={top} >
                        <Placement placement={o.placement.into()} sede />
                        <TeamName escola={o.escola.clone()} name={o.team_name.clone()} />
                        <div class="cell quadrado">{o.problem.clone()}</div>
                        <Problem prob=o.problem.chars().next().unwrap_or('Z') problem=Some(o.problem_view) />
                    </div>
                }
            }}
        />
        </div>
    }
}

fn number_submissions(s: usize) -> Option<usize> {
    if s == 1 {
        None
    } else {
        Some(s - 1)
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
fn Problem(prob: char, problem: Option<data::ProblemView>) -> impl IntoView {
    // log!("rendered problem {:?}", problem);
    view! {
            <div class={match &problem {
                Some(p) => if p.solved && p.solved_first {
                    "star cell quadrado".to_string()
                } else if p.solved {
                    "accept cell quadrado".to_string()
                } else {
                    let cell_type = if p.wait { "inqueue"} else { "unsolved" };
                    format!("cell quadrado {cell_type}")
                },
                None => "not-tried cell quadrado".to_string(),
            }}>
            {match &problem {
                Some(p) => {
                    (if p.solved {
                        let balao = format!("balao_{}", prob);
                        let img = if p.solved_first { "star-img"} else { "accept-img" };
                        view! {
                            <div class=format!("{img} {balao}")></div>
                            <div class="accept-text cell-content">
                                +{number_submissions(p.submissions)}<br />{p.time_solved}
                            </div>
                        }
                    } else {
                        let cell_symbol = if p.wait { "?" } else { "X" };

                        view! {
                            <div class="cima">{cell_symbol}</div>
                            <div class="baixo">"("{p.submissions}")"</div>
                        }
                    }).into_view()

                },
                None => {
                    {"-"}.into_view()
                },
            }}
            </div>
    }
}

#[component]
fn ContestPanelLine<'cs>(
    is_compressed: Signal<bool>,
    p_center: Signal<Option<usize>>,
    local_placement: Signal<Option<usize>>,
    team: &'cs TeamSignal,
    sede: Signal<Rc<Sede>>,
) -> impl IntoView {
    let style = move || {
        local_placement.with(|t| match t {
            Some(placement) => format!(
                "top: {}; z-index: {};",
                cell_top(*placement, &p_center.get()),
                -(*placement as i32)
            ),
            None => "display: none;".to_string(),
        })
    };

    // let center = move || {
    //     team.with(|t| {
    //         p_center
    //             .get()
    //             .and_then(|c| (c == t.placement).then_some("center"))
    //             .iter()
    //             .chain([&"run_prefix"])
    //             .join(" ")
    //     })
    // };

    let show_photo = create_rw_signal(PhotoState::default());

    let problems = team.problems.clone();
    let problems = 
        problems
        .into_iter()
        .map(|(letter, problem)| {
            move || view! { <Problem prob=letter.chars().next().unwrap() problem=problem.get() /> }
        })
        .collect_view();

    let escola= team.escola.clone();
    let name = team.name.clone();
    let team_login = team.login.clone();
    let score = team.score.clone();
    let placement_global = team.placement_global.clone();

    
    view! {
        <div
            class="run_box" id=team_login.clone() style={style}
            on:click={move |_| {
                log!("clicked");
                show_photo.update(|s| s.clicked())}}
        >
            <div class="run">
                <div class:run_prefix=true >
                    {move || {
                        let placement = placement_global.get();
                        is_compressed.get().then_some(view! {
                            <Placement placement sede />
                        })
                    }}
                    {move || local_placement.get().map(|placement|
                        view!{ <Placement placement sede /> }
                    )}
                    <TeamName escola name />
                    <div class="cell problema quadrado">
                        <div class="cima">{move || score.get().solved}</div>
                        <div class="baixo">{move || score.get().penalty}</div>
                    </div>
                </div>
            {problems}
            </div>
        </div>
        <TeamPhoto team_login show={show_photo} />
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

fn find_center(center: &str, teams: &[Team]) -> Option<usize> {
    teams
        .iter()
        .find(|team| team.login == center)
        .map(|t| t.placement)
}

#[component]
pub fn ContestPanel<'cs>(
    contest: Signal<ContestFile>,
    contest_signal: &'cs ContestSignal,
    center: Signal<Option<String>>,
    sede: Signal<Rc<Sede>>,
) -> impl IntoView {
    log!("contest panel refresh");
    let n: Signal<usize> = Signal::derive(move || contest.with(|c| c.number_problems));
    let all_problems = Signal::derive(move || &data::PROBLEM_LETTERS[..n.get()]);
    let is_compressed = create_memo(move |_| {
        sede.with(|sede| {
            contest.with(|c| c.teams.values().any(|team| !sede.team_belongs(team)))            
        })
    });

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

    // let p_center = (move || {
    //     {
    //         teams.with(|t| {
    //             center.with(|center| center.as_ref().and_then(|center| find_center(&center, t)))
    //         })
    //     }
    // })
    // .into_signal();

    let p_center = create_rw_signal(None::<usize>);

    let panel_lines = contest_signal
        .teams
        .values()
        .map(move |team| {
            let login = team.login.clone();
            let local_placement =
                (move || placements.with(|ps| ps.get(&login).copied())).into_signal();
            view! {
                <ContestPanelLine
                    is_compressed=is_compressed.into()
                    p_center=p_center.into()
                    local_placement
                    team
                    sede
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
    contest_signal: &'cs ContestSignal,
    panel_items: ReadSignal<Vec<RunsPanelItem>>,
    timer: ReadSignal<(TimerData, TimerData)>,
    sede: Signal<Rc<Sede>>,
) -> impl IntoView {
    let (center, _) = create_signal(None);

    view! {
        <div style="display: flex; width: 320px;">
            <div style="display: flex; flex-direction: column; width: 320px;">
                <Timer timer />
                <div class="submission-title"> Últimas Submissões </div>
                {move || view!{<RunsPanel items=panel_items.into() sede />}}
            </div>
            <div class="automatic" style="margin-left: 8px;">
                <ContestPanel contest contest_signal=contest_signal center=center.into() sede />
            </div>
        </div>
    }
}
