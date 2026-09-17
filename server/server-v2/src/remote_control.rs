use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use data::remote_control::ControlMessage;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast::{Receiver, Sender, error::SendError};
use tokio_stream::wrappers::BroadcastStream;
use tracing::{debug, instrument};

use service::remote_control::{ConnectionControlMessage, ControlSender, next_request_id};

use crate::envelope::send_json;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    SendError(#[from] SendError<ConnectionControlMessage>),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Axum(#[from] axum::Error),
}

#[instrument(skip(rec, sender), err)]
async fn send_to_clients(
    rec: Receiver<ConnectionControlMessage>,
    mut sender: SplitSink<WebSocket, Message>,
    connection_request_id: u64,
) -> Result<(), Error> {
    let mut rec_stream = BroadcastStream::new(rec);

    while let Some(Ok(ConnectionControlMessage {
        request_id,
        message,
    })) = rec_stream.next().await
    {
        if request_id != connection_request_id && !send_json(&mut sender, &message).await {
            return Ok(());
        }
    }

    Ok(sender.send(Message::Close(None)).await?)
}

fn get_text(message: Message) -> Result<Option<ControlMessage>, Error> {
    match message {
        Message::Text(text) => Ok(Some(serde_json::from_str(text.as_str())?)),
        Message::Binary(_) | Message::Ping(_) | Message::Pong(_) | Message::Close(_) => Ok(None),
    }
}

#[instrument(skip(stream, sender), err)]
async fn read_from_clients(
    stream: &mut SplitStream<WebSocket>,
    sender: Sender<ConnectionControlMessage>,
    request_id: u64,
) -> Result<(), Error> {
    while let Some(Ok(raw_message)) = stream.next().await {
        if let Some(message) = get_text(raw_message)? {
            debug!(?message, "receive");
            sender.send(ConnectionControlMessage {
                request_id,
                message,
            })?;
        }
    }

    Ok(())
}

/// Relays control messages between every client of the same sender channel.
pub(crate) async fn relay_remote_control(sender: ControlSender, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| async move {
        let (sender_half, mut receiver_half) = socket.split();
        let rec = sender.subscribe();

        let request_id = next_request_id();
        tracing::info!(?request_id, "established remote control");

        let (send_result, read_result) = tokio::join!(
            send_to_clients(rec, sender_half, request_id),
            read_from_clients(&mut receiver_half, sender, request_id),
        );
        if let Err(err) = send_result {
            tracing::debug!(?err, "failed sending");
        }
        if let Err(err) = read_result {
            tracing::debug!(?err, "failed reading");
        }
    })
}
