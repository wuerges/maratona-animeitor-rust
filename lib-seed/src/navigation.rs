// use maratona_animeitor_rust::data;
use maratona_animeitor_rust::configdata;
use maratona_animeitor_rust::config;
use seed::{prelude::*, *};

// use crate::requests::*;
// use crate::helpers::*;
// use crate::views;

extern crate rand;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
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

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            // log!("url requested!");
            url.go_and_load();
            // orders.skip().request_url(url);
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
    format!("{}/seed/everything.html?source={}", base_url, sede.source)
}

fn link(target: &str, locat: &str) -> String {
    format!("<a href=\"{}\" target=\"{}\"> aoeuaoe </a>", locat, target)
}

fn view(model: &Model) -> Node<Msg> {
    let base_url = config::contest().host;
    table![
        config::contest().sedes.iter().map( |sede| {
            tr![
                td![&sede.name],
                // td![&sede.source],
                // td![&sede.codes],
                // onclick="window.open('../html-link.htm','name','width=600,height=400')">
                td![a![attrs!{At::Href=>build_url(&base_url, &sede), At::Target=>"principal"}, "Renumerado"]],
                td![a![attrs!{At::Href=>build_url_filter(&base_url, &sede), At::Target=>"principal"}, "Filtrado"]],
            ]
        })
    ]
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
