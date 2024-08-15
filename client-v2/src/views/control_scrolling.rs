use data::remote_control::{ControlMessage, WindowScroll};
use leptos::*;
use leptos_dom::logging::console_error;
use leptos_router::{use_query, Params};
use leptos_use::{use_idle, use_websocket, use_window_scroll, UseIdleReturn, UseWebsocketReturn};
use web_sys::ScrollToOptions;

use crate::api::remote_control_url;

#[derive(Params, PartialEq, Eq, Clone, Default)]
struct RemoteControlQuery {
    remote_control: Option<String>,
}

#[component]
fn Scrolling<SendFn: Fn(&str) + 'static>(
    idle: Signal<bool>,
    message_signal: Memo<Option<ControlMessage>>,
    send: SendFn,
) -> impl IntoView {
    let (_get_x, get_y) = use_window_scroll();

    let window = web_sys::window().unwrap();

    create_effect(move |_| {
        if !idle.get() {
            match serde_json::to_string(&WindowScroll { y: get_y.get() }) {
                Ok(text) => send(&text),
                Err(err) => console_error(&format!("failed serializing idle scroll {:?}", err)),
            }
        }
    });

    create_effect(move |_| {
        if let Some(message) = message_signal.get() {
            match message {
                ControlMessage::WindowScroll(WindowScroll { y }) => window
                    .scroll_to_with_scroll_to_options(
                        ScrollToOptions::new()
                            .behavior(web_sys::ScrollBehavior::Smooth)
                            .top(y),
                    ),
            }
        }
    });
}

#[component]
pub fn RemoteControl() -> impl IntoView {
    let query = use_query::<RemoteControlQuery>();

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
                    <Scrolling idle message_signal send />
                }
            })
    }
}
