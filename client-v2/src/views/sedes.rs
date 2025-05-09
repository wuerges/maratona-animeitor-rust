use std::sync::Arc;

use data::{
    configdata::{ConfigContest, Sede},
    ContestFile, TimerData,
};
use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_query,
    params::Params,
    *,
};

use crate::{
    api::{create_config, create_timer, ContestQuery},
    model::{
        contest_signal::ContestSignal, provide_contest, runs_panel_signal::RunsPanelItemManager,
        ContestProvider,
    },
    views::{
        background_color::BackgroundColor,
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

#[derive(PartialEq, Eq, Clone, Debug, Default)]
struct QueryParams {
    sede: Option<String>,
    secret: Option<String>,
    settings: Option<bool>,
}

impl Params for QueryParams {
    fn from_map(map: &params::ParamsMap) -> std::result::Result<Self, params::ParamsError> {
        let sede = map.get("sede");
        let secret = map.get("secret");
        let settings = map.get("settings").and_then(|s| s.parse::<bool>().ok());
        Ok(QueryParams {
            sede,
            secret,
            settings,
        })
    }
}

impl QueryParams {
    fn is_settings_enabled(&self) -> bool {
        self.settings.unwrap_or_default()
    }
}

fn use_static_query() -> Signal<QueryParams> {
    let query_params = use_query::<QueryParams>();
    Signal::derive(move || query_params.get().ok().unwrap_or_default())
}

fn use_configured_sede(
    config: Arc<ConfigContest>,
    titulo: Arc<Sede>,
    sede_param: Option<String>,
) -> Arc<Sede> {
    let config = config.into_contest();
    let sub_sede = sede_param
        .and_then(|sede| config.get_sede_nome_sede(&sede))
        .cloned()
        .map(Arc::new);

    sub_sede.unwrap_or(titulo)
}

fn use_titulo(config: Arc<ConfigContest>) -> Arc<Sede> {
    let config = config.into_contest();
    Arc::new(config.titulo)
}

#[component]
fn ProvideSede(
    original_contest: Arc<ContestFile>,
    contest_signal: Arc<ContestSignal>,
    panel_items: Arc<RunsPanelItemManager>,
    config_contest: Arc<ConfigContest>,
    timer: ReadSignal<(TimerData, TimerData)>,
    sede_param: Signal<QueryParams>,
) -> impl IntoView {
    let titulo = use_titulo(config_contest.clone());
    let titulo_sede = titulo.clone();
    let sede = Memo::new(move |_| {
        use_configured_sede(
            config_contest.clone(),
            titulo_sede.clone(),
            sede_param.get().sede,
        )
    });

    let titulo = Signal::derive(move || {
        sede.with(|s| {
            if s.entry.name == titulo.entry.name {
                None
            } else {
                Some(titulo.clone())
            }
        })
    });

    view! { <Contest original_contest contest_signal panel_items timer titulo sede=sede.into() /> }
}

#[component]
fn ConfiguredReveleitor(
    contest_provider: LocalResource<ContestProvider>,
    secret: String,
    sede_param: Option<String>,
) -> impl IntoView {
    let secret = secret.clone();
    let sede_param = sede_param.clone();

    Suspend::new(async move {
        let provider = contest_provider.await;
        let titulo = use_titulo(provider.config_contest.clone());
        let sede = use_configured_sede(provider.config_contest.clone(), titulo, sede_param);

        {
            view! { <Reveleitor sede secret contest=provider.starting_contest.clone() /> }
        }
    })
}

#[component]
pub fn Sedes() -> impl IntoView {
    let timer = create_timer();

    let negative_memo = Memo::new(move |_| timer.get().is_negative());

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

        let secret = Signal::derive(move || {
            query_params
                .with(|q| q.secret.clone())
                .or(global_settings.global.with(|g| g.get_secret()))
        });
        let secret = Memo::new(move |_| secret.get());
        let contest_query =
            Signal::derive(|| use_query::<ContestQuery>().get().unwrap_or_default());

        let animeitor = move || {
            let contest_provider = LocalResource::new(move || {
                let q = contest_query.get();
                provide_contest(q)
            });

            match secret.get() {
                Some(secret) => (move || view! {
                    <ConfiguredReveleitor contest_provider=contest_provider secret=secret.clone() sede_param=query_params.with(|p| p.sede.clone()) />
                }).into_any(),
                None => {
                    let config_contest = LocalResource::new(move || {
                        let q = contest_query.get();
                        create_config(q)
                    });
                    let suspend = Suspend::new(async move {
                        let provider = contest_provider.await;

                        view! {
                            <ProvideSede
                                    original_contest=provider.starting_contest.clone()
                                    contest_signal=provider.new_contest_signal.clone()
                                    panel_items=provider.runs_panel_item_manager
                                    timer
                                    config_contest=provider.config_contest.clone()
                                    sede_param=query_params
                                    />
                        }
                    });

                    view! {
                    <Navigation config_contest />
                    {suspend}
                }.into_any()}
            }
                .into_view()
        };

        if negative_memo.get() {
            view! { <Timer timer /> }.into_any()
        } else {
            view! {
                <BackgroundColor />
                <RemoteControl />
                {settings_panel}
                {animeitor}
            }
            .into_any()
        }
    };

    view! {
        <Router>
            <Routes fallback=move || root>
                <Route
                path=path!("")
                view=move || root
                />
            </Routes>
        </Router>
    }
}
