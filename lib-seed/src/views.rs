
use maratona_animeitor_rust::data::ContestFile;
use seed::{prelude::*, *};

fn get_color(n : usize) -> String {
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

pub fn view_scoreboard<T>(contest: &ContestFile, margin_top : i64) -> Node<T> {
    let problem_letters = 
        vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"];
    let n = contest.number_problems;
    let all_problems = &problem_letters[..n];
    div![
        div![
                C!["run"],
                style!{ 
                    St::Top => px(margin_top),
                    St::Position => "absolute", 
                    St::Transition => "top 1s ease 0s",
                },
                div![C!["cell", "titulo"], "Placar"],
                all_problems.iter().map( |p| div![C!["cell", "problema"], p])
        ],
        contest.teams.values().map (|team| {
            let (solved, penalty) = team.score();
            div![
                id![&team.login],
                C!["run"],
                style!{
                    St::Top => px(margin_top + (team.placement as i64) * 90),
                    St::Position => "absolute",
                    St::Transition => "top 1s ease 0s",
                },
                div![C!["cell", "colocacao", get_color(team.placement)], team.placement],
                div![
                    C!["cell", "time"],
                    div![C!["nomeEscola"], &team.escola],
                    div![C!["nomeTime"], &team.name],
                ],
                div![
                    C!["cell", "problema"],
                    div![C!["cima"], solved],
                    div![C!["baixo"], penalty],
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