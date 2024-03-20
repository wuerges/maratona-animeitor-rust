use data::{configdata::ConfigContest, ContestFile, RunsPanelItem, TimerData};
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

#[derive(Params, PartialEq, Eq, Clone)]
struct SedeParam {
    name: Option<String>,
}

#[component]
fn ProvideSede(
    contest: ReadSignal<Option<ContestFile>>,
    panel_items: ReadSignal<Vec<RunsPanelItem>>,
    config_contest: Resource<(), ConfigContest>,
    timer: ReadSignal<(TimerData, TimerData)>,
) -> impl IntoView {
    move || {
        let params = use_params::<SedeParam>();
        let sede = params.get().unwrap().name;
        let config_contest = config_contest.get();

        match (sede, config_contest) {
            (Some(sede), Some(config)) => {
                let config = config.into_contest();
                let sede = config.get_sede_nome_sede(&sede).cloned();

                view! {  <Contest contest panel_items timer sede /> }.into_view()
            }
            (sede, None) => view! { <p> sede={sede}, config=None </p> }.into_view(),
            (sede, Some(_)) => view! { <p> sede={sede}, config=Some(config) </p> }.into_view(),
        }
    }
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
                        <Contest contest panel_items timer sede=None />
                    }/>
                    <Route path="/:name" view=move || view!{
                        <ProvideSede contest panel_items timer config_contest />
                    } />
                    <Route path="/*any" view=|| view! { <h1>"Not Found"</h1> }/>
            </Routes>
        </Router>
    }
}
