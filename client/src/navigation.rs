use data::configdata;
use seed::{prelude::*, *};

use crate::requests;

extern crate rand;

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.perform_cmd(fetch_all());

    Model { contest: None }
}

async fn fetch_all() -> Msg {
    let c = requests::fetch_config().await;
    Msg::Fetched(c)
}

struct Model {
    contest: Option<configdata::ConfigContest>,
}

enum Msg {
    Fetched(fetch::Result<configdata::ConfigContest>),
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::Fetched(Ok(config)) => {
            model.contest = Some(config);
        }
        Msg::Fetched(Err(e)) => {
            log!("Error: failed loading config", e);
        }
    }
}

fn build_url_filter(sede: &configdata::Sede) -> String {
    Url::new()
        .add_path_part("everything2.html")
        .set_search(UrlSearch::new(vec![
            ("sede", vec![&sede.name]),
            ("filter", sede.codes.iter().map(|s| s).collect()),
        ]))
        .to_string()
}

fn view(model: &Model) -> Node<Msg> {
    match model.contest.as_ref() {
        None => div![],
        Some(contest) => div![
            C!["sedesnavigation"],
            contest.sedes.iter().map(|sede| {
                span![
                    C!["sedeslink"],
                    a![
                        attrs! {At::Href=>build_url_filter(&sede), At::Target=>"principal"},
                        &sede.name
                    ],
                ]
            }),
        ],
    }
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
