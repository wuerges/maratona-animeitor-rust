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

fn build_url_filter(base_url: &str, sede : &configdata::Sede) -> String {
    let mut s = format!("{}/seed/everything.html?source={}", base_url, sede.parent_source);
    
    for flt in &sede.codes {
        s.push_str(format!("&filter={}", flt).as_str());
    }
    s
}

fn build_url(base_url: &str, sede : &configdata::Sede) -> String {

    Url::new()
    .add_path_part("seed")
    .add_path_part("everything.html")
    .set_search(UrlSearch::new(vec![
        ("source", vec![&sede.source]),
        ("sede", vec![&sede.name]),
    ])).to_string()


    // format!("{}/seed/everything.html?source={}&sede={}", base_url, sede.source, sede.name)
}

fn view(_: &Model) -> Node<Msg> {
    let base_url = config::contest().host;
    table![
        config::contest().sedes.iter().map( |sede| {
            tr![
                td![&sede.name],
                td![a![attrs!{At::Href=>build_url(&base_url, &sede), At::Target=>"principal"}, "Renumerado"]],
                td![a![attrs!{At::Href=>build_url_filter(&base_url, &sede), At::Target=>"principal"}, "Filtrado"]],
            ]
        })
    ]
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
