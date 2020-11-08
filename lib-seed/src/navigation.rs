// use maratona_animeitor_rust::data;
use maratona_animeitor_rust::configdata;
use maratona_animeitor_rust::config;
use seed::{prelude::*, *};

// use crate::requests::*;
// use crate::helpers::*;
// use crate::views;

extern crate rand;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {

    Model {
    }
}

struct Model {
}

enum Msg {
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
}

fn build_url_filter(sede : &configdata::Sede) -> String {
    let mut s = format!("/seed/everything.html?source={}", sede.parent_source);

    for flt in &sede.codes {
        s.push_str(format!("&filter={}", flt).as_str());
    }
    s
}

fn build_url(sede : &configdata::Sede) -> String {
    format!("/seed/everything.html?source={}", sede.name)
}

fn view(model: &Model) -> Node<Msg> {
    let base_url = config::contest().host;
    table![
        config::contest().sedes.iter().map( |sede| {
            tr![
                td![&sede.name],
                td![&sede.source],
                // td![&sede.codes],
                td![a![attrs!{At::Href=>build_url(&sede), "target"=>"principal"}, "Renumerado"]],
                td![a![attrs!{At::Href=>build_url_filter(&sede), "target"=>"principal"}, "Filtrado"]],
            ]
        })
    ]
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
