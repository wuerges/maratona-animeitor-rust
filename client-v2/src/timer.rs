use futures::StreamExt;
use gloo_net::websocket::{futures::WebSocket, Message};
use gloo_timers::future::TimeoutFuture;
use leptos::{leptos_dom::logging::console_log, *};
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

#[derive(Deserialize, Clone, PartialEq, Eq)]
struct Timer {
    pub current_time: i32,
    pub score_freeze_time: i32,
}

impl Default for Timer {
    fn default() -> Self {
        let max = 60 * 60 * 60;
        Self {
            current_time: max - 1,
            score_freeze_time: max,
        }
    }
}

fn parse_timer<E: std::fmt::Debug>(ws_message: Option<Result<Message, E>>) -> Option<Timer> {
    ws_message.and_then(|msg| {
        msg.ok().and_then(|msg| match &msg {
            Message::Text(txt) => serde_json::from_str(txt).ok(),
            Message::Bytes(_) => None,
        })
    })
}

fn create_timer() -> ReadSignal<Timer> {
    let (timer, set_timer) = create_signal(Timer::default());

    spawn_local(async move {
        loop {
            match WebSocket::open("ws://localhost:9000/api/timer") {
                Ok(ws) => {
                    let (_, mut read) = ws.split();
                    let mut prev = None;
                    while let Some(next_timer) = parse_timer(read.next().await) {
                        if !prev.as_ref().is_some_and(|p| p == &next_timer) {
                            prev = Some(next_timer.clone());
                            set_timer.set(next_timer);
                        }
                    }
                    console_log("Timer websocket closed.");
                }
                Err(err) => console_log(&format!("Websocket error: {:?}", err)),
            }
            console_log("Wait 5 seconds to reconnect.");
            TimeoutFuture::new(5_000).await;
        }
    });

    timer
}

#[component]
pub fn Timer() -> impl IntoView {
    let timer = create_timer();

    let current_time = move || timer.get().current_time;
    let score_freeze_time = move || timer.get().score_freeze_time * 60;

    view! {
        <p> Time?: "`"{current_time}/{score_freeze_time}"'" </p>
    }
}
