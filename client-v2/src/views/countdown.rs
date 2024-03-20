use data::{
    configdata::{ConfigContest, Sede},
    ContestFile, RunsPanelItem, TimerData,
};
use leptos::{logging::log, *};
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

fn use_sede_param() -> Option<String> {
    let params = use_params::<SedeParam>()
        .get()
        .inspect_err(|e| log!("{}", e))
        .ok()?;
    params.name
}

fn use_configured_sede(config: ConfigContest) -> Option<Sede> {
    let config = config.into_contest();
    let name = use_sede_param()?;
    let sede = config.get_sede_nome_sede(&name)?;
    Some(sede.clone())
}

#[component]
fn ProvideSede(
    contest: ReadSignal<Option<ContestFile>>,
    panel_items: ReadSignal<Vec<RunsPanelItem>>,
    config_contest: Resource<(), ConfigContest>,
    timer: ReadSignal<(TimerData, TimerData)>,
) -> impl IntoView {
    move || {
        view! {
            <Suspense fallback=|| view! { <p> Loading config... </p> }>
                {move || {config_contest.get().map(|config| {
                    let sede = use_configured_sede(config);
                    match sede {
                        Some(sede) =>
                        view! {  <Contest contest panel_items timer sede /> }.into_view()
                        ,
                        None =>
                        view!{ <p> Failed to match site </p> }.into_view()
                        ,
                    }
                })}}
            </Suspense>
        }
    }
}

#[component]
pub fn Countdown() -> impl IntoView {
    let timer = create_timer();
    let (contest, panel_items) = provide_contest();
    let config_contest = create_local_resource(|| (), |()| create_config());

    view! {
        <Router>
            <Show when=move || timer.get().is_negative()>
                <Timer timer />
            </Show>
            <Show when=move || !timer.get().is_negative()>
                <Navigation config_contest />
            </Show>
            <Routes>
                    <Route path="/" view= move || view!{
                        <Contest contest panel_items timer />
                    }/>
                    <Route path="/:name" view=move || view!{
                        <ProvideSede contest panel_items timer config_contest />
                    } />
                    <Route path="/*any" view=|| view! { <h1>"Not Found"</h1> }/>
            </Routes>
        </Router>
    }
}
