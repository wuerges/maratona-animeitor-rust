use data::{
    configdata::{Color, Sede},
    BelongsToContest, ContestFile, RunsPanelItem, Team, TimerData,
};
use itertools::Itertools;
use leptos::{logging::log, *};

use crate::views::timer::Timer;

pub fn get_color(n: usize, sede: Option<&Sede>) -> Option<Color> {
    match sede {
        Some(sede) => sede.premio(n),
        None => {
            if n == 0 {
                Some(Color::Red)
            } else if n <= 4 {
                Some(Color::Gold)
            } else if n <= 8 {
                Some(Color::Silver)
            } else if n <= 12 {
                Some(Color::Bronze)
            } else {
                None
            }
        }
    }
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
fn Placement<'a>(
    placement: usize,
    #[prop(optional_no_strip)] sede: Option<&'a Sede>,
) -> impl IntoView {
    let color = get_color(placement, sede);
    let background_color = color.map(get_class).unwrap_or_default();

    view! {
        <div
            // style:background-color=background_color
            class=format!("cell quadrado colocacao {background_color}")
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

#[component]
fn RunResult<'run>(problem: String, answer: &'run data::Answer) -> impl IntoView {
    let balao = format!("balao_{}", problem);
    view! {
        <div class="cell quadrado">{problem}</div>
        {match answer {
            data::Answer::Yes(time) => view! {
                <div class="accept cell quadrado">
                    <div class=format!("accept-img {balao}") />
                    <div class="accept-text cell-content">{*time}</div>
                </div>
            },
            data::Answer::No => view! {
                <div class="unsolved cell quadrado cell-content">
                    <div class="no-img-run" />
                </div>
            },
            data::Answer::Wait => view! {
                <div class="inqueue cell quadrado cell-content">
                    <div class="wait-img-run" />
                </div>
            },
            data::Answer::Unk => view! {
                <div class="inqueue cell quadrado cell-content">
                    <div class="unk-img-run" />
                </div>
            },
        }}
    }
}

#[component]
fn RunsPanel(items: Vec<RunsPanelItem>, #[prop(optional)] sede: Option<Sede>) -> impl IntoView {
    view! {
        <div class="runstable">
        {
            items.iter().take(30).enumerate().map(|(i, r)| {

                let top = format!("calc(var(--row-height) * {} + var(--root-top))", i);
                let problem = r.problem.clone();

                view! {
                    <div class="run" style:top={top} >
                        <Placement placement={r.placement} sede=sede.as_ref() />
                        <TeamName escola={r.escola.clone()} name={r.team_name.clone()} />
                        <RunResult problem answer={&r.result} />
                    </div>
                }
            }).collect_view()
        }
        </div>
    }
}

fn center_class(p: usize, center: &Option<usize>) -> std::option::Option<&str> {
    match center {
        None => None,
        Some(c) => {
            if *c == p {
                Some("center")
            } else {
                None
            }
        }
    }
}

fn number_submissions(s: usize) -> Option<usize> {
    if s == 1 {
        None
    } else {
        Some(s - 1)
    }
}

fn nome_sede(sede: Option<&Sede>) -> &str {
    match sede {
        None => "Placar",
        Some(sede) => sede.entry.name.as_str(),
    }
}

fn estilo_sede(sede: Option<&Sede>) -> Option<&str> {
    sede.and_then(|s| s.entry.style.as_deref())
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
fn Problem<'a>(prob: char, team: &'a Team) -> impl IntoView {
    match team.problems.get(&prob.to_string()) {
        None => view! {<div class="not-tried cell quadrado"> - </div>},
        Some(prob_v) => {
            if prob_v.solved {
                let balao = format!("balao_{}", prob);
                view! {
                    <div class="accept cell quadrado">
                        <div class=format!("accept-img {balao}")></div>
                        <div class="accept-text cell-content">
                            +{number_submissions(prob_v.submissions)}<br />{prob_v.time_solved}
                        </div>
                    </div>
                }
            } else {
                let cell_type = if prob_v.wait() { "inqueue" } else { "unsolved" };
                let cell_symbol = if prob_v.wait() { "?" } else { "X" };

                view! {
                    <div class={format!("cell quadrado {}", cell_type)}>
                        <div class="cima">{cell_symbol}</div>
                        <div class="baixo">"("{prob_v.submissions}")"</div>
                    </div>
                }
            }
        }
    }
}
#[component]
fn ContestPanelLine(
    is_compressed: Signal<bool>,
    i: usize,
    p_center: Option<usize>,
    team: Team,
    all_problems: Signal<&'static str>,
) -> impl IntoView {
    log!("line refresh");
    let score = team.score();
    let local_placement = i + 1;
    view! {
        <div class="run_box" id={team.login.clone()} style={format!("top: {}; z-index: {};", cell_top(local_placement, &p_center), -((local_placement) as i32))}>
            <div class="run">
                <div class={center_class(local_placement, &p_center).iter().chain(&["run_prefix"]).join(" ")}>
                    {move || is_compressed.get().then_some(view! {
                        <Placement placement={team.placement_global} />
                    })}
                    <Placement placement={local_placement} />
                    <TeamName escola={team.escola.clone()} name={team.name.clone()} />
                    <div class="cell problema quadrado">
                        <div class="cima">{score.solved}</div>
                        <div class="baixo">{score.penalty}</div>
                    </div>
                </div>
                {move || all_problems.get().char_indices().map(|(_prob_i, prob)| {
                    view! { <Problem prob team=&team /> }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn ContestPanelHeader<'a>(
    sede: Option<&'a Sede>,
    all_problems: Signal<&'static str>,
) -> impl IntoView {
    log!("header refresh");
    view! {
        <div id="runheader" class="run">
            <div class={estilo_sede(sede).iter().chain(&["cell", "titulo"]).join(" ")}>
                {nome_sede(sede).to_string()}
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
        .find_position(|team| team.login == center)
        .map(|p| p.0 + 1)
}

#[component]
pub fn ContestPanel(
    contest: Signal<ContestFile>,
    center: Signal<Option<String>>,
    sede: Option<Sede>,
) -> impl IntoView {
    log!("contest panel refresh");
    // let p_center = center.as_ref().map(|s| contest.teams[s].placement);
    let n: Signal<usize> = Signal::derive(move || contest.get().number_problems);
    let all_problems = Signal::derive(move || &data::PROBLEM_LETTERS[..n.get()]);
    let cloned_sede = sede.clone();
    let is_compressed = Signal::derive(move || {
        contest
            .get()
            .teams
            .values()
            .any(|team| !team.belongs_to_contest(cloned_sede.as_ref()))
    });

    let cloned_sede = sede.clone();
    let teams = Signal::derive(move || {
        contest
            .get()
            .teams
            .into_values()
            .filter(|team| team.belongs_to_contest(cloned_sede.as_ref()))
            .sorted_by_cached_key(|team| team.score())
            .collect_vec()
    });

    let p_center = Signal::derive(move || {
        center
            .get()
            .and_then(|center| find_center(&center, &teams.get()))
    });

    view! {
        <div class="runstable">
            <div class="run_box" style:top={move || cell_top(0, &p_center.get())}>
                <ContestPanelHeader sede=sede.as_ref() all_problems />
            </div>

            {move || teams.get().into_iter().enumerate().map(|(i, team)| {
                view! {
                    <ContestPanelLine is_compressed i p_center=p_center.get() team all_problems />
                }
            }).collect_view()}
        </div>
    }
}

#[component]
fn EmptyContestPanel<'a>(sede: Option<&'a Sede>) -> impl IntoView {
    view! {
        <div class="runstable">
            <div class="run_box" style:top={cell_top(0, &None)}>
                <ContestPanelHeader sede=sede all_problems=create_signal("").0.into() />
            </div>
        </div>
    }
}

#[component]
pub fn Contest(
    contest: Signal<ContestFile>,
    panel_items: ReadSignal<Vec<RunsPanelItem>>,
    timer: ReadSignal<(TimerData, TimerData)>,
    #[prop(optional)] sede: Option<Sede>,
) -> impl IntoView {
    let panel_items = panel_items.get();

    let (center, _) = create_signal(None);

    view! {
        <body style="height: 1px">
            <div style="display: flex; width: 320px;">
                <div style="display: flex; flex-direction: column; width: 320px;">
                    <Timer timer />
                    <div class="submission-title"> Últimas Submissões </div>
                    <RunsPanel items=panel_items />
                </div>
                <div class="automatic" style="margin-left: 8px;">
                    <ContestPanel contest center=center.into() sede=sede.clone() />
                </div>
            </div>
        </body>
    }
}
