use data::TimerData;
use leptos::*;
use leptos_router::*;

use crate::{
    api::{create_config, create_timer},
    model::provide_contest,
    views::{contest::Contest, navigation::Navigation},
};

use super::timer::Timer;

trait IsNegative {
    fn is_negative(&self) -> bool;
}

impl IsNegative for (TimerData, TimerData) {
    fn is_negative(&self) -> bool {
        self.0.current_time < 0
    }
}

#[component]
pub fn Countdown() -> impl IntoView {
    let timer = create_timer();
    let (contest, panel_items) = provide_contest();

    let (contest_name, set_contest_name) = create_signal(None);
    let config_contest = create_local_resource(|| (), |()| create_config());

    let (get_sede, _set_sede) = create_signal(None);
    move || {
        // let (time_data, _) = timer.get();
        // let time = time_data.current_time;
        view! {
            <Router>
                <Show when=move || timer.get().is_negative()>
                    <Timer timer />
                </Show>
                <Show when=move || !timer.get().is_negative()>
                    <Navigation config_contest contest_name />
                </Show>
                <Routes>
                        <Route path="/" view= move || view!{
                            <Contest contest panel_items timer sede=get_sede />
                        } />
                        <Route path="/sedes/:id" view= move || view!{
                            <Contest contest panel_items timer sede=get_sede />
                        } />
                        <Route path="/*any" view=|| view! { <h1>"Not Found"</h1> }/>
                </Routes>
                // {
                //     if time < 0 {
                //         view! {
                //         }
                //         .into_view()
                //     } else {
                //         view! {
                //         }
                //         .into_view()
                //     }
                // }
            </Router>
        }
    }
}
