use leptos::*;
use leptos_router::*;

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

    let (contest_name, set_contest_name) = create_signal(None);
    let config_contest = create_resource(|| (), |()| create_config());

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
                // <Router>
                //     <Route path="/" view= || view!{
                        <Contest contest panel_items timer sede=get_sede />
                //     } />
                //     // <Route path="/sedes/:sede_name" view=UserProfile/>
                //     <Route path="/*any" view=|| view! { <h1>"Not Found"</h1> }/>
                // </Router>
            }
            .into_view()
        }
    }
}
