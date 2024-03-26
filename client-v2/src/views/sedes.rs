use data::{
    configdata::{ConfigContest, Sede},
    ContestFile, RunsPanelItem, TimerData,
};
use leptos::{logging::*, *};
use leptos_router::*;

use crate::{
    api::{create_config, create_timer},
    model::provide_contest,
    views::{contest::Contest, navigation::Navigation},
};

use super::{reveleitor::Reveleitor, timer::Timer};

trait IsNegative {
    fn is_negative(&self) -> bool;
}

impl IsNegative for (TimerData, TimerData) {
    fn is_negative(&self) -> bool {
        self.0.current_time < 0
    }
}

#[derive(Params, PartialEq, Eq, Clone, Debug)]
struct LocalParams {
    sede: Option<String>,
    secret: Option<String>,
}

fn use_local_params() -> Option<LocalParams> {
    let params = use_query::<LocalParams>()
        .get()
        .inspect_err(|e| log!("{}", e))
        .ok()?;
    Some(params)
}

fn use_configured_sede(config: ConfigContest) -> Option<Sede> {
    let config = config.into_contest();
    let name = use_local_params()?.sede?;
    let sede = config.get_sede_nome_sede(&name)?;
    Some(sede.clone())
}

#[component]
fn ProvideSede(
    contest: Signal<ContestFile>,
    panel_items: ReadSignal<Vec<RunsPanelItem>>,
    config_contest: Resource<(), ConfigContest>,
    timer: ReadSignal<(TimerData, TimerData)>,
) -> impl IntoView {
    move || {
        config_contest.get().map(|config| {
            let sede = use_configured_sede(config);
            match sede {
                Some(sede) => view! { <Contest contest panel_items timer sede /> }.into_view(),
                None => view! { <p> Failed to match site </p> }.into_view(),
            }
        })
    }
}

#[component]
fn ConfiguredReveleitor(
    config_contest: Resource<(), ConfigContest>,
    secret: String,
) -> impl IntoView {
    view! {
        <Suspense fallback=|| view! { <p> Preparing reveleitor... </p> }>
        {
            let secret = secret.clone();
            move || {
                let secret = secret.clone();
                config_contest.get().map(move |config| {
                    let sede = use_configured_sede(config);
                    match sede {
                        Some(sede) => view! { <Reveleitor sede secret /> }.into_view(),
                        None => view! { <p> Failed to match site </p> }.into_view(),
                    }
                })
            }}
        </Suspense>
    }
}

#[component]
pub fn Sedes() -> impl IntoView {
    let timer = create_timer();
    let contest_and_panel = create_local_resource(|| (), |()| provide_contest());
    let config_contest = create_local_resource(|| (), |()| create_config());

    let negative_memo = create_memo(move |_| timer.get().is_negative());

    let root = move || {
        if negative_memo.get() {
            view! { <Timer timer /> }.into_view()
        } else {
            match use_local_params() {
                None => {
                    error!("failed loading params");
                    view! {<p> Failed loading params </p> }.into_view()
                }
                Some(params) => {
                    log!("loaded params: {:?}", params);
                    match params.secret {
                    Some(secret) => view! { <ConfiguredReveleitor config_contest secret/> }.into_view(),
                    None => view! {
                        <Navigation config_contest />
                        <Suspense fallback=|| view! { <p> Loading contest... </p> }>
                            {move || contest_and_panel.get().map(|(contest, panel_items)| view!{ <ProvideSede contest panel_items timer config_contest /> })}
                        </Suspense>
                    }.into_view(),
                }
                }
            }
        }
    };

    view! {
        <Router>
            <Routes>
                <Route path="*any" view=move || root />
            </Routes>
        </Router>
    }
}
