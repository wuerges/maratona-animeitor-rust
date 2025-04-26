use leptos::{logging::log, prelude::*};
use leptos_router::hooks::use_query_map;
use leptos_use::use_document;

use crate::views::global_settings::use_global_settings;

#[component]
pub fn BackgroundColor() -> impl IntoView {
    let settings = use_global_settings();

    let query = use_query_map();
    let query_bg = Signal::derive(move || query.with(|ps| ps.get("background-color")));

    Effect::new(move |_| {
        let color = query_bg
            .get()
            .or_else(|| settings.global.with(|g| g.background_color.clone()));
        let document = use_document();

        if let Some(body) = document.body() {
            match color {
                Some(color) => {
                    body.style()
                        .set_property("background-color", &color)
                        .and_then(|()| Ok(log!("updated background color")))
                        .ok();
                }
                None => {
                    body.style()
                        .remove_property("background-color")
                        .and_then(|x| Ok(log!("removed background color: {}", x)))
                        .ok();
                }
            }
        }
    });

    view! {}
}
