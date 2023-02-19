use crate::helpers::get_url_parameter;
use data::configdata;
use seed::{prelude::*, *};

use crate::requests;

extern crate rand;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.perform_cmd(fetch_all());

    Model {
        contest: None,
        contest_name: get_url_parameter(&url, "contest"),
    }
}

async fn fetch_all() -> Msg {
    let c = requests::fetch_config().await;
    Msg::Fetched(c)
}

struct Model {
    contest: Option<configdata::ConfigContest>,
    contest_name: Option<String>,
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

fn build_url_filter(model: &Model, sede: &configdata::Sede) -> String {
    let mut search = vec![
        ("sede", vec![&sede.name]),
        ("filter", sede.codes.iter().collect()),
    ];
    if let Some(ref contest_name) = model.contest_name {
        search.push(("contest", vec![contest_name]));
    }
    Url::new()
        .add_path_part("everything2.html")
        .set_search(UrlSearch::new(search))
        .to_string()
}

fn view(model: &Model) -> Node<Msg> {
    match model.contest.as_ref() {
        None => div![],
        Some(contest) => {
            log!(model.contest_name);
            let sedes = contest
                .sedes
                .iter()
                .filter(|sede| model.contest_name.is_some() && model.contest_name == sede.contest);

            div![
                C!["sedesnavigation"],
                sedes.map(|sede| {
                    span![
                        C!["sedeslink"],
                        a![
                            attrs! {At::Href=>build_url_filter(model, sede), At::Target=>"principal"},
                            &sede.name
                        ],
                    ]
                }),
            ]
        }
    }
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
