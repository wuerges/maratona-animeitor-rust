use crate::{helpers, views};
use data;
use seed::{prelude::*, *};

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    let ws = WebSocket::builder(helpers::get_ws_url("/timer"), orders)
        .on_message(Msg::TimerUpdate)
        .build_and_open()
        .expect("Open WebSocket");
    Model {
        p_timer_data: data::TimerData::new(0, 1),
        timer_data: data::TimerData::fake(),
        _socket: ws,
    }
}

struct Model {
    p_timer_data: data::TimerData,
    timer_data: data::TimerData,
    _socket: WebSocket,
}

enum Msg {
    TimerUpdate(WebSocketMessage),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::TimerUpdate(m) => {
            model.p_timer_data = model.timer_data;
            model.timer_data = m.json().expect("Message should have TimerData");

            if model.timer_data == model.p_timer_data {
                orders.skip();
            }
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    views::view_clock(model.timer_data, model.p_timer_data)
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
