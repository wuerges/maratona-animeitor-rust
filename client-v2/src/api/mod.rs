use data::{configdata::ConfigContest, ContestFile, RunTuple, TimerData};
use futures::channel::mpsc::UnboundedReceiver;

use leptos::*;

use crate::net::{request_signal::create_request, websocket_stream::create_websocket_stream};

pub async fn create_contest() -> ContestFile {
    let contest_message = create_request("http://localhost:9000/api/contest").await;

    contest_message
}

pub async fn create_config() -> ConfigContest {
    let config_message = create_request("http://localhost:9000/api/config").await;

    config_message
}

pub fn create_runs() -> UnboundedReceiver<RunTuple> {
    create_websocket_stream::<RunTuple>("ws://localhost:9000/api/allruns_ws")
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
