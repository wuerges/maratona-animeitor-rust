use data;
use seed::{prelude::*, *};

use crate::helpers::*;
use crate::requests::*;
use crate::views;

extern crate rand;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    orders.perform_cmd(fetch_all());

    Model {
        center: None,
        url_filter: get_url_filter(&url),
        original: data::ContestFile::dummy(),
        contest: data::ContestFile::dummy(),
        runs: data::RunsFile::empty(),
        ws: None,
        dirty: true,
    }
}

struct Model {
    center: Option<String>,
    url_filter: Option<Vec<String>>,
    original: data::ContestFile,
    contest: data::ContestFile,
    runs: data::RunsFile,
    ws: Option<WebSocket>,
    dirty: bool,
}

enum Msg {
    UrlChanged(subs::UrlChanged),
    RunUpdate(WebSocketMessage),
    Reload,
    Fetched(fetch::Result<data::ContestFile>),
}

async fn fetch_all() -> Msg {
    let c = fetch_contest().await;
    Msg::Fetched(c)
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            model.url_filter = get_url_filter(&url);
        }
        // Msg::Recenter => {
        //     model.center = None;
        // }
        Msg::Reload => {
            if model.dirty {
                model.contest = model.original.clone();
                for r in &model.runs.sorted() {
                    model
                        .contest
                        .apply_run(r)
                        .expect("Should be able to apply the run");
                }
                model
                    .contest
                    .recalculate_placement()
                    .expect("Should recalculate scores");
                model.dirty = false;
            }
        }
        Msg::RunUpdate(m) => {
            let run: data::RunTuple = m.json().expect("Should be a RunTuple");
            if model.runs.refresh_1(&run) {
                model.dirty = true;
            }
        }
        Msg::Fetched(Ok(contest)) => {
            model.original = contest;
            model.dirty = true;
            model.ws = Some(
                WebSocket::builder(get_ws_url("/allruns_ws"), orders)
                    .on_message(Msg::RunUpdate)
                    .build_and_open()
                    .expect("Open WebSocket"),
            );
            orders
                .skip()
                .stream(streams::interval(1_000, || Msg::Reload));
        }
        Msg::Fetched(Err(e)) => {
            log!("failed fetching contest: ", e);
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    views::view_scoreboard(&model.contest, &model.center, &model.url_filter)
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
