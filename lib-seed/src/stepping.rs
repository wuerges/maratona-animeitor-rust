// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
//#![allow(clippy::wildcard_imports)]

use maratona_animeitor_rust::data;
use seed::{prelude::*, *};

extern crate rand;


// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.


// Request::new(get_request_url())
//         .method(Method::Post)
//         .json(&shared::SendMessageRequestBody { text: new_message })?
//         .fetch()
//         .await?
//         .check_status()?
//         .json()
//         .await

async fn fetch_allruns() -> fetch::Result<data::RunsFile> {
    Request::new("/allruns")
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

async fn fetch_contest() -> fetch::Result<data::ContestFile> {
    Request::new("/contest")
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    // Model::default()

    orders.skip().perform_cmd({
        async {
            let m = fetch_contest().await;
            Msg::FetchedContest(m)
        }
    });
    Model { 
        contest: data::ContestFile::dummy(),
        runs: data::RunsFile::empty(),
        current_run: 0,
    }
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
// type Model = Vec<i64>;
struct Model {
    contest : data::ContestFile,
    runs: data::RunsFile,
    current_run: usize,
}

enum Msg {
    // Append,
    Shuffle,
    // Sort,
    // SortEnd,
    Prox(usize),
    FetchedRuns(fetch::Result<data::RunsFile>),
    FetchedContest(fetch::Result<data::ContestFile>),
}

fn shuffle<T>(v: &mut  Vec<T> ) {
    use rand::thread_rng;
    use rand::seq::SliceRandom;

    let mut rng = thread_rng();
    v.shuffle(&mut rng);
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        // Msg::Append => model.items.push(model.items.len() as i64),
        Msg::Shuffle => {
            shuffle(&mut model.contest.score_board)
        },
        // Msg::Sort => {
        //     orders.perform_cmd(cmds::timeout(1000, || Msg::SortEnd));
        //     model.items.sort();
        // },
        // Msg::SortEnd => {
        //     log!("sort ended!")
        // },
        Msg::Prox(n) => {
            for _ in 0..n {
                if model.current_run < model.runs.runs.len() {
                    let run = &model.runs.runs[model.current_run];
                    model.contest.apply_run(run).unwrap();
                    // log!("appllied run:", run);
                    model.current_run += 1;
                }
            }
            model.contest.recalculate_placement().unwrap();
        },
        Msg::FetchedRuns(Ok(runs)) => {
            // log!("fetched runs data!");
            model.runs = runs;
            model.runs.runs.reverse();
        },
        Msg::FetchedContest(Ok(contest)) => {
            // log!("fetched contest data!");

            model.contest = contest;
            model.contest.reload_score().unwrap();
            orders.perform_cmd({
                async { Msg::FetchedRuns(fetch_allruns().await) }
            });
        },
        Msg::FetchedContest(Err(e)) => {
            log!("fetched contest error!", e)
        },
        Msg::FetchedRuns(Err(e)) => {
            log!("fetched runs error!", e)
        },

    }
}

fn make_style(e : & i64) -> seed::Style {
    style!{
        St::Position => "absolute",
        St::Top => px(100 + e*50),
        St::Transition => "1s ease top",
    }
}

// ------ ------
//     View
// ------ ------

// (Remove the line below once your `Model` become more complex.)
// #[allow(clippy::trivially_copy_pass_by_ref)]
// `view` describes what to display.


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


fn view(model: &Model) -> Node<Msg> {
    let margin_top = 100;
    let problem_letters = 
        vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"];
    // let all_problems = vec!["A", "B", "C"];
    let n = model.contest.number_problems;
    // log!(model.contest.number_problems);
    let s = model.contest.score_board.len();
    // let s = (model.contest.score_board.len()).min(100);
    let all_problems = &problem_letters[..n];
    div![
        button!["+1", ev(Ev::Click, |_| Msg::Prox(1)),],
        button!["+10", ev(Ev::Click, |_| Msg::Prox(10)),],
        button!["+100", ev(Ev::Click, |_| Msg::Prox(100)),],
        button!["+1000", ev(Ev::Click, |_| Msg::Prox(1000)),],
        div!["Runs: ", model.current_run, "/", model.runs.runs.len()],
        // button!["shuffle", ev(Ev::Click, |_| Msg::Shuffle),],        
        attrs!{"border" => 1},
        div![
            C!["run"],
            style!{ St::Position => "absolute", St::Top => px(margin_top) },
            div![C!["cell", "titulo"], "Placar"],
            all_problems.iter().map( |p| div![C!["cell", "problema"], p])
        ],
        model.contest.score_board.iter().enumerate().map (|(idx, dev)| {
            let team = &model.contest.teams[&dev.clone()];
            let (solved, penalty) = team.score();
            div![
                id![dev],
                C!["run"],
                style!{
                    St::Top => px(margin_top + (team.placement) * 90),
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
        // button!["sort", ev(Ev::Click, |_| Msg::Sort),],
        // model.items.iter().enumerate().map( |(i,e)| 
        //     div![
        //         id![i],
        //         make_style(e, 0),
        //         i,
        //         "->",
        //         e
        //     ]
        // ),
        // div![
        //     id![1],
        //     "Up",
        //     make_style(model),
        // ],
        // div![                // make_style(&(idx as i64)),

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
// #[wasm_bindgen(start)]
pub fn stepping_start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
