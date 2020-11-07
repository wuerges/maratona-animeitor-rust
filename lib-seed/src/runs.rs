use maratona_animeitor_rust::data;
use seed::{prelude::*, *};
use crate::views;
use crate::requests::*;

extern crate rand;


fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.skip().send_msg(Msg::Reset);
    orders.stream(streams::interval(30_000, || Msg::Reset));
    Model { 
        url_filter : url.hash().map( |s| s.clone()),
        runs: Vec::new(),
    }
}

struct Model {
    url_filter : Option<String>,
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

            model.runs = runs.into_iter().filter( |r| views::check_filter_login(&model.url_filter, &r.team_login)).collect();
            
            // match &model.url_filter {
            //     None => runs,
            //     Some(f) => runs.into_iter().filter( |r| check_filter_login(r.team_login.find(f).is_some() ).collect()
            // };
            model.runs.truncate(30);
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
        data::Answer::Yes(_)  => "answeryes",
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
