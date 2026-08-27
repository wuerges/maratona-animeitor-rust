use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::{Message, MessageStream, Session};
use data::remote_control::ControlMessage;
use futures::StreamExt;
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
    Closed(#[from] actix_ws::Closed),
}

#[instrument(skip(rec, session), err)]
async fn send_to_clients(
    rec: Receiver<ConnectionControlMessage>,
    mut session: Session,
    connection_request_id: u64,
) -> Result<(), Error> {
    let mut rec_stream = BroadcastStream::new(rec);

    while let Some(Ok(ConnectionControlMessage {
        request_id,
        message,
    })) = rec_stream.next().await
    {
        if request_id != connection_request_id && !send_json(&mut session, &message).await {
            return Ok(());
        }
    }

    Ok(session.close(None).await?)
}

fn get_text(message: Message) -> Result<Option<ControlMessage>, Error> {
    match message {
        Message::Text(text) => Ok(Some(serde_json::from_slice(text.as_bytes())?)),
        Message::Binary(_) => Ok(None),
        Message::Continuation(_) => Ok(None),
        Message::Ping(_) => Ok(None),
        Message::Pong(_) => Ok(None),
        Message::Close(_) => Ok(None),
        Message::Nop => Ok(None),
    }
}

#[instrument(skip(stream, sender), err)]
async fn read_from_clients(
    stream: &mut MessageStream,
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
pub(crate) async fn relay_remote_control(
    sender: ControlSender,
    req: HttpRequest,
    body: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, session, mut msg_stream) = actix_ws::handle(&req, body)?;

    let rec = sender.subscribe();

    let request_id = next_request_id();
    tracing::info!(?request_id, "established remote control");

    actix_web::rt::spawn(async move {
        if let Err(err) = send_to_clients(rec, session, request_id).await {
            tracing::debug!(?err, "failed sending");
        }
    });

    actix_web::rt::spawn(async move {
        if let Err(err) = read_from_clients(&mut msg_stream, sender, request_id).await {
            tracing::debug!(?err, "failed reading");
        }
    });

    Ok(response)
}
