use seed::{prelude::*, *};

use crate::requests::{self, team_photo_location};

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
    Reconnect,
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Fetched(Ok(config)) => {
            model.contest = Some(config);
        }
        Msg::Fetched(Err(e)) => {
            log!("Error: failed loading config, retrying in 5 seconds", e);
            orders.perform_cmd(cmds::timeout(5000, || Msg::Reconnect));
        }
        Msg::Reconnect => {
            orders.perform_cmd(fetch_all());
        }
    }
}

fn fake() -> String {
    format!(
        "this.onerror=''; this.src='{}';",
        team_photo_location("fake")
    )
}

fn view(model: &Model) -> Node<Msg> {
    match model.contest.as_ref() {
        None => div![span!["Contest is not ready yet!"],],
        Some(contest) => {
            div![id!["foto_container"],
            contest.teams.iter().map(|(team_login, team_entry)| {
                let foto_id = format!("foto_{}", team_login);
                div![C!["foto"], id![foto_id],
                    attrs!{At::OnClick =>
                        std::format!("document.getElementById('foto_{}').style.display = 'none';",
                        &team_entry.login)
                    },
                    // div![C!["nomeTime"], &team_entry.name],
                    img![C!["foto_img"],
                        attrs!{At::Src => team_photo_location(team_login)},
                        attrs!{At::OnError => fake()}
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
