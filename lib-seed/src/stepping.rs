use maratona_animeitor_rust::data;
use seed::{prelude::*, *};
use crate::views;
use crate::requests::*;

extern crate rand;


fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    // orders.skip().perform_cmd( fetch_all() );
    orders.send_msg(Msg::Reset);
    Model { 
        url_filter : url.hash().map( |s| s.clone()),
        contest: data::ContestFile::dummy(),
        runs: data::RunsFile::empty(),
        current_run: 0,
        center: None,
        lock_frozen : true,
    }
}

struct Model {
    url_filter : Option<String>,
    contest : data::ContestFile,
    runs: data::RunsFile,
    current_run: usize,
    center : Option<String>,
    lock_frozen : bool,
}

enum Msg {
    Prox(usize),
    Prox1,
    Wait,
    Recalculate,
    ToggleFrozen,
    // FetchedRuns(fetch::Result<data::RunsFile>),
    // FetchedContest(fetch::Result<data::ContestFile>),
    Reset,
    Fetched(
        fetch::Result<data::RunsFile>,
        fetch::Result<data::ContestFile>),
}

async fn fetch_all() -> Msg {
    let r = fetch_allruns().await;
    let c = fetch_contest().await;
    Msg::Fetched(r, c)
}

// fn apply_run_model(model: &mut Model) {
//     if model.current_run < model.runs.runs.len() {
//         let mut run = model.runs.runs[model.current_run].clone();
//         run.answer = data::Answer::Wait;
//         model.contest.apply_run(&run).unwrap();
//     }
// }

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::ToggleFrozen => {
            model.lock_frozen = !model.lock_frozen;
        },
        Msg::Wait => {
            if model.current_run < model.runs.runs.len() {
                let mut run = model.runs.runs[model.current_run].clone();
                run.answer = data::Answer::Wait;
                model.contest.apply_run(&run).unwrap();
            }
            orders.perform_cmd(cmds::timeout(2000, move || Msg::Recalculate));
        }
        Msg::Recalculate => {
            if model.current_run < model.runs.runs.len() {
                let run = &model.runs.runs[model.current_run];
                model.contest.apply_run(run).unwrap();
                model.current_run += 1;
            }
            model.contest.recalculate_placement().unwrap();
        },
        Msg::Prox1 => {
            if model.current_run < model.runs.runs.len() {
                let run = &model.runs.runs[model.current_run];
                if run.time < model.contest.score_freeze_time || !model.lock_frozen {
                    model.center = Some(run.team_login.clone());
                    orders.perform_cmd(cmds::timeout(5000, move || Msg::Wait));
                }
            }
            else {
                model.center = None;
            }
        }
        Msg::Prox(n) => {
            model.center = None;
            for _ in 0..n {
                if model.current_run < model.runs.runs.len() {
                    let run = &model.runs.runs[model.current_run];
                    if run.time < model.contest.score_freeze_time || !model.lock_frozen {
                        // log!("run time? ", run.time, " -> ", model.contest.score_freeze_time);
                        model.contest.apply_run(run).unwrap();
                        model.current_run += 1;
                    }
                }
            }
            model.contest.recalculate_placement().unwrap();
        },
        Msg::Fetched(Ok(runs), Ok(contest)) => {
            model.current_run = 0;
            model.center = None;
            model.runs = runs;
            model.runs.runs.reverse();
            model.contest = contest;
            model.contest.reload_score().unwrap();
        },
        Msg::Fetched(Err(e), _) => {
            log!("fetched runs error!", e)
        },
        Msg::Fetched(_, Err(e)) => {
            log!("fetched contest error!", e)
        },
        Msg::Reset => {
            orders.skip().perform_cmd( fetch_all() );    
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    let frozen = if model.lock_frozen {"Frozen Locked"} else { "Frozen Unlocked"};
    div![
        div![
            C!["commandpanel"],
            button!["+1", ev(Ev::Click, |_| Msg::Prox1),],
            button!["+10", ev(Ev::Click, |_| Msg::Prox(10)),],
            button!["+100", ev(Ev::Click, |_| Msg::Prox(100)),],
            button!["+1000", ev(Ev::Click, |_| Msg::Prox(1000)),],
            button!["Reset", ev(Ev::Click, |_| Msg::Reset),],
            button![frozen, ev(Ev::Click, |_| Msg::ToggleFrozen),],
            div!["Runs: ", model.current_run, "/", model.runs.runs.len()],
        ],
        views::view_scoreboard(&model.contest, &model.center, &model.url_filter),
    ]
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
