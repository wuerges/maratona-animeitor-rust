use seed::{prelude::*, *};

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);

    Model { 
        nome : url.search().get("sede").unwrap_or(&vec![]).iter().cloned().next(),
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
            model.nome =
                url.search().get("sede").unwrap_or(&vec![]).iter().cloned().next();
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
