use seed::{prelude::*, *};
use crate::helpers;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);

    Model { 
        nome : helpers::get_sede(&url),
    }
}

struct Model {
    nome : Option<String>,
}

enum Msg {
    UrlChanged(subs::UrlChanged)
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            model.nome = helpers::get_sede(&url);
            // orders.skip().send_msg(Msg::Reload);
            // url.go_and_load();
        },
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![C!["nomesede"], &model.nome]
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
