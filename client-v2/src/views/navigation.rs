use data::configdata::{ConfigContest, SedeEntry};
use leptos::*;
use leptos_router::*;

use crate::api::ContestQuery;

#[component]
fn Sede(sede: SedeEntry, query: Memo<ParamsMap>) -> impl IntoView {
    let name = sede.name.clone();

    move || {
        let mut params = query.get();
        let name = name.clone();
        params.insert("sede".to_string(), name.clone());
        view! {
            <span class="sedeslink">
                <A href=params.to_query_string()> {name} </A>
            </span>
        }
    }
}

#[component]
pub fn Navigation(config_contest: Resource<ContestQuery, ConfigContest>) -> impl IntoView {
    let query = use_query_map();

    move || {
        view! {
            <div class="sedesnavigation">
                <Suspense
                    fallback=||view! {<p> Loading... </p> }
                >
                    {move || {
                        config_contest.with(|config| config.as_ref().map(|contest| {
                            contest
                            .sedes.iter().flatten()
                            .cloned()
                            .map(move |sede| {
                                view! {<Sede sede query />}
                            })
                            .collect_view()
                        }))
                    }}
                </Suspense>
            </div>
        }
    }
}
