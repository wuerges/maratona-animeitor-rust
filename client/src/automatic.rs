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
        // url_filter: get_url_filter(&url),
        sede: get_sede(&url),
        original: data::ContestFile::dummy(),
        contest: data::ContestFile::dummy(),
        config: data::configdata::ConfigContest::dummy(),
        runs: data::RunsFile::empty(),
        ws: None,
        dirty: true,
    }
}

struct Model {
    center: Option<String>,
    // url_filter: Option<Vec<String>>,
    sede: Option<String>,
    original: data::ContestFile,
    contest: data::ContestFile,
    config: data::configdata::ConfigContest,
    runs: data::RunsFile,
    ws: Option<WebSocket>,
    dirty: bool,
}

enum Msg {
    UrlChanged(subs::UrlChanged),
    RunUpdate(WebSocketMessage),
    Reload,
    Fetched(fetch::Result<data::ContestFile>, fetch::Result<data::configdata::ConfigContest>),
}

async fn fetch_all() -> Msg {
    let c = fetch_contest().await;
    let cfg = fetch_config().await;

    Msg::Fetched(c, cfg)
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            model.sede = get_sede(&url);
        }
        Msg::Reload => {
            // log!("reload!");
            if model.dirty {
                // log!("reload dirty!");
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
            else {
                // log!("reload clean!");
                orders.skip();
            }
        }
        Msg::RunUpdate(m) => {
            match m.json::<data::RunTuple>() {
                Ok(run) => {
                    if model.runs.refresh_1(&run) {
                        model.dirty = true;
                    }
                    orders.skip();
                },
                Err(e) => {
                    log!("Websocket error: {}", e);
                    orders.perform_cmd(fetch_all());                    
                }
            }
            // let run: data::RunTuple = m.json().expect("Should be a RunTuple");
            
        }
        Msg::Fetched(Ok(contest), Ok(config)) => {
            model.original = contest;
            model.config = config;
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
        Msg::Fetched(Err(e), _) => {
            log!("failed fetching contest: ", e);
        }
        Msg::Fetched(_, Err(e)) => {
            log!("failed fetching config: ", e);
        }
    }
}

fn view(model: &Model) -> Node<Msg> {

    let opt_sede = model.config.sedes.iter().filter(|s| &s.name == model.sede.as_ref().unwrap_or(&"<fake>".to_string()) ).next();
    views::view_scoreboard(&model.contest, &model.center, opt_sede)
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
