use leptos::{logging::log, prelude::*};
use leptos_router::hooks::use_query_map;
use leptos_use::use_document;

use crate::views::global_settings::use_global_settings;

#[component]
pub fn BackgroundColor() -> impl IntoView {
    let query = use_query_map();
    let query_bg = Signal::derive(move || query.with(|ps| ps.get("background-color")));

    view! { <BackgroundColorValue override_color=query_bg /> }
}

#[component]
pub fn BackgroundColorValue(override_color: Signal<Option<String>>) -> impl IntoView {
    let settings = use_global_settings();

    Effect::new(move |_| {
        let color = override_color
            .get()
            .or_else(|| settings.global.with(|g| g.background_color.clone()));
        let document = use_document();

        if let Some(body) = document.body() {
            match color {
                Some(color) => {
                    body.style()
                        .set_property("background-color", &color)
                        .map(|()| log!("updated background color"))
                        .ok();
                }
                None => {
                    body.style()
                        .remove_property("background-color")
                        .map(|x| log!("removed background color: {}", x))
                        .ok();
                }
            }
        }
    });

    View::new(())
}
