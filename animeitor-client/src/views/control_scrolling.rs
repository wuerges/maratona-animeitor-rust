use codee::string::FromToStringCodec;
use data::remote_control::{ControlMessage, QueryString, WindowScroll};
use leptos::{logging::error, prelude::*};
use leptos_router::{
    hooks::{use_navigate, use_query, use_query_map},
    params::Params,
};
use leptos_use::{
    signal_throttled, use_idle, use_websocket, use_window_scroll, UseIdleReturn, UseWebSocketReturn,
};
use web_sys::ScrollToOptions;

use crate::api::remote_control_url;

use super::team_media::{use_global_photo_state, PhotoState};

#[derive(PartialEq, Eq, Clone, Default)]
struct RemoteControlQuery {
    remote_control: Option<String>,
}

impl Params for RemoteControlQuery {
    fn from_map(
        map: &leptos_router::params::ParamsMap,
    ) -> std::result::Result<Self, leptos_router::params::ParamsError> {
        let remote_control = map.get("remote_control");
        Ok(RemoteControlQuery { remote_control })
    }
}

#[component]
fn Tab<SendFn: Fn(&String) + 'static>(idle: Signal<bool>, send: SendFn) -> impl IntoView {
    let query_params = use_query_map();

    let memo = Memo::new(move |_| (query_params.get(), idle.get()));

    Effect::new(move |_| {
        let (params, idle) = memo.get();
        if !idle {
            match serde_json::to_string(&QueryString {
                query: params.to_query_string(),
            }) {
                Ok(text) => send(&text),
                Err(err) => error!("failed serializing idle scroll {:?}", err),
            }
        }
    });
}

fn into_data_photo_state(photo_state: PhotoState) -> data::remote_control::PhotoState {
    match photo_state {
        PhotoState::Hidden => data::remote_control::PhotoState::Hidden,
        PhotoState::Show(team_login) => data::remote_control::PhotoState::Show(team_login),
    }
}
fn from_data_photo_state(photo_state: data::remote_control::PhotoState) -> PhotoState {
    match photo_state {
        data::remote_control::PhotoState::Hidden => PhotoState::Hidden,
        data::remote_control::PhotoState::Show(team_login) => PhotoState::Show(team_login),
    }
}

#[component]
fn ShowTeamPhoto<SendFn: Fn(&String) + 'static>(
    idle: Signal<bool>,
    send: SendFn,
    photo_state: RwSignal<PhotoState>,
) -> impl IntoView {
    let memo = Memo::new(move |_| (photo_state.get(), idle.get()));

    Effect::new(move |_| {
        let (photo_state, idle) = memo.get();
        if !idle {
            match serde_json::to_string(&into_data_photo_state(photo_state)) {
                Ok(text) => send(&text),
                Err(err) => error!("failed serializing idle scroll {:?}", err),
            }
        }
    });
}

#[component]
fn Effects(
    idle: Signal<bool>,
    message_signal: Memo<Option<ControlMessage>>,
    photo_state: RwSignal<PhotoState>,
) -> impl IntoView {
    let window = web_sys::window().unwrap();
    let navigate = use_navigate();
    let options = ScrollToOptions::new();
    options.set_behavior(web_sys::ScrollBehavior::Smooth);

    Effect::new(move |_| {
        if idle.get() {
            if let Some(message) = message_signal.get() {
                match message {
                    ControlMessage::WindowScroll(WindowScroll { y }) => {
                        options.set_top(y);
                        window.scroll_to_with_scroll_to_options(&options)
                    }
                    ControlMessage::QueryString(QueryString { query }) => {
                        navigate(&query, Default::default())
                    }
                    ControlMessage::PhotoState(state) => {
                        photo_state.set(from_data_photo_state(state))
                    }
                }
            }
        }
    });
}

#[component]
fn Scrolling<SendFn: Fn(&String) + 'static>(idle: Signal<bool>, send: SendFn) -> impl IntoView {
    let (_get_x, get_y) = use_window_scroll();

    let memo_y = Memo::new(move |_| get_y.get());
    let throttled_y = signal_throttled(memo_y, 300.0);

    let memo = Memo::new(move |_| (idle.get(), throttled_y.get()));

    Effect::new(move |_| {
        let (idle, y) = memo.get();
        if !idle {
            match serde_json::to_string(&WindowScroll { y }) {
                Ok(text) => send(&text),
                Err(err) => error!("failed serializing idle scroll {:?}", err),
            }
        }
    });
}

#[component]
pub fn RemoteControl(event_contest: crate::api::EventContest) -> impl IntoView {
    let query = use_query::<RemoteControlQuery>();

    let photo_state = use_global_photo_state();

    // Memo (PartialEq-filtered) so the websocket is recreated only when the
    // remote_control key actually changes — not on every query-param change
    // (e.g. controller navigation).
    let key = Memo::new(move |_| query.get().ok().and_then(|q| q.remote_control.clone()));

    move || {
        key.get().map(|key| {
            // leptos-use 0.19 ties the websocket's hidden lifetime to the
            // url, so it must outlive the returned view. Leaked per key
            // change; the memo above recreates this only on key change.
            let url: &'static str =
                Box::leak(remote_control_url(&event_contest, &key).into_boxed_str());
            let UseWebSocketReturn { message, send, .. } =
                use_websocket::<String, String, FromToStringCodec>(url);
            let UseIdleReturn { idle, .. } = use_idle(5_000);

            let message_signal = Memo::new(move |_| {
                message
                    .get()
                    .and_then(|text| serde_json::from_str::<ControlMessage>(&text).ok())
            });

            view! {
                <Scrolling idle send=send.clone() />
                <Tab idle send=send.clone() />
                <ShowTeamPhoto idle send photo_state />
                <Effects idle message_signal photo_state />
            }
        })
    }
}
