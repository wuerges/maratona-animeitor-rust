use crate::dataio::DB;
use data::{RunTuple, Answer};

use std::sync::Arc;
use tokio::sync::Mutex;
use futures::{SinkExt, StreamExt, stream::SplitSink};
use warp::ws::Message;
use warp::{Filter, Reply};
use warp::filters::BoxedFilter;
use crate::membroadcast;
use tokio::sync::broadcast;
use crate::routes;

pub fn serve_all_runs(shared_db: Arc<Mutex<DB>>, runs_tx: Arc<membroadcast::Sender::<RunTuple>>) -> BoxedFilter<(impl Reply,)> {
    warp::ws()
        .and(routes::with_db(shared_db.clone()))
        .and(warp::any().map(move || runs_tx.clone()))
        .map(|ws: warp::ws::Ws, db, tx| ws.on_upgrade(move |ws| serve_all_runs_ws(ws, db, tx)))
        .boxed()
}


async fn convert_and_send(tx: &mut SplitSink<warp::ws::WebSocket, Message>, r : RunTuple) -> bool {
    let m = serde_json::to_string(&r).map(Message::text).expect("Expected a message");
    tx.send(m).await.is_ok()
}

async fn serve_all_runs_ws(
    ws: warp::ws::WebSocket,
    runs: Arc<Mutex<DB>>,
    runs_tx: Arc<membroadcast::Sender::<RunTuple>>,
) {
    let mut rx = runs_tx.subscribe();
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
    use super::*;

    #[tokio::test]
    async fn test_serve_timer_ws() {
        let (orig_runs_tx, _) : (membroadcast::Sender::<RunTuple>, _) = membroadcast::channel(1000000);
        let runs_tx = Arc::new(orig_runs_tx);
        let send_runs_tx = runs_tx.clone();

        let db = Arc::new(Mutex::new(DB::empty()));

        let runs = warp::path("allruns_ws").and(serve_all_runs(db.clone(), runs_tx));

        let run1 = RunTuple::new(1, 1, "team1".to_string(), "A".to_string(), Answer::Yes(1));
        let run2 = RunTuple::new(2, 2, "team1".to_string(), "B".to_string(), Answer::Yes(2));

        let expected1 = Message::text(serde_json::to_string(&run1).unwrap());
        let expected2 = Message::text(serde_json::to_string(&run2).unwrap());

        let mut client1 = warp::test::ws()
            .path("/allruns_ws")
            .handshake(runs.clone())
            .await
            .expect("handshake");

        send_runs_tx.send_memo(run1);
        send_runs_tx.send_memo(run2);

        assert_eq!(client1.recv().await.expect("to receive message"), expected1);
        assert_eq!(client1.recv().await.expect("to receive message"), expected2);
    }
}