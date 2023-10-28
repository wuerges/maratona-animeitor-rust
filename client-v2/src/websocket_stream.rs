use futures::{
    channel::mpsc::{self, UnboundedReceiver},
    SinkExt, StreamExt,
};
use gloo_net::websocket::{futures::WebSocket, Message, WebSocketError};
use gloo_timers::future::TimeoutFuture;
use leptos::leptos_dom::logging::{console_error, console_log, console_warn};
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

#[derive(Debug)]
enum Error {
    Serde(serde_json::Error),
    WebSocket(WebSocketError),
    EmptyMessage,
}

fn parse_message<M: for<'a> Deserialize<'a>>(
    ws_message: Option<Result<Message, WebSocketError>>,
) -> Result<M, Error> {
    let message = ws_message.ok_or(Error::EmptyMessage)?;
    let msg = message.map_err(Error::WebSocket)?;
    Ok(match &msg {
        Message::Text(txt) => serde_json::from_str(txt).map_err(Error::Serde)?,
        Message::Bytes(bytes) => serde_json::from_slice(&bytes).map_err(Error::Serde)?,
    })
}

pub fn create_websocket_stream<M: for<'a> Deserialize<'a> + Clone + 'static>(
    url: &str,
) -> UnboundedReceiver<M> {
    let url = url.to_string();
    let (mut tx, rx) = mpsc::unbounded::<M>();

    spawn_local(async move {
        loop {
            match WebSocket::open(&url) {
                Ok(ws) => {
                    console_log(&format!("ws connected: {url}"));
                    let (_, mut read) = ws.split();
                    loop {
                        match parse_message::<M>(read.next().await) {
                            Ok(next_timer) => {
                                if let Err(err) = tx.send(next_timer).await {
                                    console_error(&format!("unbounded channel timeout: {err:?}"));
                                }
                            }
                            Err(err) => {
                                console_error(&format!("parse failed: {err:?}"));
                                break;
                            }
                        }
                    }
                    console_warn(&format!("websocket closed: {url}"));
                }
                Err(err) => console_error(&format!("Websocket error: {:?}", err)),
            }
            console_log("Wait 5 seconds to reconnect.");
            TimeoutFuture::new(5_000).await;
        }
    });

    rx
}
