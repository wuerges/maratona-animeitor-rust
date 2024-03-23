use data::{
    configdata::{Color, Sede},
    BelongsToContest, ContestFile, RunsPanelItem, Team, TimerData,
};
use itertools::Itertools;
use leptos::*;

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
                format!(
                    "calc(var(--row-height) * {} + var(--root-top-center))",
                    (i - p)
                )
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
fn ContestPanelLine<'a>(
    is_compressed: bool,
    i: usize,
    p_center: Option<usize>,
    team: &'a Team,
    all_problems: &'static str,
) -> impl IntoView {
    let score = team.score();
    let local_placement = i + 1;
    view! {
        <div class="run_box" style={format!("top: {}; zIndex: {};", cell_top(local_placement, &p_center), -((local_placement) as i32))}>
            <div class="run" id={team.login.clone()}>
                <div class={center_class(team.placement, &p_center).iter().chain(&["run_prefix"]).join(" ")}>
                    {is_compressed.then_some(view! {
                        <Placement placement={team.placement_global} />
                    })}
                    <Placement placement={local_placement} />
                    <TeamName escola={team.escola.clone()} name={team.name.clone()} />
                    <div class="cell problema quadrado">
                        <div class="cima">{score.solved}</div>
                        <div class="baixo">{score.penalty}</div>
                    </div>
                </div>
                {all_problems.char_indices().map(|(_prob_i, prob)| {
                    view! { <Problem prob team /> }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn ContestPanelHeader<'a>(sede: Option<&'a Sede>, all_problems: &'static str) -> impl IntoView {
    view! {
        <div id="runheader" class="run">
            <div class={estilo_sede(sede).iter().chain(&["cell", "titulo"]).join(" ")}>
                {nome_sede(sede).to_string()}
            </div>
            {all_problems.chars().map(|p| view! {
                <div class="cell problema quadrado">{p}</div>
            }).collect_view()}
        </div>

    }
}

#[component]
fn ContestPanel<'a>(
    contest: ContestFile,
    center: Option<String>,
    sede: Option<&'a Sede>,
) -> impl IntoView {
    let p_center = center.as_ref().map(|s| contest.teams[s].placement);
    let n: usize = contest.number_problems;
    let all_problems = &data::PROBLEM_LETTERS[..n];
    let is_compressed = contest
        .teams
        .values()
        .any(|team| !team.belongs_to_contest(sede));

    let teams = contest
        .teams
        .values()
        .filter(|team| team.belongs_to_contest(sede))
        .sorted_by_cached_key(|team| team.score());

    view! {
        <div class="runstable">
            <div class="run_box" style:top={cell_top(0, &p_center)}>
                <ContestPanelHeader sede=sede all_problems />
                {teams.enumerate().map(|(i, team)| {
                    view! {
                        <ContestPanelLine is_compressed i p_center team all_problems />
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn EmptyContestPanel<'a>(sede: Option<&'a Sede>) -> impl IntoView {
    view! {
        <div class="runstable">
            <div class="run_box" style:top={cell_top(0, &None)}>
                <ContestPanelHeader sede=sede all_problems="" />
            </div>
        </div>
    }
}

#[component]
pub fn Contest(
    contest: ReadSignal<Option<ContestFile>>,
    panel_items: ReadSignal<Vec<RunsPanelItem>>,
    timer: ReadSignal<(TimerData, TimerData)>,
    #[prop(optional)] sede: Option<Sede>,
) -> impl IntoView {
    move || {
        let contest = contest.get();
        let panel_items = panel_items.get();

        let contest_panel = match contest {
            Some(contest) => {
                view! { <ContestPanel contest center=None sede=sede.as_ref() /> }.into_view()
            }
            None => view! { <EmptyContestPanel sede=sede.as_ref() /> <p> loading contest </p> }
                .into_view(),
        };
        view! {
            <body style="height: 1px">
                <div style="display: flex; width: 320px;">
                    <div style="display: flex; flex-direction: column; width: 320px;">
                        <Timer timer />
                        <div class="submission-title"> Últimas Submissões </div>
                        <RunsPanel items=panel_items />
                    </div>
                    <div class="automatic" style="margin-left: 8px;">
                        {contest_panel}
                    </div>
                </div>
            </body>
        }
    }
}
