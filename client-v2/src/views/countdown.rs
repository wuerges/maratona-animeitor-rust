use leptos::{component, view, IntoView, SignalGet};

use crate::{api::create_timer, model::provide_contest, views::contest::Contest};

use super::timer::Timer;

#[component]
pub fn Countdown() -> impl IntoView {
    let timer = create_timer();
    let (contest, panel_items) = provide_contest();

    move || {
        let (time_data, _) = timer.get();
        let time = time_data.current_time;
        if time < 0 {
            view! {
                <Timer timer />
            }
        } else {
            view! {
                <Contest contest panel_items timer />
            }
        }
    }
}
