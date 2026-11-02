use futures::{FutureExt, Sink, SinkExt, StreamExt};
use gloo_net::websocket::{Message, WebSocketError, futures::WebSocket};
use log::{error, info, warn};
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
) -> crate::Subscription<M> {
    let url = url.to_string();
    let (mut tx, rx, canceled) = crate::subscription::channel::<M>();

    spawn_local(async move {
        let work = async move {
            let mut retry = crate::retry::Retry::default();
            loop {
                let mut connected_at = None;
                match WebSocket::open(&url) {
                    Ok(mut ws) => {
                        futures::future::poll_fn(|cx| std::pin::Pin::new(&mut ws).poll_ready(cx))
                            .await
                            .ok();
                        if matches!(ws.state(), gloo_net::websocket::State::Open) {
                            connected_at = Some(crate::retry::now_ms());
                        }
                        info!("websocket attempt ready");
                        // The socket stays whole: dropping a split half would
                        // close the connection under us.
                        loop {
                            match parse_message::<M>(ws.next().await) {
                                Ok(next_timer) => {
                                    if let Err(err) = tx.send(next_timer).await {
                                        error!("stream consumer disconnected: {err:?}");
                                        return;
                                    }
                                }
                                Err(err) => {
                                    match err {
                                        Error::Serde(err) => {
                                            error!("failed parsing response: {err:?}")
                                        }
                                        Error::WebSocket(_err) => {
                                            error!("websocket transport error")
                                        }
                                        Error::EmptyMessage => error!("empty message"),
                                    }
                                    break;
                                }
                            }
                        }
                        warn!("websocket closed");
                    }
                    Err(_) => error!("websocket open failed"),
                }
                if let Some(at) = connected_at {
                    retry.stable_for(crate::retry::now_ms() - at);
                }
                retry.wait().await;
            }
        };
        futures::select! { _ = work.fuse() => {}, _ = canceled.fuse() => {} }
    });

    rx
}

#[cfg(all(test, target_arch = "wasm32"))]
mod browser_tests {
    use super::*;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);
    #[wasm_bindgen(
        inline_js = "export function install_socket_mock() { window.socket_opens = 0; window.socket_closes = 0; window.original_socket = window.WebSocket; window.WebSocket = class extends EventTarget { constructor() { super(); this.readyState = 0; window.socket_opens++; setTimeout(() => { this.readyState = 1; this.dispatchEvent(new Event('open')); }, 0); } close() { if (this.readyState !== 3) window.socket_closes++; this.readyState = 3; } }; } export function counts() { return [window.socket_opens, window.socket_closes]; } export function restore_socket() { window.WebSocket = window.original_socket; }"
    )]
    extern "C" {
        fn install_socket_mock();
        fn counts() -> js_sys::Array;
        fn restore_socket();
    }
    #[wasm_bindgen_test(async)]
    async fn dropping_idle_consumer_closes_socket_without_reconnecting() {
        install_socket_mock();
        let stream = create_websocket_stream::<serde_json::Value>("ws://localhost/idle");
        gloo_timers::future::TimeoutFuture::new(30).await;
        drop(stream);
        gloo_timers::future::TimeoutFuture::new(1100).await;
        let values = counts();
        restore_socket();
        assert_eq!(values.get(0).as_f64(), Some(1.0));
        assert_eq!(values.get(1).as_f64(), Some(1.0));
    }
}
