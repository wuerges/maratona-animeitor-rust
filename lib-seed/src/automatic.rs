use maratona_animeitor_rust::data;
use seed::{prelude::*, *};

use crate::requests::*;
use crate::helpers::*;
use crate::views;

extern crate rand;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.skip().send_msg(Msg::Reload);
    orders.skip().stream(streams::interval(30_000, || Msg::Reload));

    Model {
        source : get_source(&url),
        center : None,
        url_filter : get_url_filter(&url),
        contest: data::ContestFile::dummy(),
        runs: data::RunsFile::empty(),
    }
}

struct Model {
    source : String,
    center : Option<String>,
    url_filter: Option<Vec<String>>,
    contest : data::ContestFile,
    runs: data::RunsFile,
}

enum Msg {
    Reload,
    Recenter,
    Fetched(
        fetch::Result<data::RunsFile>,
        fetch::Result<data::ContestFile>),
}

async fn fetch_all(source : String) -> Msg {
    let r = fetch_allruns(&source).await;
    let c = fetch_contest(&source).await;
    Msg::Fetched(r, c)
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Recenter => {
            model.center = None;
        },
        Msg::Reload => {
            orders.skip().perform_cmd(fetch_all(model.source.clone()));
        },
        Msg::Fetched(Ok(runs), Ok(contest)) => {
            
            model.runs = runs;
            model.runs.runs.reverse();
            model.contest = contest;
            
            for r in model.runs.runs.iter() {
                if r.time < model.contest.score_freeze_time {
                    model.contest.apply_run(r).unwrap();
                }
            }
            
            let old_contest = model.contest.clone();
            model.contest.recalculate_placement().unwrap();
            for (t1, t2) in model.contest.teams.values().zip(old_contest.teams.values()) {
                if t1.placement < t2.placement {
                    model.center = Some(t1.login.clone());
                    break;
                }
            }            
            
            orders.perform_cmd(cmds::timeout(10_000, move|| Msg::Recenter));
            // log!("fetched runs and contest!", model.contest);
        },
        Msg::Fetched(Err(e), Ok(_)) => {
            log!("failed fetching runs: ", e);
        },
        Msg::Fetched(_, Err(e)) => {
            log!("failed fetching contest: ", e);
        },

    }
}

fn view(model: &Model) -> Node<Msg> {
    views::view_scoreboard(&model.contest, &model.center, &model.url_filter)
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
