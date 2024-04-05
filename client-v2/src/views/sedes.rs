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

fn use_configured_sede(config: ConfigContest, sede_param: Option<String>) -> Sede {
    let config = config.into_contest();
    sede_param
        .and_then(|sede| config.get_sede_nome_sede(&sede))
        .cloned()
        .unwrap_or(config.titulo)
}

#[component]
fn ProvideSede(
    contest: Signal<ContestFile>,
    panel_items: ReadSignal<Vec<RunsPanelItem>>,
    config_contest: ConfigContest,
    timer: ReadSignal<(TimerData, TimerData)>,
    sede_param: Option<String>,
) -> impl IntoView {
    let sede = Box::new(use_configured_sede(config_contest, sede_param));
    view! { <Contest contest panel_items timer sede /> }
}

#[component]
fn ConfiguredReveleitor(
    config_contest: Resource<
        (),
        (
            Signal<ContestFile>,
            ConfigContest,
            ReadSignal<Vec<RunsPanelItem>>,
        ),
    >,
    secret: String,
    sede_param: Option<String>,
) -> impl IntoView {
    view! {
        <Suspense fallback=|| view! { <p> Preparing reveleitor... </p> }>
        {
            let secret = secret.clone();
            let sede_param = sede_param.clone();
            move || {
                let sede_param = sede_param.clone();
                let secret = secret.clone();
                config_contest.get().map(move |(contest,config,_)| {
                    let sede = Box::new(use_configured_sede(config,sede_param));
                    view! { <Reveleitor sede secret contest /> }.into_view()
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
                        Some(secret) => view! {
                            <ConfiguredReveleitor config_contest=contest_and_panel secret=secret sede_param=params.sede.clone() />
                        }.into_view(),
                        None => view! {
                            <Navigation config_contest />
                            <Suspense fallback=|| view! { <p> Loading contest... </p> }>
                                {
                                    let sede = params.sede.clone();
                                    move || contest_and_panel.get().map(|(contest, config_contest, panel_items)|
                                       view!{ <ProvideSede contest panel_items timer config_contest sede_param=sede.clone() /> }
                                    )
                                }
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
