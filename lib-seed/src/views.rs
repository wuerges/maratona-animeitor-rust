
use maratona_animeitor_rust::data::{ContestFile, Team};
use seed::{prelude::*, *};

pub fn get_color(n : usize) -> String {
    (if n == 0 {
        "vermelho"
    }
    else if n <= 3 {
        "ouro"
    }
    else if n <= 6 {
        "prata"
    }
    else if n <= 10 {
        "bronze"
    }
    else {
        "semcor"
    }).to_string()
}

pub fn cell_top(i : usize, center: &Option<usize>) -> String {
    let i = i as i64;
    match center {
        None => format!("calc(var(--row-height) * {} + var(--root-top))", i),
        Some(p) => {
            let p = *p as i64;
            format!("calc(var(--row-height) * {} + var(--root-top-center))", (i - p))
        }
    }
}

fn check_filter(url_filter: &Option<String>, t : &Team) -> bool {
    match url_filter {
        None => true,
        Some(f) => t.login.find(f).is_some(),
    }
}

use std::collections::BTreeMap;
fn compress_placement<'a, I>(plac: I) -> BTreeMap<usize, usize> 
where I : Iterator<Item= &'a usize>
    {
    let mut v :Vec<_>= plac.collect();
    v.sort_unstable();

    let mut ret = BTreeMap::new();

    for (i, e) in v.iter().enumerate() {
        *ret.entry(**e).or_default() = i;
    }
    ret
}

pub fn view_scoreboard<T>(contest: &ContestFile, center: &Option<String>, url_filter: &Option<String>) -> Node<T> {

    let p_center = center.as_ref().map(|s| contest.teams[s].placement);

    let problem_letters = 
        vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"];
    let n = contest.number_problems;
    let all_problems = &problem_letters[..n];
    let compressed = compress_placement(contest.teams.values()
                        .filter( |t| check_filter(url_filter, t))
                        .map(|t| &t.placement));
    div![
        C!["runstable"],
        div![
                C!["run"],
                style!{ 
                    St::Top => cell_top(0, &p_center),
                    // St::Top => px(margin_top),
                    // St::Position => "absolute", 
                    // St::Transition => "top 1s ease 0s",
                },
                div![C!["cell", "titulo"], "Placar"],
                all_problems.iter().map( |p| div![C!["cell", "problema"], p])
        ],
        contest.teams.values().filter( |t| check_filter(url_filter, t))
                .map (|team| {
            let score = team.score();
            let p2 = compressed.get(&team.placement).unwrap_or(&0);
            div![
                id![&team.login],
                C!["run"],
                style!{
                    // St::Top => px(margin_top + (team.placement as i64) * 90),
                    St::Top => cell_top(*p2+1, &p_center),
                    // St::Top => cell_top(team.placement, &p_center),
                    // St::Position => "absolute",
                    // St::Transition => "top 1s ease 0s",
                },
                div![C!["cell", "colocacao", get_color(team.placement)], team.placement],
                div![
                    C!["cell", "time"],
                    div![C!["nomeEscola"], &team.escola],
                    div![C!["nomeTime"], &team.name],
                ],
                div![
                    C!["cell", "problema"],
                    div![C!["cima"], score.solved],
                    div![C!["baixo"], score.penalty],
                ],
                all_problems.iter().map( |prob| {
                    match team.problems.get(*prob) {
                        None => div![C!["cell", "problema"], "-"],
                        Some(prob_v) => {
                            if prob_v.solved {
                                div![
                                    C!["cell", "problema", "verde"],
                                    div![C!["cima"], "+", prob_v.submissions],
                                    div![C!["baixo"], prob_v.penalty],
                                ]
                            }
                            else {
                                let color = if prob_v.wait {"amarelo"} else {"vermelho"};
                                div![
                                    C!["cell", "problema", color],
                                    div![C!["cima"], "X"],
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

pub fn view_clock<T>(time: i64, ptime : i64) -> Node<T> {
    div![C!["timer"], 
        span![C!["hora", changed(hor(time), hor(ptime))], hor(time)], 
        span![C!["sep"], ":"],
        span![C!["minuto", changed(min(time), min(ptime))], f(min(time))], 
        span![C!["sep"], ":"],
        span![C!["segundo", changed(seg(time), seg(ptime))], f(seg(time))], 
    ]
}