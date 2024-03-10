use leptos::{component, create_resource, create_signal, view, IntoSignal, IntoView, SignalGet};

use crate::{
    api::{create_config, create_timer},
    model::provide_contest,
    views::{contest::Contest, navigation::Navigation},
};

use super::timer::Timer;

#[component]
pub fn Countdown() -> impl IntoView {
    let timer = create_timer();
    let (contest, panel_items) = provide_contest();
    let config_contest = create_resource(|| (), |_| create_config()).into_signal();

    let (contest_name, set_contest_name) = create_signal(None);

    move || {
        let (time_data, _) = timer.get();
        let time = time_data.current_time;
        if time < 0 {
            view! {
                <Timer timer />
            }
            .into_view()
        } else {
            let (get_sede, _set_sede) = create_signal(None);
            view! {
                <Navigation config_contest contest_name />
                <Contest contest panel_items timer sede=get_sede />
            }
            .into_view()
        }
    }
}
