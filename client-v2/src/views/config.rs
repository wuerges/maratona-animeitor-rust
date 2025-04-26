use leptos::prelude::*;

use crate::api::{create_config, ContestQuery};

#[component]
pub fn Config(query: Signal<ContestQuery>) -> impl IntoView {
    let config = Resource::new_blocking(move || query.get(), |q| create_config(q));

    move || match config.get() {
        Some(config) => view! {
            <p> Config!: "`"{format!("{:#?}", config)}"'" </p>
        }
        .into_any(),
        None => view! {<p> Config is none =/ </p>}.into_any(),
    }
}
