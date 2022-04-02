
use data::{ContestFile, TimerData};
use data::configdata::Sede;
use seed::{prelude::*, *};
use crate::helpers::*;



pub fn get_color(n : usize, sede: Option<&Sede>) -> &str {
    match sede {
        Some(sede) => {
            sede.premio(n)
        },
        None => {
            if n == 0 {
                "vermelho"
            }
            else if n <= 4 {
                "ouro"
            }
            else if n <= 8 {
                "prata"
            }
            else if n <= 12 {
                "bronze"
            }
            else {
                "semcor"
            }
        }
    }
}

pub fn cell_top(i : usize, center: &Option<usize>) -> String {
    let i = i as i64;
    match center {
        None => format!("calc(54px * {} + var(--root-top))", i),
        Some(p) => {
            let p = *p as i64;
            if p < 9 {
                format!("calc(54px * {} + var(--root-top))", i)
            }
            else {
                format!("calc(54px * {} + var(--root-top-center))", (i - p))
            }
        }
    }
}

use std::collections::BTreeMap;
fn compress_placement<'a, I>(plac: I) -> BTreeMap<usize, usize>
where I : Iterator<Item= &'a usize>
    {
    let mut v :Vec<_>= plac.collect();
    v.sort();

    let mut ret = BTreeMap::new();

    for (i, e) in v.iter().enumerate() {
        *ret.entry(**e).or_default() = i;
    }
    ret
}

fn center_class(p: usize, center : &Option<usize>) -> std::option::Option<seed::Attrs> {
    match center {
        None => None,
        Some(c) => if *c == p { Some(C!["center"]) } else { None }
    }
}

fn number_submissions(s : usize) -> Option<usize> {
    if s == 1 { None } else { Some(s - 1) }
}

fn nome_sede (sede: Option<&Sede>) -> String {
    match sede {
        None => "Placar".to_string(),
        Some(sede) => sede.name.clone()
    }
}

fn estilo_sede (sede: Option<&Sede>) -> Option<&String> {
    sede.map( |s| s.style.as_ref() ).flatten()
}

pub fn view_scoreboard<T>(contest: &ContestFile, center: &Option<String>, sede: Option<&Sede>) -> Node<T> {

    let p_center = center.as_ref().map(|s| contest.teams[s].placement);
    let url_filter =  sede.as_ref().map( |s| &s.codes );

    // let problem_letters = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"];
    let n = contest.number_problems;
    let all_problems = &data::PROBLEM_LETTERS[..n];
    let compressed_ = compress_placement(contest.teams.values()
                        .filter( |t| data::check_filter(url_filter, t))
                        .map(|t| &t.placement));

    let is_compressed = compressed_.len() < contest.teams.len();
    div![
        C!["runstable"],
        div![
            id!["runheader"],
            C!["run"],
            style!{
                St::Top => cell_top(0, &p_center),
                // St::Top => px(margin_top),
                // St::Position => "absolute",
                // St::Transition => "top 1s ease 0s",
            },
            div![
                C![
                    "cell",
                    "titulo",
                    estilo_sede(sede),
                    if is_compressed {"duplaColocacao"} else {"unicaColocacao"}
                    ],
                    nome_sede(sede)],
            all_problems.chars().map( |p| div![C!["cell", "problema"], p.to_string()])
        ],
        contest.teams.values() //.filter( |t| data::check_filter(url_filter, t))
                .map (|team| {
            let score = team.score();
            // let p2 = 1+*compressed_.get(&team.placement).unwrap_or(&0);
            let p2 = team.placement;
            let display = data::check_filter(url_filter, team);
            div![
                IF!(!display => style!{St::Display => "none"}),
                id![&team.login],
                C!["run"],
                center_class(team.placement, &p_center),
                style!{
                    // St::Top => px(margin_top + (team.placement as i64) * 90),
                    St::Top => cell_top(p2, &p_center),
                    // St::Top => cell_top(team.placement, &p_center),
                    // St::Position => "absolute",
                    // St::Transition => "top 1s ease 0s",
                },
                IF!(is_compressed => div![C!["cell", "colocacao", get_color(team.placement_global, None)], team.placement_global]),
                div![C!["cell", "colocacao", get_color(p2, sede)], p2],
                div![
                    C!["cell", "time"],
                    div![C!["nomeEscola"], &team.escola],
                    div![C!["nomeTime"], &team.name,
                    ],
                    attrs!{At::OnClick =>
                        std::format!("document.getElementById('foto_{}').style.display = 'block';", &team.login),
                        // std::format!("alert('foto_{}')", &team.login),
                        // document.getElementById('a').style.backgroundColor = ''"
                    },
                ],
                div![
                    C!["cell", "problema"],
                    div![C!["cima"], score.solved],
                    div![C!["baixo"], score.penalty],
                ],
                all_problems.char_indices().map( |(prob_i, prob)| {
                    let hue = get_answer_hue_deg(n, prob_i as u32);
                    match team.problems.get(&prob.to_string()) {

                        None => div![C!["not-tried", "cell"], "-"],
                        Some(prob_v) => {
                            if prob_v.solved {
                                let balao = std::format!("balao_{}", prob);
                                div![
                                    C!["accept", "cell"],
                                    img![
                                        C!["accept-img", balao],
                                        attrs!{At::Src => "/static/assets/balloon.svg"},
                                    ],
                                    img![
                                        C!["accept-img"],
                                        attrs!{At::Src => "/static/assets/balloon-border.svg"},
                                    ],
                                    div![
                                        C!["accept-text"],
                                        div!["+", number_submissions(prob_v.submissions)],
                                        div![prob_v.time_solved],
                                    ],
                                ]
                            }
                            else {
                                let cell_type = if prob_v.wait() {"inqueue"} else {"unsolved"};
                                let cell_symbol = if prob_v.wait() {"?"} else {"X"};
                                div![
                                    C![cell_type, "cell"],
                                    div![C!["cima"], cell_symbol],
                                    div![C!["baixo"], "(", prob_v.submissions, ")"],
                                ]
                            }
                        },
                    }
                })
            ]
        }),
    ]
}

fn f(n : i64) -> String {
    format!("{:0>2}", n)
}

fn seg(n : i64) -> i64 {
    n % 60

}
fn min(n : i64) -> i64 {
    (n / 60) % 60
}
fn hor(n : i64) -> i64 {
    n / 60 / 60
}
fn changed(a : i64, b: i64) -> &'static str {
    if a == b {
        "same"
    }
    else {
        "changed"
    }
}

pub fn view_clock<T>(time_data: TimerData, ptime_data : TimerData) -> Node<T> {

    let time = time_data.current_time;
    let ptime = ptime_data.current_time;

    let frozen = if time_data.is_frozen() { Some(C!["frozen"]) } else { None };

    div![C!["timer"], frozen,
        span![C!["hora", changed(hor(time), hor(ptime))], hor(time)],
        span![C!["sep"], ":"],
        span![C!["minuto", changed(min(time), min(ptime))], f(min(time))],
        span![C!["sep"], ":"],
        span![C!["segundo", changed(seg(time), seg(ptime))], f(seg(time))],
    ]
}
