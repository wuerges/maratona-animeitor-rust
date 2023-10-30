use leptos::{prelude::*, *};

use crate::model::provide_runs;

#[component]
pub fn Runs() -> impl IntoView {
    let runs_file = provide_runs();

    let txt = move || format!("{:#?}", runs_file.get());

    view! { <p> Runs: {txt} </p> }
}
