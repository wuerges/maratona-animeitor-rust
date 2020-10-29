use maratona_animeitor_rust::data;
use seed::{prelude::*, *};
use crate::views;
use crate::requests::*;

extern crate rand;


fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    // orders.skip().perform_cmd( fetch_all() );
    orders.send_msg(Msg::Reset);
    Model { 
        contest: data::ContestFile::dummy(),
        runs: data::RunsFile::empty(),
        current_run: 0,
        center: None
    }
}

struct Model {
    contest : data::ContestFile,
    runs: data::RunsFile,
    current_run: usize,
    center : Option<String>,
}

enum Msg {
    Prox(usize),
    Prox1,
    Wait,
    Recalculate,
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


fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
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
                model.center = Some(run.team_login.clone());
                orders.perform_cmd(cmds::timeout(1000, move || Msg::Wait));
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
                    model.contest.apply_run(run).unwrap();
                    model.current_run += 1;
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
    let margin_top = match &model.center {
        None => 100,
        Some(s) => {
            let h = window().inner_height().unwrap().as_f64().unwrap() as i64 / 2;
            let p = model.contest.teams[s].placement as i64;
            h + -p * 90
        },
    };
    div![
        div![
            style!{St::Position => "absolute", St::Top => px(10), St::ZIndex=>123123 },
            button!["+1", ev(Ev::Click, |_| Msg::Prox1),],
            button!["+10", ev(Ev::Click, |_| Msg::Prox(10)),],
            button!["+100", ev(Ev::Click, |_| Msg::Prox(100)),],
            button!["+1000", ev(Ev::Click, |_| Msg::Prox(1000)),],
            button!["Reset", ev(Ev::Click, |_| Msg::Reset),],
            div!["Runs: ", model.current_run, "/", model.runs.runs.len()],
        ],
        views::view_scoreboard(&model.contest, margin_top),
    ]
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
