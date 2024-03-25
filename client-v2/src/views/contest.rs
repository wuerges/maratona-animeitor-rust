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
fn Placement(
    placement: MaybeSignal<usize>,
    #[prop(optional_no_strip)] sede: Option<Sede>,
) -> impl IntoView {
    move || {
        let color = get_color(placement.get(), sede.as_ref());
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
                        <Placement placement={r.placement.into()} sede=sede.clone() />
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
fn Problem(prob: char, team: Signal<Team>) -> impl IntoView {
    let problem =
        Signal::derive(move || team.with(move |t| t.problems.get(&prob.to_string()).cloned()));
    view! {

            <div class={move || match problem.get() {
                Some(p) => if p.solved {
                    "accept cell quadrado".to_string()
                } else {
                    let cell_type = if p.wait() { "inqueue"} else { "unsolved" };
                    format!("cell quadrado {cell_type}")
                },
                None => "not-tried cell quadrado".to_string(),
            }}>
            {move || match problem.get() {
                Some(p) => {
                    (if p.solved {
                        let balao = format!("balao_{}", prob);
                        view! {
                            <div class=format!("accept-img {balao}")></div>
                            <div class="accept-text cell-content">
                                +{number_submissions(p.submissions)}<br />{p.time_solved}
                            </div>
                        }
                    } else {
                        let cell_symbol = if p.wait() { "?" } else { "X" };

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
fn ContestPanelLine(
    is_compressed: Signal<bool>,
    local_placement: Signal<usize>,
    p_center: Signal<Option<usize>>,
    team: Signal<Team>,
    all_problems: Signal<&'static str>,
) -> impl IntoView {
    log!("line refresh");

    view! {
        <div class="run_box" id={move || team.with(|t| t.login.clone())} style={move || format!(
            "top: {}; z-index: {};",
            cell_top(local_placement.get(), &p_center.get()),
            -((local_placement.get()) as i32)
        )}>
            <div class="run">
            {move || {
                let team_value = team.get();
                let is_compressed = is_compressed.get();
                let score = team_value.score();
                view!{
                    <div class={center_class(local_placement.get(), &p_center.get()).iter().chain(&["run_prefix"]).join(" ")}>
                        {is_compressed.then_some(view! {
                            <Placement placement={(move || team_value.placement_global).into_signal().into()} />
                        })}
                        <Placement placement=local_placement.into() />
                        <TeamName escola=team_value.escola.clone() name=team_value.login.clone() />
                        <div class="cell problema quadrado">
                            <div class="cima">{score.solved}</div>
                            <div class="baixo">{score.penalty}</div>
                        </div>
                    </div>
                    {move || all_problems.get().char_indices().map(|(_prob_i, prob)| {
                        view! { <Problem prob team /> }
                    }).collect_view()}
                }
            }}
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
        .map(|p| p.0)
}

#[component]
pub fn ContestPanel(
    contest: Signal<ContestFile>,
    center: Signal<Option<String>>,
    sede: Option<Sede>,
) -> impl IntoView {
    log!("contest panel refresh");
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
            .collect_vec()
    });

    let placements = Signal::derive(move || {
        teams.with(|ts| {
            let ps = ts
                .iter()
                .enumerate()
                .sorted_by_cached_key(|(_, t)| t.placement_global)
                .map(|(i, _)| i)
                .collect_vec();

            ps.into_iter()
                .enumerate()
                .map(|(u, v)| (v, u))
                .sorted()
                .map(|(_, v)| v)
                .collect_vec()
        })
    });

    let p_center = Signal::derive(move || {
        with!(|center, teams, placements| {
            center
                .as_ref()
                .and_then(|center| find_center(&center, teams).map(|c| placements[c] + 1))
        })
    });

    view! {
        <div class="runstable">
            <div class="run_box" style:top={move || {
                log!("center {:?} {:?}", center.get(), p_center.get());
                cell_top(0, &p_center.get())}}>
                <ContestPanelHeader sede=sede.as_ref() all_problems />
            </div>

            <For
                each=move || teams.get().into_iter().enumerate()
                key=|(_, team)| team.login.clone()
                children={move |(i, _)| {
                    let local_placement = Signal::derive(move || placements.with(|ps| ps[i] + 1));
                    let team = Signal::derive(move || teams.with(|ts| ts[i].clone()));

                    view!{
                        <ContestPanelLine is_compressed local_placement p_center team all_problems />
                    }
                }}
            />
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
