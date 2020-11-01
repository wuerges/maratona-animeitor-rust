use maratona_animeitor_rust::data;
use seed::{prelude::*, *};
use crate::views;
use crate::requests::*;

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.stream(streams::interval(1000, || Msg::Reset));
    Model { 
        p_time_file: 0,
        time_file: 86399,
        // time_file: 0,
    }
}

struct Model {
    p_time_file: data::TimeFile,
    time_file: data::TimeFile,
}

enum Msg {
    Reset,
    Fetched(fetch::Result<data::TimeFile>),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Fetched(Ok(runs)) => {
            // log!("fetched runs!", runs);
            model.p_time_file = model.time_file;
            model.time_file = runs;
        },
        Msg::Fetched(Err(e)) => {
            log!("fetched runs error!", e)
        },
        Msg::Reset => {
            orders.skip().perform_cmd( async { Msg::Fetched(fetch_time_file().await) } );    
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    views::view_clock(model.time_file, model.p_time_file)
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
