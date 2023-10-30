use leptos::{prelude::*, *};

use crate::api::create_runs;

#[component]
pub fn Runs() -> impl IntoView {
    let runs_file = create_runs();

    let txt = move || format!("{:#?}", runs_file.get());

    view! { <p> Runs: {txt} </p> }
}
