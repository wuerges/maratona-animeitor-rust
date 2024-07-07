use data::configdata::{ConfigContest, SedeEntry};
use leptos::*;
use leptos_router::*;

use crate::api::ContestQuery;

#[component]
fn Sede<'query>(sede: SedeEntry, query: &'query ContestQuery) -> impl IntoView {
    let contest = query
        .contest
        .as_ref()
        .map(|c| format!("&contest={c}"))
        .unwrap_or_default();
    view! {
        <span class="sedeslink">
            <A href=format!("?sede={}{}",sede.name, contest)> {sede.name} </A>
        </span>
    }
}

#[component]
pub fn Navigation(
    config_contest: Resource<ContestQuery, ConfigContest>,
    query: Signal<ContestQuery>,
) -> impl IntoView {
    move || {
        view! {
            <div class="sedesnavigation">
                <Suspense
                    fallback=||view! {<p> Loading... </p> }
                >
                    {move || {
                        let query = query.get();
                        config_contest.with(|config| config.as_ref().map(|contest| {
                            contest
                            .sedes.iter().flatten()
                            .cloned()
                            .map(move |sede| {
                                view! {<Sede sede query=&query />}
                            })
                            .collect_view()
                        }))
                    }}
                </Suspense>
            </div>
        }
    }
}
