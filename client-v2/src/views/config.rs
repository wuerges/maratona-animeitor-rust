use leptos::prelude::*;

use crate::api::{create_config, ContestQuery};

#[component]
pub fn Config(query: Signal<ContestQuery>) -> impl IntoView {
    let config = LocalResource::new(move || create_config(query.get()));

    move || match config.get() {
        Some(config) => view! {
            <p> Config!: "`"{format!("{:#?}", config)}"'" </p>
        }
        .into_any(),
        None => view! {<p> Config is none =/ </p>}.into_any(),
    }
}
