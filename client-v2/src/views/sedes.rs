use std::rc::Rc;

use data::{
    configdata::{ConfigContest, Sede},
    ContestFile, RunsPanelItem, TimerData,
};
use leptos::{logging::*, *};
use leptos_router::*;

use crate::{
    api::{create_config, create_timer, ContestQuery},
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

#[derive(Params, PartialEq, Eq, Clone, Debug, Default)]
struct SedeQuery {
    sede: Option<String>,
}

#[derive(Params, PartialEq, Eq, Clone, Debug, Default)]
struct SecretQuery {
    secret: Option<String>,
}

fn use_secret_query() -> Option<SecretQuery> {
    use_query::<SecretQuery>()
        .get()
        .inspect_err(|e| error!("incorrect secret: {:?}", e))
        .ok()
}

fn use_sede_query() -> Signal<SedeQuery> {
    (|| use_query::<SedeQuery>().get().unwrap_or_default()).into_signal()
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
    contest_provider: Resource<ContestQuery, ContestProvider>,
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

    let negative_memo = create_memo(move |_| timer.get().is_negative());

    let root = move || {
        if negative_memo.get() {
            view! { <Timer timer /> }.into_view()
        } else {
            let query = (|| use_query::<ContestQuery>().get().unwrap_or_default()).into_signal();
            let sede_query = use_sede_query();
            let secret_query = use_secret_query();
            let contest_provider =
                create_local_resource(move || query.get(), |q| provide_contest(q));
            let config_contest = create_local_resource(move || query.get(), |q| create_config(q));

            (move || match secret_query.clone() {
                None => {
                    error!("failed loading params");
                    view! {<p> Failed loading params </p> }.into_view()
                }
                Some(params) => {
                    log!("loaded params: {:?}", params);

                    match params.clone().secret {
                        Some(secret) => view! {
                            <ConfiguredReveleitor contest_provider secret=secret sede_param=sede_query.get().sede />
                        }.into_view(),
                        None => view! {
                            <Navigation config_contest query />
                            <Suspense fallback=|| view! { <p> Loading contest... </p> }>
                                {
                                    move || contest_provider.with(|contest_provider|
                                        contest_provider.as_ref().map(|provider|
                                            view!{
                                                <ProvideSede
                                                    contest=provider.running_contest
                                                    panel_items=provider.panel_items
                                                    timer
                                                    config_contest=provider.config_contest.clone()
                                                    sede_param=sede_query.get().sede
                                                />
                                            }
                                        )
                                    )
                                }
                            </Suspense>
                        }.into_view(),
                    }
                }
            })
            .into_view()
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
