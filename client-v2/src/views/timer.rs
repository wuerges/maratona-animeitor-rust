use leptos::*;

use data::TimerData;

use crate::api::create_timer;

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
