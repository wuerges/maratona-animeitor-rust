use leptos::*;

use crate::api::create_config;

#[component]
pub fn Config() -> impl IntoView {
    let config = create_config();

    move || match config.get() {
        Some(config) => view! {
            <p> Config!: "`"{format!("{:#?}", config)}"'" </p>
        },
        None => view! {<p> Config is none =/ </p>},
    }
}
