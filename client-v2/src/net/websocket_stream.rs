use std::time::Duration;

use futures::{
    channel::mpsc::{self, SendError, UnboundedReceiver, UnboundedSender},
    SinkExt, Stream, StreamExt,
};
use gloo_net::websocket::{futures::WebSocket, Message, WebSocketError};
use gloo_timers::future::{sleep, TimeoutFuture};
use leptos::{
    leptos_dom::logging::{console_error, console_log, console_warn},
    logging::error,
};
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
        Message::Bytes(bytes) => serde_json::from_slice(bytes).map_err(Error::Serde)?,
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
                                match err {
                                    Error::Serde(err) => {
                                        console_error(&format!("failed parsing response: {err:?}"))
                                    }
                                    Error::WebSocket(err) => {
                                        console_error(&format!("websocket error: {err:?}"))
                                    }
                                    Error::EmptyMessage => console_error("empty message"),
                                }
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

async fn open_ws(url: &str) -> WebSocket {
    loop {
        match WebSocket::open(url) {
            Ok(websocket) => return websocket,
            Err(err) => {
                error!("could not open websocket: {err}");
                sleep(Duration::from_secs(5)).await
            }
        }
    }
}

async fn send_messages<M: for<'a> Deserialize<'a> + Clone + 'static>(
    ws: &mut WebSocket,
    mut tx: UnboundedSender<M>,
) -> Result<(), SendError> {
    loop {
        match parse_message::<M>(ws.next().await) {
            Ok(m) => tx.send(m).await?,
            Err(err) => {
                error!("parse message: {err:?}");
                return Ok(());
            }
        }
    }
}

pub fn create_websocket_stream_2<M: for<'a> Deserialize<'a> + Clone + 'static>(
    url: &str,
) -> impl Stream<Item = M> {
    let (tx, rx) = mpsc::unbounded::<M>();

    let url = url.to_string();

    spawn_local(async move {
        loop {
            let mut ws = open_ws(&url).await;
            console_log(&format!("connected: {url}"));

            match send_messages(&mut ws, tx.clone()).await {
                Ok(()) => (),
                Err(err) => {
                    error!("client disconnected: {err}");
                    break;
                }
            }
        }
    });

    rx
}
