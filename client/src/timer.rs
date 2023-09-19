use crate::{requests, views};

use seed::{prelude::*, *};

fn open_websocket(orders: &mut impl Orders<Msg>) -> Result<WebSocket, WebSocketError> {
    log("connecting...");
    WebSocket::builder(requests::get_ws_url("/timer"), orders)
        .on_message(Msg::TimerUpdate)
        .on_open(Msg::Open)
        .on_close(Msg::Close)
        .on_error(Msg::Error)
        .build_and_open()
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.stream(streams::interval(1_000, || Msg::Reconnect));
    Model {
        p_timer_data: data::TimerData::new(0, 1),
        timer_data: data::TimerData::fake(),
        socket: None,
    }
}

struct Model {
    p_timer_data: data::TimerData,
    timer_data: data::TimerData,
    socket: Option<WebSocket>,
}

enum Msg {
    TimerUpdate(WebSocketMessage),
    Open(),
    Close(CloseEvent),
    Error(),
    Reconnect,
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::TimerUpdate(m) => {
            model.p_timer_data = model.timer_data;
            match m.json() {
                Ok(data) => model.timer_data = data,
                Err(error) => {
                    model.socket = None;
                    orders.skip();
                    log!("error parsing json", error);
                }
            }

            if model.timer_data == model.p_timer_data {
                orders.skip();
            }
        }
        Msg::Open() => {
            log("... connected!");
            orders.skip();
        }
        Msg::Close(e) => {
            log(e);
            model.socket = None;
            orders.skip();
        }
        Msg::Error() => {
            log("websocket disconnection error");
            model.socket = None;
            orders.skip();
        }
        Msg::Reconnect => {
            if model.socket.is_none() {
                match open_websocket(orders) {
                    Ok(connected) => model.socket = Some(connected),
                    Err(error) => log!("failed to reconnect", error),
                }
            }
            orders.skip();
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    views::view_clock(model.timer_data, model.p_timer_data)
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
