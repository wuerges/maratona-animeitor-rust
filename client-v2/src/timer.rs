use leptos::*;
use serde::Deserialize;

use crate::ws_component::create_websocket_signal;

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

fn create_timer() -> ReadSignal<Timer> {
    let timer_message = create_websocket_signal("ws://localhost:9000/api/timer", Timer::default());

    let (timer, set_timer) = create_signal(Timer::default());

    create_effect(move |_| {
        let next = timer_message.get();
        if next != timer.get() {
            set_timer.set(next);
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
