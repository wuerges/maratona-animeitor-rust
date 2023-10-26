use crate::helpers::*;
use crate::requests::*;
use crate::views;
use data::{configdata::Sede, revelation::RevelationDriver};
use seed::{prelude::*, *};

extern crate rand;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    let event_stream = orders.stream_with_handle(streams::window_event(Ev::KeyDown, |event| {
        Msg::KeyPressed(event.unchecked_into())
    }));
    let secret = get_secret(&url);

    orders.skip().perform_cmd(fetch_all(secret));

    Model {
        button_disabled: true,
        revelation: None,
        center: None,
        sede: get_sede(&url),
        opt_sede: None,
        event_stream,
    }
}

#[derive(Debug)]
struct Model {
    button_disabled: bool,
    center: Option<String>,
    revelation: Option<RevelationDriver>,
    sede: Option<String>,
    opt_sede: Option<Sede>,

    #[allow(dead_code)]
    event_stream: StreamHandle,
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
    Back1,
    Scroll1,
    Reset,
    Unlock,
    KeyPressed(web_sys::KeyboardEvent),
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
                    r.reveal_step().ok();
                    r.peek()
                })
                .cloned();

            model.button_disabled = false;
        }
        Msg::Back1 => {
            model.center = model.revelation.as_mut().and_then(|r| {
                r.back_one().ok();
                r.peek().cloned()
            });
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
                    r.reveal_top_n(n).ok();
                    r.peek()
                })
                .cloned();

            orders.perform_cmd(cmds::timeout(5000, move || Msg::Unlock));
        }
        Msg::Unlock => {
            model.button_disabled = false;
        }
        Msg::KeyPressed(event) => match event.key().as_str() {
            "ArrowRight" => {
                orders.send_msg(Msg::Prox1);
            }
            "ArrowLeft" => {
                orders.send_msg(Msg::Back1);
            }
            _ => (),
        },
        Msg::Fetched(Ok(runs), Ok(contest), Ok(config)) => {
            model.opt_sede = model
                .sede
                .as_ref()
                .and_then(|sede_name| config.into_contest().get_sede_nome_sede(sede_name).cloned());

            let contest = match model.opt_sede.as_ref() {
                Some(sede) => contest.filter_sede(sede),
                None => contest,
            };

            model.revelation = RevelationDriver::new(contest, runs).ok();
            model.center = None;
            model.button_disabled = false;
        }
        Msg::Fetched(Err(e), _, _) => log!("fetched runs error!", e),
        Msg::Fetched(_, Err(e), _) => log!("fetched contest error!", e),
        Msg::Fetched(_, _, Err(e)) => log!("fetched config error!", e),
        Msg::Reset => {
            if let Some(r) = &mut model.revelation {
                r.restart();
            }
            model.center = None;
            model.button_disabled = false;
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
            button!["←", ev(Ev::Click, |_| Msg::Back1), button_disabled.clone()],
            button!["→", ev(Ev::Click, |_| Msg::Prox1), button_disabled.clone()],
            button![
                "Top 100",
                ev(Ev::Click, |_| Msg::Prox(100)),
                button_disabled.clone()
            ],
            button![
                "Top 50",
                ev(Ev::Click, |_| Msg::Prox(50)),
                button_disabled.clone()
            ],
            button![
                "Top 30",
                ev(Ev::Click, |_| Msg::Prox(30)),
                button_disabled.clone()
            ],
            button![
                "Top 10",
                ev(Ev::Click, |_| Msg::Prox(10)),
                button_disabled.clone()
            ],
            button![
                "All",
                ev(Ev::Click, |_| Msg::Prox(0)),
                button_disabled.clone()
            ],
            button!["Reset", ev(Ev::Click, |_| Msg::Reset), button_disabled],
            div!["Times: ", model.remaining()],
        ],
        div![
            style! {St::Position => "relative", St::Top => px(60)},
            model.revelation.as_ref().map(|r| views::view_scoreboard(
                r.contest(),
                &model.center,
                model.opt_sede.as_ref(),
                true
            )),
        ],
    ]
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
