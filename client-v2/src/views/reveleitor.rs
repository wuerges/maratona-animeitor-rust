use data::{configdata::Sede, ContestFile};
use leptos::{logging::log, *};

use crate::api::{create_config, create_contest, create_secret_runs};

#[component]
pub fn Reveleitor(sede: Sede) -> impl IntoView {
    let all_runs = create_local_resource(|| (), |()| create_secret_runs("saltsecret"));
    let contest = create_local_resource(|| (), |()| create_contest());
    let config = create_local_resource(|| (), |()| create_config());

    move || match (all_runs.get(), contest.get(), config.get()) {
        (Some(_), Some(_), Some(_)) => Some(view! {
            <p> Loaded stuff </p>
        }),
        _ => None,
    }
}
