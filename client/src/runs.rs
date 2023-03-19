use crate::helpers::*;
use crate::requests::*;
use crate::views;

use seed::{prelude::*, *};

extern crate rand;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    orders.skip().perform_cmd(fetch_all());
    orders.stream(streams::interval(1_000, || Msg::Reset));
    Model {
        url_filter: get_url_filter(&url),
        runs_file: data::RunsFile::empty(),
        runs: Vec::new(),
        contest: data::ContestFile::dummy(),
        ws: None,
        dirty: true,
    }
}

struct Model {
    url_filter: Option<Vec<String>>,
    runs_file: data::RunsFile,
    runs: Vec<data::RunsPanelItem>,
    contest: data::ContestFile,
    ws: Option<WebSocket>,
    dirty: bool,
}

enum Msg {
    Reset,
    UrlChanged(subs::UrlChanged),
    Fetched(fetch::Result<data::ContestFile>),
    RunUpdate(WebSocketMessage),
}

async fn fetch_all() -> Msg {
    let f = fetch_contest().await;
    Msg::Fetched(f)
}
async fn reset() -> Msg {
    Msg::Reset
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            model.url_filter = get_url_filter(&url);
            model.dirty = true;
            orders.skip().perform_cmd(reset());
        }
        Msg::RunUpdate(m) => {
            let run: data::RunTuple = m.json().expect("Expected a RunTuple");
            if model.runs_file.refresh_1(&run) {
                model.dirty = true;
            }
            orders.skip();
        }
        Msg::Fetched(Ok(contest)) => {
            model.contest = contest;
            model.ws = Some(
                WebSocket::builder(get_ws_url("/allruns_ws"), orders)
                    .on_message(Msg::RunUpdate)
                    .build_and_open()
                    .expect("Open WebSocket"),
            );
            orders.skip();
        }
        Msg::Fetched(Err(e)) => log!("fetched runs error!", e),
        Msg::Reset => {
            if model.dirty {
                let mut mock = model.contest.clone();
                let mut runs = model.runs_file.sorted();
                for r in &runs {
                    mock.apply_run(r);
                }
                mock.recalculate_placement(model.url_filter.as_ref())
                    .expect("Should recalculate placement");

                runs.reverse();

                model.runs = runs
                    .into_iter()
                    .map(|r| mock.build_panel_item(&r).expect("Expected a valid Run"))
                    .collect();

                model.dirty = false;

                // for r in &model.runs {
                //     log!("run:", r);
                // }
            } else {
                orders.skip();
            }
        }
    }
}

fn get_answer(t: &data::Answer) -> &str {
    match t {
        data::Answer::Yes(_) => "answeryes",
        data::Answer::No => "answerno",
        data::Answer::Wait => "answerwait",
        _ => "answererror",
    }
}

fn get_image(t: &data::Answer) -> &str {
    match t {
        data::Answer::Yes(_) => "/static/assets/balloon-border.svg",
        data::Answer::No => "/static/assets/no.svg",
        data::Answer::Wait => "/static/assets/question.svg",
        _ => "/static/assets/question.svg",
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![
        C!["runstable"],
        model.runs.iter().filter(
            |r|data::check_filter_login(model.url_filter.as_ref(), &r.team_login)
        ).take(30).enumerate().map({
            |(i, r)| {
                let balao = std::format!("balao_{}", r.problem);
                div![
                    C!["run"],
                    style! {
                        St::Top => format!("calc(var(--row-height) * {} + var(--root-top))", i),
                    },
                    div![
                        C!["cell", "colocacao", "quadrado", views::get_color(r.placement, None)],
                        r.placement
                    ],
                    div![
                        C!["cell", "time"],
                        div![C!["nomeEscola"], &r.escola],
                        div![C!["nomeTime"], &r.team_name],
                    ],
                    div![
                        C!["cell", "resposta", "quadrado", get_answer(&r.result)],
                        IF!(matches!(r.result, data::Answer::Yes(_)) =>
                        div![
                            img![
                                C!["answer-img", balao],
                                attrs!{At::Src => "/static/assets/balloon.svg"},
                            ],
                        ]),
                        img![
                            C!["answer-img"],
                            attrs!{At::Src => get_image(&r.result)},
                        ],
                        div![
                            C!["answer-text"],
                            &r.problem
                        ]
                    ],

                    attrs!{At::OnClick =>
                        std::format!("document.getElementById('foto_{}').style.display = 'block';", &r.team_login),
                    },
                ]
            }
        })
    ]
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
