use maratona_animeitor_rust::data;
use seed::{prelude::*, *};
use crate::views;
use crate::requests::*;
use crate::helpers::*;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.stream(streams::interval(1000, || Msg::Reset));
    Model { 
        source : get_source(&url),
        p_timer_data: data::TimerData::new(0, 1),
        timer_data: data::TimerData::new(86399, 86399+1),
        // timer_data: 0,
    }
}

struct Model {
    source : Option<String>,
    p_timer_data: data::TimerData,
    timer_data: data::TimerData,
}

enum Msg {
    Reset,
    Fetched(fetch::Result<data::TimerData>),
}

async fn fetch_all(source : Option<String>) -> Msg {
    let f = fetch_time_file(&source).await;
    Msg::Fetched(f)
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Fetched(Ok(runs)) => {
            // log!("fetched runs!", runs);
            model.p_timer_data = model.timer_data;
            model.timer_data = runs;
        },
        Msg::Fetched(Err(e)) => {
            log!("fetched runs error!", e)
        },
        Msg::Reset => {
            orders.skip().perform_cmd( fetch_all(model.source.clone()) );    
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    views::view_clock(model.timer_data, model.p_timer_data)
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
