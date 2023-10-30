use data::{configdata::ConfigContest, ContestFile, RunTuple, RunsFile, TimerData};
use futures::StreamExt;
use gloo_timers::future::TimeoutFuture;
use leptos::{leptos_dom::logging::console_log, *};

use crate::net::{
    request_signal::create_request_signal, websocket_stream::create_websocket_stream,
};

pub fn create_contest() -> ReadSignal<Option<ContestFile>> {
    let contest_message = create_request_signal("http://localhost:9000/api/contest", None);

    contest_message
}

pub fn create_config() -> ReadSignal<Option<ConfigContest>> {
    let config_message = create_request_signal("http://localhost:9000/api/config", None);

    config_message
}

pub fn create_runs() -> ReadSignal<RunsFile> {
    let runs_message = create_websocket_stream::<RunTuple>("ws://localhost:9000/api/allruns_ws");

    let mut messages_stream = runs_message.ready_chunks(100_000);

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

pub fn create_timer() -> ReadSignal<Option<TimerData>> {
    let timer_stream = create_websocket_stream("ws://localhost:9000/api/timer");
    let timer_message = create_signal_from_stream(timer_stream);

    let (timer, set_timer) = create_signal(None);

    create_effect(move |_| {
        let next = timer_message.get();

        if next.is_some() && next != timer.get() {
            set_timer.set(next);
        }
    });

    timer
}
