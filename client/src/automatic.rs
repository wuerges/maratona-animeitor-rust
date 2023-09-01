use seed::{prelude::*, *};

use crate::helpers::*;
use crate::requests::*;
use crate::views;

extern crate rand;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    orders.perform_cmd(fetch_all());
    orders.stream(streams::interval(1_000, || Msg::Reload));

    Model {
        center: None,
        sede: get_sede(&url),
        original: data::ContestFile::dummy(),
        contest: None,
        config: data::configdata::ConfigContest::dummy(),
        runs: data::RunsFile::empty(),
        ws: None,
        dirty: true,
    }
}

struct Model {
    center: Option<String>,
    sede: Option<String>,
    original: data::ContestFile,
    contest: Option<data::ContestFile>,
    config: data::configdata::ConfigContest,
    runs: data::RunsFile,
    ws: Option<WebSocket>,
    dirty: bool,
}

enum Msg {
    UrlChanged(subs::UrlChanged),
    RunUpdate(WebSocketMessage),
    Reload,
    Fetched(
        fetch::Result<data::ContestFile>,
        fetch::Result<data::configdata::ConfigContest>,
    ),
}

async fn fetch_all() -> Msg {
    let c = fetch_contest().await;
    let cfg = fetch_config().await;

    Msg::Fetched(c, cfg)
}

async fn reload() -> Msg {
    Msg::Reload
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            model.sede = get_sede(&url);
            model.dirty = true;
            orders.skip().perform_cmd(reload());
        }
        Msg::Reload => {
            match model.contest.as_mut() {
                None => {
                    // retrying to fetch contest
                    orders.perform_cmd(fetch_all());
                }
                Some(contest) => {
                    // log!("reload!");
                    let url_filter = model.sede.as_ref().and_then(|sede| {
                        model
                            .config
                            .get_sede_nome_sede(sede)
                            .as_ref()
                            .map(|s| s.codes.clone())
                    });
                    if model.dirty {
                        // log!("reload dirty!");
                        *contest = model.original.clone();
                        for r in &model.runs.sorted() {
                            contest.apply_run(r);
                        }
                        contest
                            .recalculate_placement(url_filter.as_ref())
                            .expect("Should recalculate scores");
                        model.dirty = false;
                    } else {
                        // log!("reload clean!");
                        orders.skip();
                    }
                }
            }
        }
        Msg::RunUpdate(m) => match m.json::<data::RunTuple>() {
            Ok(run) => {
                if model.runs.refresh_1(&run) {
                    model.dirty = true;
                }
                orders.skip();
            }
            Err(e) => {
                log!("Websocket error: {}", e);
                orders.perform_cmd(fetch_all());
            }
        },
        Msg::Fetched(Ok(contest), Ok(config)) => {
            model.original = contest.clone();
            model.contest = Some(contest);
            model.config = config;
            model.dirty = true;
            model.ws = Some(
                WebSocket::builder(get_ws_url("/allruns_ws"), orders)
                    .on_message(Msg::RunUpdate)
                    .build_and_open()
                    .expect("Open WebSocket"),
            );
            orders.skip().perform_cmd(reload());
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
    let opt_sede = model
        .sede
        .as_ref()
        .and_then(|sede| model.config.get_sede_nome_sede(sede));
    match model.contest {
        None => div!["Contest not ready yet!"],
        Some(ref contest) => views::view_scoreboard(contest, &model.center, opt_sede, false),
    }
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
