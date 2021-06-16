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

fn view(model: &Model) -> Node<Msg> {
    match model.contest.as_ref() {
        None => div![
            span!["Failed to load contest config"],
            ],
        Some(contest) => div![
            span!["Contest Config:"],
            contest.teams.iter().map(|team_entry| {
                div![C!["foto"], id![format!("foto_{}", &team_entry.login)], &team_entry.nome,
                attrs!{At::OnClick => 
                    std::format!("document.getElementById('foto_{}').style.display = 'none';", &team_entry.login),
                    // std::format!("alert('foto_{}')", &team.login),
                    // document.getElementById('a').style.backgroundColor = ''"
                }]
            }),
        ],
    }
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
