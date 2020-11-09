use seed::{prelude::*, *};

fn init(url: Url, _: &mut impl Orders<Msg>) -> Model {
    Model { 
        nome : url.search().get("sede").unwrap_or(&vec![]).iter().cloned().next(),
    }
}

struct Model {
    nome : Option<String>,
}

enum Msg {
}

fn update(_: Msg, _: &mut Model, _: &mut impl Orders<Msg>) {
}

fn view(model: &Model) -> Node<Msg> {
    div![C!["nomesede"], &model.nome]
}

pub fn start(e : impl GetElement) {
    App::start(e, init, update, view);
}
