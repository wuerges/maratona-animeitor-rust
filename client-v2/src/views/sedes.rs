use std::sync::Arc;

use data::{
    configdata::{ConfigContest, Sede},
    ContestFile, TimerData,
};
use leptos::prelude::*;
use leptos_router::{
    components::{ProtectedRoute, Route, Router, Routes},
    hooks::{use_location, use_navigate, use_params, use_query},
    params::Params,
    *,
};

use client_model::{
    contest_signal::ContestSignal, runs_panel_signal::RunsPanelItemManager, ContestProvider,
};

use crate::{
    api::{create_timer, provide_contest, EventContest},
    views::{
        background_color::BackgroundColor,
        contest::Contest,
        control_scrolling::RemoteControl,
        global_settings::{use_global_settings, SettingsPanel},
        landing::Landing,
        navigation::Navigation,
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
) -> impl IntoView {
    let secret = secret.clone();
    let sede_param = sede_param.clone();

    Suspend::new(async move {
        let provider = contest_provider.await;
        let titulo = use_titulo(provider.config_contest.clone());
        let sede = use_configured_sede(provider.config_contest.clone(), titulo, sede_param);

        {
            view! { <Reveleitor sede secret contest=provider.starting_contest.clone() event_contest /> }
        }
    })
}

/// The event/contest path params of the contest routes. Fields are
/// `Option` because `Params` on stable only supports optional fields.
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

/// The event/contest from a pathname (`/animeitor/{event}/{contest}[/...]`).
fn ec_from_pathname(pathname: &str) -> Option<EventContest> {
    let segments: Vec<&str> = pathname.split('/').filter(|s| !s.is_empty()).collect();
    client_model::path::event_contest_from_segments(&segments)
}

/// The scoreboard screen for a contest. Everything reads the route params
/// reactively: navigating between contests re-runs the `{}` closure and
/// rebuilds the screen for the new contest.
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

            let query_params = use_static_query();

            let secret = Signal::derive(move || {
                query_params
                    .with(|q| q.secret.clone())
                    .or(global_settings.global.with(|g| g.get_secret()))
            });
            let secret = Memo::new(move |_| secret.get());

            let settings_panel = move || {
                query_params
                    .with(|q| q.is_settings_enabled())
                    .then_some(view! {
                        <SettingsPanel />
                    })
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
                            (move || view! {
                                <ConfiguredReveleitor contest_provider=contest_provider secret=secret.clone() sede_param=query_params.with(|p| p.sede.clone()) event_contest=ec.clone() />
                            }).into_any()
                        },
                        None => {
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
        }}
    }
}

/// The countdown screen: shows the remaining time and navigates back to the
/// contest route (replacing the history entry) as soon as the timer turns
/// positive.
#[component]
fn CountdownScreen() -> impl IntoView {
    let params = use_params::<ContestParams>();

    view! {
        {move || {
            let Some(ec) = params.get().ok().and_then(ec_of) else {
                return view! { <Landing /> }.into_any();
            };
            let timer = create_timer(ec.clone());

            let navigate = use_navigate();
            let back = format!("/animeitor/{}/{}", ec.event, ec.contest);
            Effect::new(move |_| {
                if !timer.with(|pair| pair.is_negative()) {
                    navigate(
                        &back,
                        NavigateOptions {
                            replace: true,
                            ..Default::default()
                        },
                    );
                }
            });
            view! { <Countdown ec=ec.clone() timer /> }.into_any()
        }}
    }
}

#[component]
pub fn Sedes() -> AnyView {
    // One router for the whole app. The event/contest come from the route
    // params (not from parsing `window.location`): the landing is a route,
    // the contest route is guarded by the timer and redirects to its
    // countdown route while it is negative.
    view! {
        <Router>
            <Routes fallback=move || view! { <Landing /> }>
                <Route path=path!("/") view=Landing />
                <ProtectedRoute
                    path=path!("/animeitor/:event/:contest")
                    view=ContestScreen
                    // The ProtectedRoute condition and redirect run inside a
                    // Transition child scope where the matched-route params
                    // context is NOT available (use_params panics there).
                    // The location works everywhere under the Router, and
                    // the URL is the contest path when these run.
                    condition=move || {
                        let location = use_location();
                        let Some(ec) = ec_from_pathname(&location.pathname.get()) else {
                            return Some(true);
                        };
                        Some(!create_timer(ec).with(|pair| pair.is_negative()))
                    }
                    redirect_path=move || {
                        let location = use_location();
                        match ec_from_pathname(&location.pathname.get()) {
                            Some(ec) => format!("/animeitor/{}/{}/countdown", ec.event, ec.contest),
                            None => "/".to_string(),
                        }
                    }
                />
                <Route
                    path=path!("/animeitor/:event/:contest/countdown")
                    view=CountdownScreen
                />
            </Routes>
        </Router>
    }
    .into_any()
}
