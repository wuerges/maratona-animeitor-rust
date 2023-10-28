use std::future::ready;

use data::{RunTuple, RunsFile};
use futures::StreamExt;
use gloo_timers::future::TimeoutFuture;
use leptos::{leptos_dom::logging::console_log, prelude::*, *};

use crate::websocket_signal::create_websocket_signal;

fn create_runs() -> ReadSignal<RunsFile> {
    let runs_message =
        create_websocket_signal::<Option<RunTuple>>("ws://localhost:9000/api/allruns_ws", None);

    let mut messages_stream = runs_message
        .to_stream()
        .filter_map(|x| ready(x))
        .ready_chunks(100_000);

    let (runs_file, set_runs_file) = create_signal::<RunsFile>(RunsFile::empty());

    spawn_local(async move {
        loop {
            TimeoutFuture::new(1_000).await;
            let next_chunk = messages_stream.next().await;
            let size = next_chunk.as_ref().map(|v| v.len()).unwrap_or_default();
            console_log(&format!("read next {size:?} runs"));
            if let Some(next_chunk) = next_chunk {
                set_runs_file.update(|rf| {
                    for run_tuple in next_chunk {
                        rf.refresh_1(&run_tuple);
                    }
                });
            }
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
