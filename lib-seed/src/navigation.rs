// use maratona_animeitor_rust::data;
use maratona_animeitor_rust::configdata;
use maratona_animeitor_rust::config;
use seed::{prelude::*, *};

// use crate::requests::*;
// use crate::helpers::*;
// use crate::views;

extern crate rand;

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    // orders.subscribe(Msg::UrlChanged);
    orders.subscribe(Msg::UrlChanged);


    Model {
    }
}

struct Model {
}

enum Msg {
    UrlChanged(subs::UrlChanged)
}

fn update(msg: Msg, _: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            url.go_and_load();
        }
    }
}

fn build_url_filter(sede : &configdata::Sede) -> String {
    Url::new()
    .add_path_part("seed")
    .add_path_part("everything.html")
    .set_search(UrlSearch::new(vec![
        // ("source", vec![&sede.parent_source]),
        ("sede", vec![&sede.name]),
        ("filter", sede.codes.iter().map(|s| s).collect()),
    ])).to_string()
}

// fn build_url(sede : &configdata::Sede) -> String {
//     Url::new()
//     .add_path_part("seed")
//     .add_path_part("everything.html")
//     .set_search(UrlSearch::new(vec![
//         // ("source", vec![&sede.source]),
//         ("sede", vec![&sede.name]),
//     ])).to_string()
// }

fn view(_: &Model) -> Node<Msg> {
    table![
        config::contest().sedes.iter().map( |sede| {
            tr![
                td![&sede.name],
                // td![a![attrs!{At::Href=>build_url(&sede), At::Target=>"principal"}, "Renumerado"]],
                td![a![attrs!{At::Href=>build_url_filter(&sede), At::Target=>"principal"}, "Filtrado"]],
            ]
        })
    ]
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
