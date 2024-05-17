use std::rc::Rc;

use data::{
    configdata::{ConfigContest, Sede},
    ContestFile, RunsPanelItem, TimerData,
};
use leptos::{logging::*, *};
use leptos_router::*;

use crate::{
    api::{create_config, create_timer},
    model::{provide_contest, ContestProvider},
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
    let sede = Rc::new(use_configured_sede(config_contest, sede_param));
    view! { <Contest contest panel_items timer sede /> }
}

#[component]
fn ConfiguredReveleitor(
    contest_provider: Resource<(), ContestProvider>,
    secret: String,
    sede_param: Option<String>,
) -> impl IntoView {
    let configured_reveleitor = move || {
        {
            let secret = secret.clone();
            let sede_param = sede_param.clone();
            contest_provider.with(|provider| {
                provider.as_ref().map(|provider| {
                    let sede = Rc::new(use_configured_sede(
                        provider.config_contest.clone(),
                        sede_param,
                    ));
                    view! { <Reveleitor sede secret contest=provider.starting_contest.clone() /> }
                })
            })
        }
        .into_view()
    };
    view! {
        <Suspense fallback=|| view! { <p> Preparing reveleitor... </p> }>
            {configured_reveleitor()}
        </Suspense>
    }
}

#[component]
pub fn Sedes() -> impl IntoView {
    let timer = create_timer();
    let contest_provider = create_local_resource(|| (), |()| provide_contest());
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
                            <ConfiguredReveleitor contest_provider secret=secret sede_param=params.sede.clone() />
                        }.into_view(),
                        None => view! {
                            <Navigation config_contest />
                            <Suspense fallback=|| view! { <p> Loading contest... </p> }>
                                {
                                    let sede = params.sede.clone();
                                    move || contest_provider.with(|contest_provider|
                                        contest_provider.as_ref().map(|provider|
                                            view!{
                                                <ProvideSede
                                                    contest=provider.running_contest
                                                    panel_items=provider.panel_items
                                                    timer
                                                    config_contest=provider.config_contest.clone()
                                                    sede_param=sede.clone()
                                                />
                                            }
                                        )
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
