use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use data::remote_control::ControlMessage;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast::{Receiver, Sender, error::SendError};
use tokio_stream::wrappers::BroadcastStream;
use tracing::{debug, instrument};

use service::remote_control::{ConnectionControlMessage, RemoteConnection, next_request_id};

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
        if request_id != connection_request_id && !send_remote_json(&mut sender, &message).await {
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
    let mut budget = MessageBudget::new();
    while let Some(Ok(raw_message)) = stream.next().await {
        if !budget.allow() {
            break;
        }
        if let Some(message) = get_text(raw_message)? {
            let valid = match &message {
                ControlMessage::QueryString(q) => q.query.len() <= 2048,
                ControlMessage::PhotoState(data::remote_control::PhotoState::Show(team)) => {
                    team.len() <= 256
                }
                _ => true,
            };
            if !valid {
                break;
            }
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
pub(crate) async fn relay_remote_control(
    mut connection: RemoteConnection,
    ws: WebSocketUpgrade,
    permit: tokio::sync::OwnedSemaphorePermit,
) -> Response {
    ws.max_message_size(4096)
        .max_frame_size(4096)
        .max_write_buffer_size(128 * 1024 + 8192)
        .on_upgrade(move |socket| async move {
            let _permit = permit;
            let sender = connection.sender.clone();
            let (sender_half, mut receiver_half) = socket.split();
            let rec = sender.subscribe();

            let request_id = next_request_id();
            tracing::info!(?request_id, "established remote control");

            // Either half ending must release the other half and its socket.
            let result = tokio::select! {
                result = send_to_clients(rec, sender_half, request_id) => result,
                result = read_from_clients(&mut receiver_half, sender, request_id) => result,
                _ = tokio::time::sleep_until(connection.expires) => Ok(()),
                _ = connection.revoked.changed() => Ok(()),
            };
            if let Err(err) = result {
                tracing::debug!(?err, "remote control disconnected");
            }
        })
}

async fn send_remote_json(
    sender: &mut SplitSink<WebSocket, Message>,
    message: &ControlMessage,
) -> bool {
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        send_json(sender, message),
    )
    .await
    .unwrap_or(false)
}

/// Token bucket: sustained 30 frames/second, initial burst of 60 (including control frames).
pub struct MessageBudget {
    tokens: f64,
    last: std::time::Instant,
}
impl MessageBudget {
    pub fn new() -> Self {
        Self {
            tokens: 60.0,
            last: std::time::Instant::now(),
        }
    }
    pub fn allow(&mut self) -> bool {
        let now = std::time::Instant::now();
        self.tokens = (self.tokens + now.duration_since(self.last).as_secs_f64() * 30.0).min(60.0);
        self.last = now;
        if self.tokens < 1.0 {
            return false;
        }
        self.tokens -= 1.0;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn message_budget_bounds_burst() {
        let mut budget = MessageBudget::new();
        for _ in 0..60 {
            assert!(budget.allow());
        }
        assert!(!budget.allow());
    }
}
