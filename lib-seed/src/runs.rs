use maratona_animeitor_rust::data;
use seed::{prelude::*, *};
use crate::views;
use crate::requests::*;

extern crate rand;


fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.stream(streams::interval(1000, || Msg::Reset));
    Model { 
        runs: Vec::new(),
    }
}

struct Model {
    runs: Vec<data::RunsPanelItem>,
}

enum Msg {
    Reset,
    Fetched(fetch::Result<Vec<data::RunsPanelItem>>),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Fetched(Ok(runs)) => {
            // log!("fetched runs!", runs);
            model.runs = runs;
        },
        Msg::Fetched(Err(e)) => {
            log!("fetched runs error!", e)
        },
        Msg::Reset => {
            orders.skip().perform_cmd( async { Msg::Fetched(fetch_runspanel().await) } );    
        }
    }
}

fn get_answer(t : &data::Answer) -> &str {
    match t {
        data::Answer::Yes  => "answeryes",
        data::Answer::No   => "answerno",
        data::Answer::Wait => "answerwait",
        _                  => "answererror"
    }
}  

fn view(model: &Model) -> Node<Msg> {
    div![
        C!["runstable"],
        model.runs.iter().enumerate().map({ |(i,r) |
        div![
            C!["run"],
            style!{ 
                St::Top => format!("calc(var(--row-height) * {} + var(--root-top))", i),
            },
            div![C!["cell", "colocacao", views::get_color(r.placement)], r.placement],
            div![
                C!["cell", "time"],
                div![C!["nomeEscola"], &r.escola],
                div![C!["nomeTime"], &r.team_name],
            ],
            div![C!["cell", "problema"], &r.problem],
            div![C!["cell", "resposta", get_answer(&r.result)]],
        ]
    })]
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
