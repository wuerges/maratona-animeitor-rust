use data::{RunTuple, RunsFile};
use leptos::{leptos_dom::logging::console_log, prelude::*, *};

use crate::websocket_signal::create_websocket_signal;

fn create_runs() -> ReadSignal<RunsFile> {
    let runs_message =
        create_websocket_signal::<Option<RunTuple>>("ws://localhost:9000/api/allruns_ws", None);

    let (runs_file, set_runs_file) = create_signal::<RunsFile>(RunsFile::empty());

    create_effect(move |_| {
        let next = runs_message.get();
        console_log(&format!("read next message: {next:?}"));
        if let Some(next) = next {
            set_runs_file.update(|rf| {
                rf.refresh_1(&next);
            });
        }
    });

    runs_file
}

#[component]
pub fn Runs() -> impl IntoView {
    let runs_file = create_runs();

    let txt = move || format!("{:#?}", runs_file.get());

    view! { <p> Runs: {txt} </p> }
}
