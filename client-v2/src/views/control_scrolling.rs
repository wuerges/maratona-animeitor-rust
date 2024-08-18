use data::remote_control::{ControlMessage, QueryString, WindowScroll};
use leptos::*;
use leptos_dom::logging::console_error;
use leptos_router::{use_navigate, use_query, use_query_map, Params};
use leptos_use::{
    signal_throttled, use_idle, use_websocket, use_window_scroll, UseIdleReturn, UseWebsocketReturn,
};
use web_sys::ScrollToOptions;

use crate::api::remote_control_url;

use super::team_media::{use_global_photo_state, PhotoState};

#[derive(Params, PartialEq, Eq, Clone, Default)]
struct RemoteControlQuery {
    remote_control: Option<String>,
}

#[component]
fn Tab<SendFn: Fn(&str) + 'static>(idle: Signal<bool>, send: SendFn) -> impl IntoView {
    let query_params = use_query_map();

    let memo = create_memo(move |_| (query_params.get(), idle.get()));

    create_effect(move |_| {
        let (params, idle) = memo.get();
        if !idle {
            match serde_json::to_string(&QueryString {
                query: params.to_query_string(),
            }) {
                Ok(text) => send(&text),
                Err(err) => console_error(&format!("failed serializing idle scroll {:?}", err)),
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
fn ShowTeamPhoto<SendFn: Fn(&str) + 'static>(
    idle: Signal<bool>,
    send: SendFn,
    photo_state: RwSignal<PhotoState>,
) -> impl IntoView {
    let memo = create_memo(move |_| (photo_state.get(), idle.get()));

    create_effect(move |_| {
        let (photo_state, idle) = memo.get();
        if !idle {
            match serde_json::to_string(&into_data_photo_state(photo_state)) {
                Ok(text) => send(&text),
                Err(err) => console_error(&format!("failed serializing idle scroll {:?}", err)),
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

    create_effect(move |_| {
        if idle.get() {
            if let Some(message) = message_signal.get() {
                match message {
                    ControlMessage::WindowScroll(WindowScroll { y }) => window
                        .scroll_to_with_scroll_to_options(
                            ScrollToOptions::new()
                                .behavior(web_sys::ScrollBehavior::Smooth)
                                .top(y),
                        ),
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
fn Scrolling<SendFn: Fn(&str) + 'static>(idle: Signal<bool>, send: SendFn) -> impl IntoView {
    let (_get_x, get_y) = use_window_scroll();

    let memo_y = create_memo(move |_| get_y.get());
    let throttled_y = signal_throttled(memo_y, 300.0);

    let memo = create_memo(move |_| (idle.get(), throttled_y.get()));

    create_effect(move |_| {
        let (idle, y) = memo.get();
        if !idle {
            match serde_json::to_string(&WindowScroll { y }) {
                Ok(text) => send(&text),
                Err(err) => console_error(&format!("failed serializing idle scroll {:?}", err)),
            }
        }
    });
}

#[component]
pub fn RemoteControl() -> impl IntoView {
    let query = use_query::<RemoteControlQuery>();

    let photo_state = use_global_photo_state();

    move || {
        query
            .get()
            .ok()
            .and_then(|key| key.remote_control)
            .map(|key| {
                let UseWebsocketReturn { message, send, .. } =
                    use_websocket(&remote_control_url(&key));
                let UseIdleReturn { idle, .. } = use_idle(5_000);

                let message_signal = create_memo(move |_| {
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
