use crate::{requests, views};
use data;
use seed::{prelude::*, *};
use serde_json;

fn open_websocket(orders: &mut impl Orders<Msg>) -> WebSocket {
    log("connecting...");
    WebSocket::builder(requests::get_ws_url("/timer"), orders)
        .on_message(Msg::TimerUpdate)
        .on_open(Msg::Open)
        .on_close(Msg::Close)
        .build_and_open()
        .expect("Open WebSocket")
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    Model {
        p_timer_data: data::TimerData::new(0, 1),
        timer_data: data::TimerData::fake(),
        socket: open_websocket(orders),
    }
}

struct Model {
    p_timer_data: data::TimerData,
    timer_data: data::TimerData,
    socket: WebSocket,
}

enum Msg {
    TimerUpdate(WebSocketMessage),
    Open(),
    Close(CloseEvent),
    Reconnect,
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::TimerUpdate(m) => {
            // Gets the text of the websocket message
            let text = m.text().expect("text");

            // Gets the JsValue: WORKS!
            let js_value : JsValue = JsValue::from_str(&text);
            log!(js_value);

            // Gets the data::TimerData directly from the text: WORKS!
            let timer_serde : data::TimerData = serde_json::from_str(&text).expect("timer data");
            log!(timer_serde);

            // Gets the data::TimerData from the JsValue: BROKEN =/
            let timer_into_serde : data::TimerData = js_value.into_serde().expect("timer data");
            // let timer_into_serde : data::TimerData = m.json().expect("timer data");
            log!(timer_into_serde);

            model.p_timer_data = model.timer_data;
            // model.timer_data = m.json().into_serde().expect("Message should have TimerData");

            if model.timer_data == model.p_timer_data {
                orders.skip();
            }
        }
        Msg::Open() => {
            log("... connected!");
        }
        Msg::Close(e) => {
            log(e);
            orders.perform_cmd(cmds::timeout(5000, || Msg::Reconnect));
        }
        Msg::Reconnect => {
            model.socket = open_websocket(orders);
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    views::view_clock(model.timer_data, model.p_timer_data)
}

pub fn start(e: impl GetElement) {
    App::start(e, init, update, view);
}
