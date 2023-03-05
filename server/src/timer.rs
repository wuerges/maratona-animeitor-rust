use data::TimerData;
use futures::{SinkExt, StreamExt};
use metrics::increment_counter;
use tokio::sync::broadcast;
use warp::filters::BoxedFilter;
use warp::ws::Message;
use warp::{Filter, Reply};

pub fn serve_timer(time_tx: broadcast::Sender<TimerData>) -> BoxedFilter<(impl Reply,)> {
    warp::ws()
        .and(warp::any().map(move || time_tx.subscribe()))
        .map(|ws: warp::ws::Ws, tx| ws.on_upgrade(move |ws| serve_timer_ws(ws, tx)))
        .boxed()
}

async fn serve_timer_ws(ws: warp::ws::WebSocket, mut rx: broadcast::Receiver<TimerData>) {
    let (mut tx, _) = ws.split();

    increment_counter!("serve_timer_ws_clients_connected");

    let fut = async move {
        loop {
            let r: TimerData = rx.recv().await.expect("Expected a Time");
            let m = serde_json::to_string(&r)
                .map(Message::text)
                .expect("Expected a message");

            if tx.send(m).await.is_err() {
                return;
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
        let (time_tx, _): (broadcast::Sender<TimerData>, _) = broadcast::channel(1000000);
        let send_time_tx = time_tx.clone();

        let timer = warp::path("timer").and(serve_timer(time_tx));

        let expected1 = Message::text(serde_json::to_string(&TimerData::new(1, 2)).unwrap());
        let expected2 = Message::text(serde_json::to_string(&TimerData::new(2, 2)).unwrap());

        let mut client = warp::test::ws()
            .path("/timer")
            .handshake(timer.clone())
            .await
            .expect("handshake");

        {
            let mut client2 = warp::test::ws()
                .path("/timer")
                .handshake(timer)
                .await
                .expect("handshake");

            send_time_tx
                .send(TimerData::new(1, 2))
                .expect("to send message");

            assert_eq!(client2.recv().await.expect("to receive message"), expected1);
        }

        assert_eq!(client.recv().await.expect("to receive message"), expected1);

        send_time_tx
            .send(TimerData::new(2, 2))
            .expect("to send message");

        assert_eq!(client.recv().await.expect("to receive message"), expected2);
    }
}
