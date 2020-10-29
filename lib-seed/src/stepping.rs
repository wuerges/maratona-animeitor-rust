use maratona_animeitor_rust::data;
use seed::{prelude::*, *};
use crate::views;
use crate::requests;

extern crate rand;


fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.skip().perform_cmd({
        async {
            let m = requests::fetch_contest().await;
            Msg::FetchedContest(m)
        }
    });
    Model { 
        contest: data::ContestFile::dummy(),
        runs: data::RunsFile::empty(),
        current_run: 0,
    }
}

struct Model {
    contest : data::ContestFile,
    runs: data::RunsFile,
    current_run: usize,
}

enum Msg {
    Prox(usize),
    FetchedRuns(fetch::Result<data::RunsFile>),
    FetchedContest(fetch::Result<data::ContestFile>),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Prox(n) => {
            for _ in 0..n {
                if model.current_run < model.runs.runs.len() {
                    let run = &model.runs.runs[model.current_run];
                    model.contest.apply_run(run).unwrap();
                    model.current_run += 1;
                }
            }
            model.contest.recalculate_placement().unwrap();
        },
        Msg::FetchedRuns(Ok(runs)) => {
            model.runs = runs;
            model.runs.runs.reverse();
        },
        Msg::FetchedContest(Ok(contest)) => {
            model.contest = contest;
            model.contest.reload_score().unwrap();
            orders.perform_cmd({
                async { Msg::FetchedRuns(requests::fetch_allruns().await) }
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

fn view(model: &Model) -> Node<Msg> {
    let margin_top = 100;
    div![
        button!["+1", ev(Ev::Click, |_| Msg::Prox(1)),],
        button!["+10", ev(Ev::Click, |_| Msg::Prox(10)),],
        button!["+100", ev(Ev::Click, |_| Msg::Prox(100)),],
        button!["+1000", ev(Ev::Click, |_| Msg::Prox(1000)),],
        div!["Runs: ", model.current_run, "/", model.runs.runs.len()],
        views::view_scoreboard(&model.contest, margin_top),
    ]
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
