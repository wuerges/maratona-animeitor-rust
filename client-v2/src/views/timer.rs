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

    let (ptime, set_ptime) = create_signal::<TimerData>(TimerData::fake());

    // div![
    //     C!["timer"],
    //     frozen,
    //     span![C!["hora", changed(hor(time), hor(ptime))], hor(time)],
    //     span![C!["sep"], ":"],
    //     span![C!["minuto", changed(min(time), min(ptime))], f(min(time))],
    //     span![C!["sep"], ":"],
    //     span![C!["segundo", changed(seg(time), seg(ptime))], f(seg(time))],
    // ]

    move || {
        let time_data = timer.get();
        let frozen = time_data.is_frozen().then_some("frozen");
        view! {
            <div class={Some("timer").into_iter().chain(frozen).join(" ")}>
                <span class="hora">{ hor(time_data.current_time)} </span>
                <span class="sep"> ":" </span>
                <span class="minuto">{ f(min(time_data.current_time))} </span>
                <span class="sep"> ":" </span>
                <span class="segundo">{ f(seg(time_data.current_time))} </span>
            </div>
        }
    }

    // move || match timer.get() {
    //     Some(time_data) => {
    //         let time = time_data.current_time;
    //         let ptime = ptime_data.current_time;

    //         let frozen = if time_data.is_frozen() {
    //             Some("frozen")
    //         } else {
    //             None
    //         };

    //         view! {

    //         <p> Time?: "`"{current_time}/{score_freeze_time * 60}"'" </p>
    //     }},
    //     None => view! {<p> Timer is none =/ </p>},
    // }
}
