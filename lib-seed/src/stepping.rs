// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
//#![allow(clippy::wildcard_imports)]

use maratona_animeitor_rust::data;
use seed::{prelude::*, *};
use crate::views;

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




fn view(model: &Model) -> Node<Msg> {
    let margin_top = 100;
    div![
        button!["+1", ev(Ev::Click, |_| Msg::Prox(1)),],
        button!["+10", ev(Ev::Click, |_| Msg::Prox(10)),],
        button!["+100", ev(Ev::Click, |_| Msg::Prox(100)),],
        button!["+1000", ev(Ev::Click, |_| Msg::Prox(1000)),],
        div!["Runs: ", model.current_run, "/", model.runs.runs.len()],
        // button!["shuffle", ev(Ev::Click, |_| Msg::Shuffle),],
        views::view_scoreboard(&model.contest, margin_top),
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
