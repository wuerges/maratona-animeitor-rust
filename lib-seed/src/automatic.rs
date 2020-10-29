use maratona_animeitor_rust::data;
use seed::{prelude::*, *};

use crate::requests::*;
use crate::views;

extern crate rand;

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.skip().perform_cmd(fetch_all());
    Model { 
        contest: data::ContestFile::dummy(),
        runs: data::RunsFile::empty(),
    }
}

struct Model {
    contest : data::ContestFile,
    runs: data::RunsFile,
}

enum Msg {
    Reload,
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
        Msg::Reload => {
            orders.perform_cmd(fetch_all());
        }
        Msg::Fetched(Ok(runs), Ok(contest)) => {
            model.runs = runs;
            model.runs.runs.reverse();
            model.contest = contest;

            for r in model.runs.runs.iter() {
                model.contest.apply_run(r).unwrap();
            }
            model.contest.recalculate_placement().unwrap();
            log!("fetched runs and contest!", model.contest);

            orders.perform_cmd(cmds::timeout(1000, || Msg::Reload)); 
        },
        Msg::Fetched(Err(e), Ok(_)) => {
            log!("failed fetching runs: ", e);
            orders.perform_cmd(cmds::timeout(1000, || Msg::Reload)); 
        },
        Msg::Fetched(_, Err(e)) => {
            log!("failed fetching contest: ", e);
            orders.perform_cmd(cmds::timeout(1000, || Msg::Reload)); 
        },

    }
}

fn view(model: &Model) -> Node<Msg> {
    views::view_scoreboard(&model.contest, 100)
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
