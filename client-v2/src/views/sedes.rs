use std::rc::Rc;

use data::{
    configdata::{ConfigContest, Sede},
    ContestFile, TimerData,
};
use leptos::*;
use leptos_router::*;

use crate::{
    api::{create_config, create_timer, ContestQuery},
    model::{
        provide_contest, runs_panel_signal::RunsPanelItemManager, ContestProvider, ContestSignal,
    },
    views::{
        contest::Contest,
        control_scrolling::RemoteControl,
        global_settings::{use_global_settings, SettingsPanel},
        navigation::Navigation,
    },
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
struct QueryParams {
    sede: Option<String>,
    secret: Option<String>,
    settings: Option<bool>,
}

impl QueryParams {
    fn is_settings_enabled(&self) -> bool {
        self.settings.unwrap_or_default()
    }
}

fn use_static_query() -> Signal<QueryParams> {
    let query_params = use_query::<QueryParams>();
    (move || query_params.get().ok().unwrap_or_default()).into_signal()
}

fn use_configured_sede(
    config: ConfigContest,
    titulo: Rc<Sede>,
    sede_param: Option<String>,
) -> Rc<Sede> {
    let config = config.into_contest();
    let sub_sede = sede_param
        .and_then(|sede| config.get_sede_nome_sede(&sede))
        .cloned()
        .map(Rc::new);

    sub_sede.unwrap_or(titulo)
}

fn use_titulo(config: ConfigContest) -> Rc<Sede> {
    let config = config.into_contest();
    Rc::new(config.titulo)
}

#[component]
fn ProvideSede<'cs>(
    contest: Signal<ContestFile>,
    contest_signal: Rc<ContestSignal>,
    panel_items: &'cs RunsPanelItemManager,
    config_contest: ConfigContest,
    timer: ReadSignal<(TimerData, TimerData)>,
    sede_param: Signal<QueryParams>,
) -> impl IntoView {
    let titulo = use_titulo(config_contest.clone());
    let titulo_sede = titulo.clone();
    let sede = create_memo(move |_| {
        use_configured_sede(
            config_contest.clone(),
            titulo_sede.clone(),
            sede_param.get().sede,
        )
    });

    let titulo = move || {
        sede.with(|s| {
            if s.entry.name == titulo.entry.name {
                None
            } else {
                Some(titulo.clone())
            }
        })
    };

    view! { <Contest contest contest_signal panel_items timer titulo=titulo.into() sede=sede.into() /> }
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
                    let titulo = use_titulo(provider.config_contest.clone());
                    let sede =
                        use_configured_sede(provider.config_contest.clone(), titulo, sede_param);
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

    let global_settings = use_global_settings();

    let root = move || {
        let query_params = use_static_query();
        let settings_panel = move || {
            query_params
                .with(|q| q.is_settings_enabled())
                .then_some(view! {
                    <SettingsPanel />
                })
        };

        let secret = (move || {
            query_params
                .with(|q| q.secret.clone())
                .or(global_settings.global.with(|g| g.get_secret()))
        })
        .into_signal();
        let secret = create_memo(move |_| secret.get());
        let contest_query =
            (|| use_query::<ContestQuery>().get().unwrap_or_default()).into_signal();

        let animeitor =
        (move || {
            let contest_provider =
                create_local_resource(move || contest_query.get(), |q| provide_contest(q));
            let config_contest =
                create_local_resource(move || contest_query.get(), |q| create_config(q));

            match secret.get() {
                Some(secret) => (move || view! {
                    <ConfiguredReveleitor contest_provider secret=secret.clone() sede_param=query_params.with(|p| p.sede.clone()) />
                }).into_view(),
                None => view! {
                    <Navigation config_contest query=contest_query />
                    <Suspense fallback=|| view! { <p> Loading contest... </p> }>
                    {
                        move || contest_provider.with(|contest_provider|
                            contest_provider.as_ref().map(|provider|
                                view!{
                                    <ProvideSede
                                            contest=provider.running_contest
                                            contest_signal=provider.new_contest_signal.clone()
                                            panel_items=&provider.runs_panel_item_manager
                                            timer
                                            config_contest=provider.config_contest.clone()
                                            sede_param=query_params
                                            />
                                        }
                                    )
                                )
                            }
                            </Suspense>
                        }.into_view(),
                    }
                })
                .into_view();

        if negative_memo.get() {
            view! { <Timer timer /> }.into_view()
        } else {
            view! {
                <RemoteControl />
                {settings_panel}
                {animeitor}
            }
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
