use leptos::*;

use data::TimerData;

use crate::websocket_signal::create_websocket_signal;

fn create_timer() -> ReadSignal<Option<TimerData>> {
    let timer_message = create_websocket_signal("ws://localhost:9000/api/timer", None);

    let (timer, set_timer) = create_signal(None);

    create_effect(move |_| {
        let next = timer_message.get();

        if next.is_some() && next != timer.get() {
            set_timer.set(next);
        }
    });

    timer
}

#[component]
pub fn Timer() -> impl IntoView {
    let timer = create_timer();

    move || match timer.get() {
        Some(TimerData {
            current_time,
            score_freeze_time,
        }) => view! {
            <p> Time?: "`"{current_time}/{score_freeze_time * 60}"'" </p>
        },
        None => view! {<p> Timer is none =/ </p>},
    }
}
