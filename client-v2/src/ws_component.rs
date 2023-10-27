use futures::StreamExt;
use gloo_net::websocket::{futures::WebSocket, Message, WebSocketError};
use gloo_timers::future::TimeoutFuture;
use leptos::{leptos_dom::logging::console_log, *};
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

fn parse_message<M: for<'a> Deserialize<'a>>(
    ws_message: Option<Result<Message, WebSocketError>>,
) -> Option<M> {
    ws_message.and_then(|msg| {
        msg.ok().and_then(|msg| match &msg {
            Message::Text(txt) => serde_json::from_str(txt).ok(),
            Message::Bytes(bytes) => serde_json::from_slice(&bytes).ok(),
        })
    })
}

pub fn create_websocket_signal<M: for<'a> Deserialize<'a> + Clone>(
    url: &str,
    initial: M,
) -> ReadSignal<M> {
    let (message, set_message) = create_signal(initial);

    let url = url.to_string();

    spawn_local(async move {
        loop {
            match WebSocket::open(&url) {
                Ok(ws) => {
                    let (_, mut read) = ws.split();
                    while let Some(next_timer) = parse_message::<M>(read.next().await) {
                        set_message.set(next_timer);
                    }
                    console_log("Timer websocket closed.");
                }
                Err(err) => console_log(&format!("Websocket error: {:?}", err)),
            }
            console_log("Wait 5 seconds to reconnect.");
            TimeoutFuture::new(5_000).await;
        }
    });

    message
}
