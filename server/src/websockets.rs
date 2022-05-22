use crate::DB;

use std::sync::Arc;
use tokio::sync::Mutex;
use futures::{SinkExt, StreamExt, stream::SplitSink};
use warp::ws::Message;
use tokio::sync::broadcast;

pub async fn serve_timer_ws(ws: warp::ws::WebSocket, runs: Arc<Mutex<DB>>) {
    let (mut tx, _) = ws.split();

    let fut = async move {
        let dur = tokio::time::Duration::new(1, 0);
        let mut interval = tokio::time::interval(dur);
        let mut old = data::TimerData::fake();

        loop {
            interval.tick().await;
            let l = runs.lock().await.timer_data();

            if l != old {
                old = l;
                let message = serde_json::to_string(&l).map(Message::text)
                    .unwrap_or_else(|error| panic!("Could not convert `{:?}' to a message: {:?}", l, error));
                tx.send(message).await.unwrap_or_else(|error| panic!("Could not send message: {:?}", error));
            }
        }
    };

    tokio::task::spawn(fut);
}

async fn convert_and_send(tx: &mut SplitSink<warp::ws::WebSocket, Message>, r : data::RunTuple) -> bool {
    match serde_json::to_string(&r).map(Message::text) {
        Err(e) => {
            panic!("Error preparing run {:?} message: {:?}", r, e);
        }
        Ok(m) => tx.send(m).await.is_ok()
    }
}

pub async fn serve_all_runs_ws(
    ws: warp::ws::WebSocket,
    runs: Arc<Mutex<DB>>,
    mut rx: broadcast::Receiver<data::RunTuple>,
) {
    let (mut tx, _) = ws.split();

    let fut = async move {
        {
            let lock = runs.lock().await;

            for r in lock.all_runs() {
                if !convert_and_send(&mut tx, r).await {
                    return
                }
            }
        }

        loop {
            let r = rx.recv().await.expect("Expected a RunTuple");
            if !convert_and_send(&mut tx, r).await {
                return
            }
        }
    };

    tokio::task::spawn(fut);
}