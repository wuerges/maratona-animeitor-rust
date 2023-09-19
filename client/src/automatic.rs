use data::configdata::Sede;
use data::RunsFile;
use seed::{prelude::*, *};

use crate::helpers::*;
use crate::requests::*;
use crate::views;

extern crate rand;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    orders.perform_cmd(fetch_all());
    orders.stream(streams::interval(1_000, || Msg::Reload));
    orders.stream(streams::interval(1_000, || Msg::WsReconnect));

    Model {
        runs: RunsFile::empty(),
        state: ContestState::Unloaded {
            sede: get_sede(&url),
        },
        ws: None,
    }
}

fn build_ws_connection(orders: &mut impl Orders<Msg>) -> Result<WebSocket, WebSocketError> {
    WebSocket::builder(get_ws_url("/allruns_ws"), orders)
        .on_message(Msg::RunUpdate)
        .on_close(Msg::WsClosed)
        .on_error(Msg::WsError)
        .build_and_open()
}

struct Model {
    ws: Option<WebSocket>,
    runs: data::RunsFile,
    state: ContestState,
}

struct LoadedContest {
    center: Option<String>,
    sede: Option<Sede>,
    original: data::ContestFile,
    contest: data::ContestFile,
    config: data::configdata::ConfigContest,
    dirty: bool,
}

impl LoadedContest {
    fn new(
        sede_name: Option<String>,
        contest: data::ContestFile,
        config: data::configdata::ConfigContest,
    ) -> Self {
        let sede = sede_name.and_then(|name| config.get_sede_nome_sede(&name));
        Self {
            center: None,
            sede,
            original: contest.clone(),
            contest,
            config,
            dirty: true,
        }
    }
}

enum ContestState {
    Loaded(LoadedContest),
    Unloaded { sede: Option<String> },
}

impl ContestState {
    fn get_sede_name(&self) -> Option<String> {
        match self {
            ContestState::Loaded(loaded) => loaded.sede.as_ref().map(|s| s.name.clone()),
            ContestState::Unloaded { sede } => sede.clone(),
        }
    }
}

enum Msg {
    UrlChanged(subs::UrlChanged),
    RunUpdate(WebSocketMessage),
    Reload,
    WsReconnect,
    Fetched(
        fetch::Result<data::ContestFile>,
        fetch::Result<data::configdata::ConfigContest>,
    ),
    WsClosed(CloseEvent),
    WsError(),
}

async fn fetch_all() -> Msg {
    let c = fetch_contest().await;
    let cfg = fetch_config().await;

    Msg::Fetched(c, cfg)
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            let new_sede_name = get_sede(&url);
            match &mut model.state {
                ContestState::Loaded(state) => {
                    state.sede =
                        new_sede_name.and_then(|name| state.config.get_sede_nome_sede(&name));
                    state.dirty = true;
                }
                ContestState::Unloaded { sede: sede_name } => {
                    *sede_name = new_sede_name;
                }
            }
            orders.skip().send_msg(Msg::Reload);
        }
        Msg::Reload => match &mut model.state {
            ContestState::Unloaded { .. } => {
                orders.skip();
            }
            ContestState::Loaded(state) => {
                if state.dirty {
                    state.contest = state.original.clone();

                    for r in model.runs.sorted() {
                        state.contest.apply_run(&r);
                    }
                    match state.contest.recalculate_placement(state.sede.as_ref()) {
                        Ok(()) => (),
                        Err(error) => {
                            log!("failed to recalculate placement", error);
                            orders.skip().perform_cmd(fetch_all());
                        }
                    };
                    state.dirty = false;
                } else {
                    orders.skip();
                }
            }
        },
        Msg::RunUpdate(m) => {
            orders.skip();

            match m.json::<data::RunTuple>() {
                Ok(run) => {
                    if model.runs.refresh_1(&run) {
                        match &mut model.state {
                            ContestState::Loaded(state) => {
                                state.dirty = true;
                            }
                            ContestState::Unloaded { .. } => (),
                        }
                    }
                }
                Err(e) => {
                    log!("Websocket error: {}", e);
                    model.ws = None;
                }
            }
        }
        Msg::Fetched(Ok(contest), Ok(config)) => {
            model.state = ContestState::Loaded(LoadedContest::new(
                model.state.get_sede_name(),
                contest,
                config,
            ));
            orders.skip();
        }
        Msg::Fetched(Err(e), _) => {
            log!("failed fetching contest: ", e);
            orders.skip();
        }
        Msg::Fetched(_, Err(e)) => {
            log!("failed fetching config: ", e);
            orders.skip();
        }
        Msg::WsReconnect => {
            orders.skip();
            if model.ws.is_none() {
                match build_ws_connection(orders) {
                    Ok(conn) => model.ws = Some(conn),
                    Err(err) => log!("failed to connect websocket", err),
                }
            }
        }
        Msg::WsClosed(error) => {
            log!("websocket closed", error);
            orders.skip();
            model.ws = None;
        }
        Msg::WsError() => {
            log!("websocket error");
            model.ws = None;
            orders.skip();
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    match model.state {
        ContestState::Loaded(ref state) => {
            views::view_scoreboard(&state.contest, &state.center, state.sede.as_ref(), false)
        }
        ContestState::Unloaded { .. } => div!["Contest not ready yet!"],
    }
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
