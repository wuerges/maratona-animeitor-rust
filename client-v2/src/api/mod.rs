use data::{configdata::ConfigContest, ContestFile, RunTuple, TimerData};
use futures::{channel::mpsc::UnboundedReceiver, StreamExt};

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

pub fn create_timer() -> ReadSignal<(TimerData, TimerData)> {
    let mut timer_stream = create_websocket_stream::<TimerData>("ws://localhost:9000/api/timer");

    let (timer, set_timer) = create_signal((TimerData::fake(), data::TimerData::new(0, 1)));

    spawn_local(async move {
        loop {
            let next = timer_stream.next().await;
            if let Some(next) = next {
                set_timer.update(|(new, old)| {
                    *old = *new;
                    *new = next;
                });
            }
        }
    });

    timer
}
