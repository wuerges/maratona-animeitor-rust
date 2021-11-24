use seed::{prelude::*, *};

use crate::requests;

extern crate rand;

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.perform_cmd(fetch_all());

    Model { contest: None }
}

async fn fetch_all() -> Msg {
    let c = requests::fetch_contest().await;
    Msg::Fetched(c)
}

struct Model {
    contest: Option<data::ContestFile>,
}

enum Msg {
    Fetched(fetch::Result<data::ContestFile>),
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

static FAKE: &str = "this.onerror=''; this.src='/static/assets/teams/fake.webp';";

fn view(model: &Model) -> Node<Msg> {
    match model.contest.as_ref() {
        None => div![
            span!["Failed to load contest config"],
            ],
        Some(contest) => div![
            contest.teams.iter().map(|(team_login, team_entry)| {
                let foto_id = format!("foto_{}", team_login);
                div![C!["foto"], id![foto_id],
                    attrs!{At::OnClick =>  
                        std::format!("document.getElementById('foto_{}').style.display = 'none';", 
                        &team_entry.login)
                    },
                    div![C!["nomeTime"], &team_entry.name],
                        img![C!["foto_img"], 
                        attrs!{At::Src => std::format!("/static/assets/teams/{}.webp", team_login)},
                        attrs!{At::OnError => FAKE}
                    ],
                ]
            }),
        ],
    }
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
