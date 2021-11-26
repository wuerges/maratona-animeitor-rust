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

static FAKE: &str = "this.onerror=''; this.src='/static/assets/teams/fake.webp';";

fn view(model: &Model) -> Node<Msg> {
    match model.contest.as_ref() {
        None => div![
            span!["Failed to load contest config"],
            ],
        Some(contest) => div![
            contest.teams.iter().map(|team_entry| {
                let foto_id = format!("foto_{}", &team_entry.login);
                div![C!["foto"], id![foto_id],
                attrs!{At::OnClick =>
                    std::format!("document.getElementById('foto_{}').style.display = 'none';",
                    &team_entry.login)
                },
                div![C!["nomeTime"], "&team_entry.nome"],
                &team_entry.foto.as_ref().map(|f|
                    img![C!["foto_img"],
                    attrs!{At::Src => std::format!("/static/assets/teams/{}", f)},
                    attrs!{At::OnError => FAKE}
                    ]
                ),
                &team_entry.musica.as_ref().map(|m|
                    p![a![C!["musica"], attrs!{At::Href => m}, "Música do Time"]]
                ),
                &team_entry.comentario.as_ref().map(|c|
                    p![C!["comentario"], c]
                ),
    ]
            }),
        ],
    }
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
