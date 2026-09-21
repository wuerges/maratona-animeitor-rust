use std::sync::Arc;

use data::{
    configdata::{ConfigContest, Sede},
    ContestFile, TimerData,
};
use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::{use_params, use_query},
    params::Params,
    *,
};

use client_model::{
    contest_signal::ContestSignal, runs_panel_signal::RunsPanelItemManager, ContestProvider,
};

use crate::{
    api::{create_timer, provide_contest, EventContest},
    views::{
        background_color::BackgroundColor, contest::Contest, control_scrolling::RemoteControl,
        global_settings::use_global_settings, landing::Landing, navigation::Navigation,
        settings_accordion::SettingsAccordion,
    },
};

use super::{countdown::Countdown, reveleitor::Reveleitor};

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
    event_contest: EventContest,
    export_generation: u64,
) -> impl IntoView {
    let secret = secret.clone();
    let sede_param = sede_param.clone();

    Suspend::new(async move {
        let provider = contest_provider.await;
        let titulo = use_titulo(provider.config_contest.clone());
        let sede = use_configured_sede(provider.config_contest.clone(), titulo, sede_param);

        {
            view! { <Reveleitor sede secret contest=provider.starting_contest.clone() event_contest export_generation /> }
        }
    })
}

/// The event/contest path params of the contest route. Fields are `Option`
/// because `Params` on stable only supports optional fields.
#[derive(PartialEq, Eq, Clone, Debug, Default)]
struct ContestParams {
    event: Option<String>,
    contest: Option<String>,
}

impl Params for ContestParams {
    fn from_map(map: &params::ParamsMap) -> std::result::Result<Self, params::ParamsError> {
        Ok(ContestParams {
            event: map.get("event"),
            contest: map.get("contest"),
        })
    }
}

fn ec_of(params: ContestParams) -> Option<EventContest> {
    Some(EventContest {
        event: params.event?,
        contest: params.contest?,
    })
}

/// The contest screen: countdown while the timer is negative, the scoreboard
/// once it starts. Everything reads the route params reactively, so
/// navigating between contests rebuilds the screen for the new contest; the
/// countdown/scoreboard flip is a memoized branch (it swaps only when the
/// timer actually crosses zero, not on every tick).
#[component]
fn ContestScreen() -> impl IntoView {
    let global_settings = use_global_settings();
    let params = use_params::<ContestParams>();

    view! {
        {move || {
            let Some(ec) = params.get().ok().and_then(ec_of) else {
                return view! { <Landing /> }.into_any();
            };
            let timer = create_timer(ec.clone());

            let board_visible = Memo::new(move |_| !timer.with(|pair| pair.is_negative()));

            if board_visible.get() {
                let query_params = use_static_query();
                let sede_param = Memo::new(move |_| query_params.with(|p| p.sede.clone()));
                let export = crate::offline::OfflineExportContext::new();
                provide_context(export);

                let secret = Signal::derive(move || {
                    query_params
                        .with(|q| q.secret.clone())
                        .or(global_settings.global.with(|g| g.get_secret()))
                });
                let secret = Memo::new(move |_| secret.get());

                let show_settings = Memo::new(move |_| {
                    secret.get().is_some() || query_params.with(|q| q.is_settings_enabled())
                });
                let settings_panel = view! {
                    <Show when=move || show_settings.get()>
                        <SettingsAccordion>
                            <Show when=move || secret.get().is_some()>
                                {move || match export.inputs() {
                                    Some(inputs) => view! {
                                        <crate::offline::SaveOffline contest=inputs.contest.clone() runs=inputs.runs.clone() sede=inputs.sede.clone() />
                                    }.into_any(),
                                    None => view! {
                                        <p class="offline-save-loading" role="status">"Loading revelation data…"</p>
                                    }.into_any(),
                                }}
                            </Show>
                        </SettingsAccordion>
                    </Show>
                };
                let animeitor = {
                    let animeitor_ec = ec.clone();
                    move || {
                        let contest_provider = LocalResource::new({
                            let ec = animeitor_ec.clone();
                            move || provide_contest(ec.clone())
                        });

                        match secret.get() {
                            Some(secret) => {
                                let ec = animeitor_ec.clone();
                                (move || {
                                    let generation = export.begin();
                                    view! {
                                    <ConfiguredReveleitor contest_provider=contest_provider secret=secret.clone() sede_param=sede_param.get() event_contest=ec.clone() export_generation=generation />
                                    }
                                }).into_any()
                            },
                            None => {
                                export.begin();
                                let suspend = Suspend::new(async move {
                                    let provider = contest_provider.await;

                                    view! {
                                        <Navigation config_contest=provider.config_contest.clone() />
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
                                {suspend}
                            }.into_any()}
                        }
                            .into_view()
                    }
                };
                view! {
                    <BackgroundColor />
                    <RemoteControl event_contest=ec.clone() />
                    {settings_panel}
                    {animeitor}
                }
                .into_any()
            } else {
                view! { <Countdown ec=ec.clone() timer /> }.into_any()
            }
        }}
    }
}

#[component]
pub fn Sedes() -> AnyView {
    // One router for the whole app; the event/contest come from the route
    // params. The countdown/scoreboard switch is a branch inside the contest
    // screen, not a router guard: ProtectedRoute's condition/redirect run in
    // a context-less Transition scope where no router hooks work.
    view! {
        <Router>
            <Routes fallback=move || view! { <Landing /> }>
                <Route path=path!("/") view=Landing />
                <Route path=path!("/animeitor/:event/:contest") view=ContestScreen />
            </Routes>
        </Router>
    }
    .into_any()
}
