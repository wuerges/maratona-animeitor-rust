use data::{configdata::Sede, ContestFile, RunsPanelItem};
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

// div![
//         C!["runstable"],
//         model.runs.iter().filter(
//             |r| model.team_belongs_str(&r.team_login)
//         ).take(30).enumerate().map({
//             |(i, r)| {
//                 let balao = std::format!("balao_{}", r.problem);
//                 div![
//                     C!["run"],
//                     style! {
//                         St::Top => format!("calc(var(--row-height) * {} + var(--root-top))", i),
//                     },
//                     div![
//                         C!["cell", "colocacao", "quadrado", views::get_color(r.placement, None)],
//                         r.placement
//                     ],
//                     div![
//                         C!["cell", "time"],
//                         div![C!["nomeEscola"], &r.escola],
//                         div![C!["nomeTime"], &r.team_name],
//                     ],
//                     div![
//                         C!["cell", "resposta", "quadrado", get_answer(&r.result)],
//                         IF!(matches!(r.result, data::Answer::Yes(_)) =>
//                         div![
//                             img![
//                                 C!["answer-img", balao],
//                                 attrs!{At::Src => "/assets/balloon.svg"},
//                             ],
//                         ]),
//                         img![
//                             C!["answer-img"],
//                             attrs!{At::Src => get_image(&r.result)},
//                         ],
//                         div![
//                             C!["answer-text"],
//                             &r.problem
//                         ]
//                     ],

//                     attrs!{At::OnClick =>
//                         std::format!("document.getElementById('foto_{}').style.display = 'block';", &r.team_login),
//                     },
//                 ]
//             }
//         })
//     ]

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
    // let panel_items = panel.iter().take(30).collect_vec();
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
                                    <img class={["answer-img", balao.as_str()].join(" ")} src="assets/ballon.svg" />
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
        <p> Panel!: "`"{format!("{:#?}", panel.iter().take(30).collect::<Vec<_>>())}"'" </p>
    }
}

fn contest_panel(contest: &ContestFile) -> impl IntoView {
    view! {
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
                    {contest_panel(contest)}
                </div>
            </div>
        </body>
    }
}
