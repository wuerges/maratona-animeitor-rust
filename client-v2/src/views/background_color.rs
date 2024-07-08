use leptos::*;
use leptos_dom::logging::console_log;
use leptos_use::use_document;

use crate::views::global_settings::use_global_settings;

#[component]
pub fn BackgroundColor() -> impl IntoView {
    let settings = use_global_settings();

    create_effect(move |_| {
        let color = settings.global.with(|g| g.background_color.clone());
        let document = use_document();

        if let Some(body) = document.body() {
            match color {
                Some(color) => {
                    body.style()
                        .set_property("background-color", &color)
                        .and_then(|()| Ok(console_log("updated background color")))
                        .ok();
                }
                None => {
                    body.style()
                        .remove_property("background-color")
                        .and_then(|x| Ok(console_log(&format!("removed background color: {}", x))))
                        .ok();
                }
            }
        }
    });

    view! {}
}
