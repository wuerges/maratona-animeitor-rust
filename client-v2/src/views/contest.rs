use data::{configdata::Sede, BelongsToContest, ContestFile, RunsPanelItem};
use itertools::Itertools;
use leptos::*;

use crate::{model::provide_contest, views::timer::Timer};

#[component]
pub fn Contest() -> impl IntoView {
    let (contest, panel) = provide_contest();

    move || {
        panel.with(|panel| {
            contest.with(|contest| match contest {
                Some(contest) => everything(contest, panel).into_view(),
                None => view! {<p> Contest is none =/ </p>}.into_view(),
            })
        })
    }
}

pub fn get_color(n: usize, sede: Option<&Sede>) -> &str {
    match sede {
        Some(sede) => sede.premio(n),
        None => {
            if n == 0 {
                "vermelho"
            } else if n <= 4 {
                "ouro"
            } else if n <= 8 {
                "prata"
            } else if n <= 12 {
                "bronze"
            } else {
                "semcor"
            }
        }
    }
}

fn get_image(t: &data::Answer) -> &'static str {
    match t {
        data::Answer::Yes(_) => "assets/balloon-border.svg",
        data::Answer::No => "assets/no.svg",
        data::Answer::Wait => "assets/question.svg",
        _ => "assets/question.svg",
    }
}

fn get_answer(t: &data::Answer) -> &str {
    match t {
        data::Answer::Yes(_) => "answeryes",
        data::Answer::No => "answerno",
        data::Answer::Unk => "answerno", // Unknown is X -> error without penalty
        data::Answer::Wait => "answerwait",
    }
}

fn runs_panel(panel: &Vec<RunsPanelItem>) -> impl IntoView {
    view! {
        <div class="runstable">
        {
            panel.iter().take(30).enumerate().map(|(i, r)| {
                let balao = format!("balao_{}", r.problem);
                let top = format!("calc(var(--row-height) * {} + var(--root-top))", i);
                let cor = get_color(r.placement, None);
                let problem = r.problem.clone();

                view! {
                    <div class="run" style={format!("top: {top}")}>
                        <div class={["cell", "colocacao", "quadrado", cor].join(" ")}>
                            {r.placement}
                        </div>
                        <div class={["cell", "time"].join(" ")}>
                            <div class="nomeEscola">{&r.escola}</div>
                            <div class="nomeTIme">{&r.team_name}</div>
                        </div>
                        <div class={["cell", "resposta", "quadrado", get_answer(&r.result)].join(" ")}>
                            {matches!(r.result, data::Answer::Yes(_)).then_some(view! {
                                <div>
                                    <img class={["answer-img", balao.as_str()].join(" ")} src="assets/balloon.svg" />
                                </div>
                            })}
                            <img class="answer-img" src={get_image(&r.result)} />
                            <div class="answer-text">{problem}</div>
                        </div>
                    </div>
                }
            }).collect_view()
        }
        </div>
    }
}

use std::collections::BTreeMap;
fn compress_placement<'a, I>(plac: I) -> BTreeMap<usize, usize>
where
    I: Iterator<Item = &'a usize>,
{
    let mut v: Vec<_> = plac.collect();
    v.sort();

    let mut ret = BTreeMap::new();

    for (i, e) in v.iter().enumerate() {
        *ret.entry(**e).or_default() = i;
    }
    ret
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

fn contest_panel(
    contest: &ContestFile,
    center: Option<String>,
    sede: Option<&Sede>,
    revelation: bool,
) -> impl IntoView {
    let p_center = center.as_ref().map(|s| contest.teams[s].placement);
    let n = contest.number_problems;
    let all_problems = &data::PROBLEM_LETTERS[..n];
    let compressed_ = compress_placement(
        contest
            .teams
            .values()
            .filter(|t| t.belongs_to_contest(sede))
            .map(|t| &t.placement),
    );
    let is_compressed = !revelation && (compressed_.len() < contest.teams.len());

    // pub fn view_scoreboard<T>(
    //     contest: &ContestFile,
    //     center: &Option<String>,
    //     sede: Option<&Sede>,
    //     revelation: bool,
    // ) -> Node<T> {

    //     div![
    //         C!["runstable"],
    //         div![
    //             C!["run_box"],
    //             style!{St::Top => cell_top(0, &p_center)},
    //             div![
    //                 id!["runheader"],
    //                 C!["run"],
    //                 div![
    //                     C![
    //                         "cell",
    //                         "titulo",
    //                         estilo_sede(sede)
    //                     ],
    //                     nome_sede(sede)
    //                 ],
    //                 all_problems.chars().map( |p| div![C!["cell", "problema", "quadrado"], p.to_string()])
    //             ]
    //         ],
    //         contest.teams.values()
    //                 .map (|team| {
    //             let score = team.score();
    //             let p2 = team.placement;
    //             let display = team.belongs_to_contest(sede);
    //             div![
    //                 C!["run_box"],
    //                 style!{St::Top => cell_top(p2, &p_center), St::ZIndex => -(p2 as i32)},
    //                 div![
    //                     IF!(!display => style!{St::Display => "none"}),
    //                     id![&team.login],
    //                     C!["run"],
    //                     div![C!["run_prefix"],
    //                         center_class(team.placement, &p_center),
    //                         IF!(is_compressed => div![C!["cell", "colocacao", "quadrado", get_color(team.placement_global, None)], team.placement_global]),
    //                         div![C!["cell", "colocacao", "quadrado", get_color(p2, sede)], p2],
    //                         div![
    //                             C!["cell", "time"],
    //                             div![C!["nomeEscola"], &team.escola],
    //                             div![C!["nomeTime"], &team.name,
    //                             ],
    //                             attrs!{At::OnClick =>
    //                                 std::format!("document.getElementById('foto_{}').style.display = 'block';", &team.login),
    //                             },
    //                         ],
    //                         div![
    //                             C!["cell", "problema", "quadrado"],
    //                             div![C!["cima"], score.solved],
    //                             div![C!["baixo"], score.penalty],
    //                         ],
    //                     ],
    //                     all_problems.char_indices().map( |(_prob_i, prob)| {
    //                         match team.problems.get(&prob.to_string()) {

    //                             None => div![C!["not-tried", "cell", "quadrado"], "-"],
    //                             Some(prob_v) => {
    //                                 if prob_v.solved {
    //                                     let balao = std::format!("balao_{}", prob);
    //                                     div![C!["accept", "cell", "quadrado"],
    //                                         div![
    //                                             C!["accept-img", balao],
    //                                         ],
    //                                         div![
    //                                             C!["accept-text"],
    //                                             "+",
    //                                             number_submissions(prob_v.submissions),
    //                                             br![],
    //                                             prob_v.time_solved,
    //                                         ],
    //                                     ]
    //                                 }
    //                                 else {
    //                                     let cell_type = if prob_v.wait() {"inqueue"} else {"unsolved"};
    //                                     let cell_symbol = if prob_v.wait() {"?"} else {"X"};
    //                                     div![
    //                                         C![cell_type, "cell", "quadrado"],
    //                                         div![C!["cima"], cell_symbol],
    //                                         div![C!["baixo"], "(", prob_v.submissions, ")"],
    //                                     ]
    //                                 }
    //                             },
    //                         }
    //                     })
    //                 ]
    //             ]
    //         }),
    //     ]
    // }

    view! {
        <div class="runstable">
            <div class="run_box" style={format!("top: {}", cell_top(0, &p_center))}>
                <div id="runheader" class="run">
                    <div class={estilo_sede(sede).iter().chain(&["cell", "titulo"]).join(" ")}>
                        {nome_sede(sede).to_string()}
                    </div>
                    {all_problems.chars().map(|p| view! {
                        <div class="cell problema quadrado">{p}</div>
                    }).collect_view()}
                </div>
            </div>
        </div>
        <p> Contest!: "`"{format!("{:#?}", contest)}"'" </p>
    }
}

pub fn everything(contest: &ContestFile, panel: &Vec<RunsPanelItem>) -> impl IntoView {
    view! {
        <body style="height: 1px">
            <div style="display: flex; width: 320px;">
                <div style="display: flex; flex-direction: column; width: 320px;">
                    // <Sedes />
                    <Timer />
                    <div class="submission-title"> Últimas Submissões </div>
                    {runs_panel(panel)}
                </div>
                <div class="automatic" style="margin-left: 8px;">
                    {contest_panel(contest, None, None, false)}
                </div>
            </div>
        </body>
    }
}
