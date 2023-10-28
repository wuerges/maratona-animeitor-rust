use leptos::*;

use data::TimerData;

use crate::websocket_stream::create_websocket_stream;

fn create_timer() -> ReadSignal<Option<TimerData>> {
    let timer_stream = create_websocket_stream("ws://localhost:9000/api/timer");
    let timer_message = create_signal_from_stream(timer_stream);

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
