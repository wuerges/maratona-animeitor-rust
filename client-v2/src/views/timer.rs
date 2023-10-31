use itertools::Itertools;
use leptos::*;

use data::TimerData;

use crate::api::create_timer;

fn f(n: i64) -> String {
    format!("{:0>2}", n)
}

fn seg(n: i64) -> i64 {
    n % 60
}
fn min(n: i64) -> i64 {
    (n / 60) % 60
}
fn hor(n: i64) -> i64 {
    n / 60 / 60
}
fn changed(a: i64, b: i64) -> &'static str {
    if a == b {
        "same"
    } else {
        "changed"
    }
}

#[component]
pub fn Timer() -> impl IntoView {
    let timer = create_timer();

    move || {
        let (time_data, ptimer_data) = timer.get();
        let time = time_data.current_time;
        let ptime = ptimer_data.current_time;
        let frozen = time_data.is_frozen().then_some("frozen");
        view! {
            <div class={Some("timer").into_iter().chain(frozen).join(" ")}>
                <span class={["hora", changed(hor(time), hor(ptime))].join(" ")}>{ hor(time_data.current_time)} </span>
                <span class="sep"> ":" </span>
                <span class={["minuto", changed(min(time), min(ptime))].join(" ")}>{ f(min(time_data.current_time))} </span>
                <span class="sep"> ":" </span>
                <span class={["segundo", changed(seg(time), seg(ptime))].join(" ")}>{ f(seg(time_data.current_time))} </span>
            </div>
        }
    }
}
