use leptos::*;

use crate::api::{create_config, ContestQuery};

#[component]
pub fn Config(query: Signal<ContestQuery>) -> impl IntoView {
    let config = create_resource(move || query.get(), |q| create_config(q));

    move || match config.get() {
        Some(config) => view! {
            <p> Config!: "`"{format!("{:#?}", config)}"'" </p>
        },
        None => view! {<p> Config is none =/ </p>},
    }
}
