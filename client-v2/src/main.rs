use futures::StreamExt;
use gloo_net::websocket::{futures::WebSocket, Message};
use leptos::{leptos_dom::logging::console_log, *};
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;

#[derive(Deserialize, Clone)]
struct Timer {
    current_time: i32,
    score_freeze_time: i32,
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

fn create_timer() -> ReadSignal<Timer> {
    let ws = WebSocket::open("ws://localhost:9000/api/timer").unwrap();
    let (_, mut read) = ws.split();
    let (timer, set_timer) = create_signal(Timer::default());

    spawn_local(async move {
        while let Some(msg) = read.next().await {
            console_log(&format!("1. {:?}", msg));

            if let Ok(msg) = &msg {
                if let Message::Text(txt) = msg {
                    set_timer.set(serde_json::from_str(txt).unwrap());
                }
            }
        }
        console_log("WebSocket Closed")
    });

    timer
}

#[component]
fn Timer() -> impl IntoView {
    let timer = create_timer();

    let current_time = move || timer.get().current_time;
    let score_freeze_time = move || timer.get().score_freeze_time * 60;

    view! {
        <p> Time?: "`"{current_time}/{score_freeze_time}"'" </p>
    }
}

fn main() {
    mount_to_body(|| view! { <Timer/> })
}
