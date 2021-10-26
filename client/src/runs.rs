use crate::helpers::*;
use crate::requests::*;
use crate::views;
use data;
use seed::{prelude::*, *};

extern crate rand;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    orders.skip().perform_cmd(fetch_all());
    orders.stream(streams::interval(1_000, || Msg::Reset));
    Model {
        url_filter: get_url_filter(&url),
        runs_file: data::RunsFile::empty(),
        runs : Vec::new(),
        contest: data::ContestFile::dummy(),
        ws: None,
        dirty : true,
    }
}

struct Model {
    url_filter: Option<Vec<String>>,
    runs_file: data::RunsFile,
    runs : Vec<data::RunsPanelItem>,
    contest: data::ContestFile,
    ws: Option<WebSocket>,
    dirty : bool,
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

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            model.url_filter = get_url_filter(&url);
        },
        Msg::RunUpdate(m) => {
            let run : data::RunTuple = m.json().expect("Expected a RunTuple");
            if model.runs_file.refresh_1(&run) {
                model.dirty = true;
            }            
        },
        Msg::Fetched(Ok(contest)) => {
            model.contest = contest;
            model.ws = Some(WebSocket::builder(get_ws_url("/allruns_ws"), orders)
                .on_message(Msg::RunUpdate)
                .build_and_open()
                .expect("Open WebSocket"));
        }
        Msg::Fetched(Err(e)) => log!("fetched runs error!", e),
        Msg::Reset => {
            if model.dirty {
                let mut mock = model.contest.clone();
                let mut runs = model.runs_file.sorted();
                for r in &runs {
                    mock.apply_run(r).expect("Should apply run just fine");
                }
                mock.recalculate_placement().expect("Should recalculate placement");

                runs.reverse();

                model.runs = runs.into_iter().take(30).map(|r| {
                    mock.build_panel_item(&r).expect("Expected a valid Run")
                }).collect();

                model.dirty = false;

                // for r in &model.runs {
                //     log!("run:", r);
                // }
            }
            else {
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

fn view(model: &Model) -> Node<Msg> {
    div![
        C!["runstable"],
        model.runs.iter().filter(
            |r|check_filter_login(model.url_filter.as_ref(), &r.team_login)
        ).enumerate().map({
            |(i, r)| {
                let pnum = data::PROBLEM_LETTERS.find(r.problem.as_str()).unwrap_or(0);
                let hue = get_answer_hue_deg(model.contest.number_problems, pnum as u32);
                let balao = std::format!("balao_{}", r.problem);
                div![
                    C!["run"],
                    style! {
                        St::Top => format!("calc(var(--row-height) * {} + var(--root-top))", i),
                    },
                    div![
                        C!["cell", "colocacao", views::get_color(r.placement, None)],
                        r.placement
                    ],
                    div![
                        C!["cell", "time"],
                        div![C!["nomeEscola"], &r.escola],
                        div![C!["nomeTime"], &r.team_name],
                    ],
                    div![C!["cell", "problema"], &r.problem],
                    div![
                        C!["cell", "resposta", get_answer(&r.result)],
                        IF!(matches!(r.result, data::Answer::Yes(_)) => 
                        div![
                            C!["balao", balao],
                            style!{ St::Filter => format!("hue-rotate({}deg)", hue)},
                        ]),
                    ],

                    attrs!{At::OnClick => 
                        std::format!("document.getElementById('foto_{}').style.display = 'block';", &r.team_login),
                        // std::format!("alert('foto_{}')", &team.login),
                        // document.getElementById('a').style.backgroundColor = ''"
                    },                        
                ]
            }
        })
    ]
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
