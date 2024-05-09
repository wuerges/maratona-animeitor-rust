use std::rc::Rc;

use data::{
    configdata::{Color, Sede},
    ContestFile, RunsPanelItem, Team, TimerData,
};
use itertools::Itertools;
use leptos::{logging::log, *};

use crate::views::timer::Timer;

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
fn Placement(placement: MaybeSignal<usize>, sede: Rc<Sede>) -> impl IntoView {
    move || {
        let color = get_color(placement.get(), &sede);
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
fn RunResult(
    problem: String,
    answer: Signal<data::Answer>,
    first_solved: Signal<bool>,
) -> impl IntoView {
    let balao = format!("balao_{}", problem);
    view! {
        <div class="cell quadrado">{problem}</div>
        {move || match answer.get() {
            data::Answer::Yes(time) => {
                let img = if first_solved.get() {"star-img"} else {"accept-img"};
                view! {
                <div class="accept cell quadrado">
                    <div class=format!("{img} {balao}") />
                    <div class="accept-text cell-content">{time}</div>
                </div>
            }},
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
fn RunsPanel(items: Signal<Vec<RunsPanelItem>>, sede: Rc<Sede>) -> impl IntoView {
    let sede_move = sede.clone();

    view! {
        <div class="runstable">
        <For
        each=move || take_30(items.get(), sede_move.clone())
        key=|(_, r)| r.id
        children={move |(i, o)| {
                let sede = sede.clone();
                let top = format!("calc(var(--row-height) * {} + var(--root-top))", i);
                let result = Signal::derive(move || items.get()[i].result.clone());
                let first_solved = Signal::derive(move || items.get()[i].first_solved);

                view! {
                    <div class="run" style:top={top} >
                        <Placement placement={o.placement.into()} sede=sede.clone() />
                        <TeamName escola={o.escola.clone()} name={o.team_name.clone()} />
                        <RunResult problem=o.problem.clone() answer=result first_solved />
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
    log!("rendered problem {:?}", problem);
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

fn problem_key(p: &data::ProblemView) -> u64 {
    p.id
}

#[component]
fn ContestPanelLine(
    is_compressed: Signal<bool>,
    p_center: Signal<Option<usize>>,
    team: Signal<Team>,
    all_problems: Signal<&'static str>,
    sede: Rc<Sede>,
) -> impl IntoView {
    log!("line refresh");

    let team_problems = create_memo(move |_| {
        let all_problems = all_problems.get();
        team.with(move |team| {
            all_problems
                .chars()
                .map(move |prob| (prob, team.problems.get(&prob.to_string()).map(|p| p.view())))
                .collect_vec()
        })
    });

    let id = move || team.with(|t| t.login.clone());
    let style = move || {
        team.with(|t| {
            format!(
                "top: {}; z-index: {};",
                cell_top(t.placement, &p_center.get()),
                -((t.placement) as i32)
            )
        })
    };

    let center = move || {
        team.with(|t| {
            p_center
                .get()
                .and_then(|c| (c == t.placement).then_some("center"))
                .iter()
                .chain([&"run_prefix"])
                .join(" ")
        })
    };

    view! {
        <div class="run_box" id={id} style={style}>
            <div class="run">
            {move || {
                team.with(|t| {
                    log!("rerendering teams: {t:?}");
                    let is_compressed = is_compressed.get();
                    let score = t.score();
                    view!{
                        <div class={center}>
                            {is_compressed.then_some(view! {
                                <Placement placement={t.placement.into()} sede=sede.clone() />
                            })}
                            <Placement placement=t.placement.into() sede=sede.clone() />
                            <TeamName escola=t.escola.clone() name=t.name.clone() />
                            <div class="cell problema quadrado">
                                <div class="cima">{score.solved}</div>
                                <div class="baixo">{score.penalty}</div>
                            </div>
                        </div>
                    }
                })
            }}
            <For
                each=move || team_problems.get()
                key=|(k, prob)| (*k, prob.as_ref().map(problem_key))
                children = {move |(prob, problem)| {
                    view!{ <Problem prob problem /> }
                }}
            />
            </div>
        </div>
    }
}

#[component]
fn ContestPanelHeader<'a>(sede: &'a Sede, all_problems: Signal<&'static str>) -> impl IntoView {
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

fn team_key(team: &Team) -> u64 {
    team.id
}

#[component]
pub fn ContestPanel(
    contest: Signal<ContestFile>,
    center: Signal<Option<String>>,
    sede: Rc<Sede>,
) -> impl IntoView {
    log!("contest panel refresh");
    let n: Signal<usize> = Signal::derive(move || contest.with(|c| c.number_problems));
    let all_problems = Signal::derive(move || &data::PROBLEM_LETTERS[..n.get()]);
    let cloned_sede = sede.clone();
    let is_compressed = create_memo(move |_| {
        contest.with(|c| c.teams.values().any(|team| !cloned_sede.team_belongs(team)))
    });

    let cloned_sede = sede.clone();
    let teams = create_memo(move |_| {
        contest.with(|c| {
            c.teams
                .values()
                .filter(|team| cloned_sede.team_belongs(team))
                .cloned()
                .collect_vec()
        })
    });

    let placements = create_memo(move |_| {
        log!("started calculating placements");
        let result = contest.with(|_c| {
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
        log!("finished calculating placements");
        result
    });

    let p_center = (move || {
        with!(|center, teams, placements| {
            center
                .as_ref()
                .and_then(|center| find_center(&center, teams).map(|c| placements[c] + 1))
        })
    })
    .into_signal();

    view! {
        <div class="runstable">
            <div class="run_box" style:top={move || {
                log!("center {:?} {:?}", center.get(), p_center.get());
                cell_top(0, &p_center.get())}}>
                <ContestPanelHeader sede=&sede all_problems />
            </div>

            <For
                each=move || (0..teams.with(|t| t.len()))
                key=move |i| teams.with(|t| team_key(&t[*i as usize]))
                children={move |i| {
                    log!("rerender children");
                    let team = create_memo(move |_| contest.with(|_c| teams.with(|ts| ts[i].clone())));

                    view!{
                        <ContestPanelLine is_compressed=is_compressed.into() p_center=p_center.into() team=team.into() all_problems sede=sede.clone() />
                    }
                }}
            />
        </div>
    }
}

#[component]
fn EmptyContestPanel<'a>(sede: &'a Sede) -> impl IntoView {
    view! {
        <div class="runstable">
            <div class="run_box" style:top={cell_top(0, &None)}>
                <ContestPanelHeader sede all_problems=create_signal("").0.into() />
            </div>
        </div>
    }
}

#[component]
pub fn Contest(
    contest: Signal<ContestFile>,
    panel_items: ReadSignal<Vec<RunsPanelItem>>,
    timer: ReadSignal<(TimerData, TimerData)>,
    sede: Rc<Sede>,
) -> impl IntoView {
    let (center, _) = create_signal(None);

    view! {
        <body style="height: 1px">
            <div style="display: flex; width: 320px;">
                <div style="display: flex; flex-direction: column; width: 320px;">
                    <Timer timer />
                    <div class="submission-title"> Últimas Submissões </div>
                    <RunsPanel items=panel_items.into() sede=sede.clone() />
                </div>
                <div class="automatic" style="margin-left: 8px;">
                    <ContestPanel contest center=center.into() sede=sede.clone() />
                </div>
            </div>
        </body>
    }
}
