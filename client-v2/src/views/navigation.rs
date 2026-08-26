use std::sync::Arc;

use data::configdata::{ConfigContest, SedeEntry};
use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_query_map, params::ParamsMap};

#[component]
fn Sede(sede: SedeEntry, query: Memo<ParamsMap>) -> impl IntoView {
    let name = sede.name.clone();

    move || {
        let mut params = query.get();
        let name = name.clone();
        params.replace("sede".to_string(), name.clone());
        view! {
            <span class="sedeslink">
                <A href=params.to_query_string()> {name} </A>
            </span>
        }
    }
}

#[component]
pub fn Navigation(config_contest: Arc<ConfigContest>) -> impl IntoView {
    let query = use_query_map();

    view! {
        <div class="sedesnavigation">
            {config_contest
                .sedes
                .iter()
                .flatten()
                .cloned()
                .map(move |sede| {
                    view! {<Sede sede query />}
                })
                .collect_view()}
        </div>
    }
}
