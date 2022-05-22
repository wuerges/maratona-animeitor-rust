use crate::dataio::DB;

use std::sync::Arc;
use tokio::sync::Mutex;
use futures::{SinkExt, StreamExt, stream::SplitSink};
use warp::ws::Message;
use warp::{Filter, Reply};
use warp::filters::BoxedFilter;
use tokio::sync::broadcast;
use crate::routes;

pub fn serve_all_runs(shared_db: Arc<Mutex<DB>>, runs_tx: broadcast::Sender::<data::RunTuple>) -> BoxedFilter<(impl Reply,)> {
    warp::ws()
        .and(routes::with_db(shared_db.clone()))
        .and(warp::any().map(move || runs_tx.subscribe()))
        .map(|ws: warp::ws::Ws, db, tx| ws.on_upgrade(move |ws| serve_all_runs_ws(ws, db, tx)))
        .boxed()
}


async fn convert_and_send(tx: &mut SplitSink<warp::ws::WebSocket, Message>, r : data::RunTuple) -> bool {
    let m = serde_json::to_string(&r).map(Message::text).expect("Expected a message");
    tx.send(m).await.is_ok()
}

async fn serve_all_runs_ws(
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


#[cfg(test)]
mod tests {
    // use tokio::sync::broadcast;
    // use warp::Filter;
    // use super::serve_runs;
    // use warp::filters::ws::Message;
    // use serde_json;

    // #[tokio::test]
    // async fn test_serve_timer_ws() {
    //     let (time_tx, _) : (broadcast::Sender::<TimerData>, _) = broadcast::channel(1000000);
    //     let send_time_tx = time_tx.clone();

    //     let timer = warp::path("timer").and(serve_timer_ws_filter(time_tx));

    //     let expected1 = Message::text(serde_json::to_string(&TimerData::new(1, 2)).unwrap());
    //     let expected2 = Message::text(serde_json::to_string(&TimerData::new(2, 2)).unwrap());

    //     let mut client = warp::test::ws()
    //         .path("/timer")
    //         .handshake(timer.clone())
    //         .await
    //         .expect("handshake");

    //     {
    //         let mut client2 = warp::test::ws()
    //             .path("/timer")
    //             .handshake(timer)
    //             .await
    //             .expect("handshake");

    //         send_time_tx.send(TimerData::new(1, 2)).expect("to send message");

    //         assert_eq!(client2.recv().await.expect("to receive message"), expected1);
    //     }

    //     assert_eq!(client.recv().await.expect("to receive message"), expected1);

    //     send_time_tx.send(TimerData::new(2, 2)).expect("to send message");

    //     assert_eq!(client.recv().await.expect("to receive message"), expected2);
    // }
}