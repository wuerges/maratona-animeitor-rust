use crate::helpers::*;
use crate::requests::*;
use crate::views;
use data::revelation::RevelationDriver;
use seed::{prelude::*, *};

extern crate rand;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.send_msg(Msg::Reset);
    Model {
        button_disabled: false,
        secret: get_secret(&url),
        revelation: None,
        center: None,
        vencedor: None,
    }
}

struct Model {
    button_disabled: bool,
    secret: String,
    center: Option<String>,
    revelation: Option<RevelationDriver>,
    vencedor: Option<String>,
}

impl Model {
    fn remaining(&self) -> usize {
        self.revelation.as_ref().map(|r| r.len()).unwrap_or(0)
    }
}

enum Msg {
    Prox(usize),
    Scroll(usize),
    Prox1,
    Scroll1,
    Reset,
    Unlock,
    Fetched(
        fetch::Result<data::RunsFile>,
        fetch::Result<data::ContestFile>,
        fetch::Result<data::configdata::ConfigContest>,
    ),
}

async fn fetch_all(secret: String) -> Msg {
    let r = fetch_allruns_secret(&secret).await;
    let c = fetch_contest().await;
    let cfg = fetch_config().await;
    Msg::Fetched(r, c, cfg)
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Prox1 => {
            model.button_disabled = true;
            let next_center = model.revelation.as_mut().and_then(|r| r.peek());
            if next_center == model.center.as_ref() {
                orders.send_msg(Msg::Scroll1);
            } else {
                let delay = 1000;

                model.center = next_center.cloned();

                orders.perform_cmd(cmds::timeout(delay, move || Msg::Scroll1));
            }
        }
        Msg::Scroll1 => {
            model.center = model
                .revelation
                .as_mut()
                .and_then(|r| {
                    r.reveal_step();
                    r.peek()
                })
                .cloned();

            model.button_disabled = false;
        }
        Msg::Prox(n) => {
            model.button_disabled = true;
            orders.send_msg(Msg::Scroll(n));
        }
        Msg::Scroll(n) => {
            model.center = model
                .revelation
                .as_mut()
                .and_then(|r| {
                    r.reveal_top_n(n);
                    r.peek()
                })
                .cloned();

            orders.perform_cmd(cmds::timeout(5000, move || Msg::Unlock));
        }
        Msg::Unlock => {
            model.button_disabled = false;
        }
        Msg::Fetched(Ok(runs), Ok(contest), Ok(cfg)) => {
            model.revelation = Some(RevelationDriver::new(contest, runs));
            model.center = None;
            model.button_disabled = false;
        }
        Msg::Fetched(Err(e), _, _) => log!("fetched runs error!", e),
        Msg::Fetched(_, Err(e), _) => log!("fetched contest error!", e),
        Msg::Fetched(_, _, Err(e)) => log!("fetched contest config error!", e),
        Msg::Reset => {
            model.button_disabled = true;
            orders.skip().perform_cmd(fetch_all(model.secret.clone()));
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    let button_disabled = if model.button_disabled {
        attrs! {At::Disabled => true}
    } else {
        attrs! {}
    };
    div![
        div![
            C!["commandpanel"],
            button!["+1", ev(Ev::Click, |_| Msg::Prox1), button_disabled.clone()],
            button![
                "All",
                ev(Ev::Click, |_| Msg::Prox(0)),
                button_disabled.clone()
            ],
            button![
                "Top 10",
                ev(Ev::Click, |_| Msg::Prox(10)),
                button_disabled.clone()
            ],
            button![
                "Top 30",
                ev(Ev::Click, |_| Msg::Prox(30)),
                button_disabled.clone()
            ],
            button![
                "Top 50",
                ev(Ev::Click, |_| Msg::Prox(50)),
                button_disabled.clone()
            ],
            button![
                "Top 100",
                ev(Ev::Click, |_| Msg::Prox(100)),
                button_disabled.clone()
            ],
            button!["Reset", ev(Ev::Click, |_| Msg::Reset), button_disabled],
            div![
                C!["vencedor"],
                model
                    .vencedor
                    .as_ref()
                    .map(|v| format!("Vencedor da sede: {}", v)),
            ],
            div!["Times: ", model.remaining()],
        ],
        div![
            style! {St::Position => "relative", St::Top => px(60)},
            model.revelation.as_ref().map(|r| views::view_scoreboard(
                r.contest(),
                &model.center,
                None
            )),
        ],
    ]
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
