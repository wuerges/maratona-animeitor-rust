use crate::DB;

use std::sync::Arc;
use tokio::sync::Mutex;
use futures::{SinkExt, StreamExt, stream::SplitSink};
use warp::ws::Message;
use tokio::sync::broadcast;

pub async fn serve_timer_ws(ws: warp::ws::WebSocket, mut rx: broadcast::Receiver<data::TimerData>) {
    let (mut tx, _) = ws.split();

    let fut = async move {
        loop {
            let r : data::TimerData = rx.recv().await.expect("Expected a Time");
            let m = serde_json::to_string(&r).map(Message::text).expect("Expected a message");

            if !tx.send(m).await.is_ok() {
                return
            }
        }
    };

    tokio::task::spawn(fut);
}

async fn convert_and_send(tx: &mut SplitSink<warp::ws::WebSocket, Message>, r : data::RunTuple) -> bool {
    let m = serde_json::to_string(&r).map(Message::text).expect("Expected a message");
    tx.send(m).await.is_ok()
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