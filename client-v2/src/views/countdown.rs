use data::{ContestFile, RunsPanelItem, TimerData};
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
fn ProvideSede(
    contest: ReadSignal<Option<ContestFile>>,
    panel_items: ReadSignal<Vec<RunsPanelItem>>,
    timer: ReadSignal<(TimerData, TimerData)>,
) -> impl IntoView {
    let params = use_params_map();

    let (get_sede, _set_sede) = create_signal(None);
    let id = move || params.with(|p| p.get("sede").cloned().unwrap_or_default());

    view! {  <Contest contest panel_items timer sede=get_sede /> }
}

#[component]
pub fn Countdown() -> impl IntoView {
    let timer = create_timer();
    let (contest, panel_items) = provide_contest();
    let config_contest = create_local_resource(|| (), |()| create_config());
    let (contest_name, set_contest_name) = create_signal(None);

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
                        <h1> Root </h1>
                    }/>
                    <Route path="/sedes" view= move || view!{
                        <ProvideSede contest panel_items timer />
                    } >
                        <Route path=":id" view= move || view!{
                            <ProvideSede contest panel_items timer />
                        } />
                    </Route>
                    <Route path="/*any" view=|| view! { <h1>"Not Found"</h1> }/>
            </Routes>
        </Router>
    }
}
