use std::time::Duration;

use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_ws::{Message, MessageStream, Session};
use autometrics::autometrics;
use data::remote_control::ControlMessage;
use futures::StreamExt;
use tokio::sync::broadcast::{
    error::{RecvError, SendError},
    Receiver, Sender,
};
use tokio_stream::wrappers::BroadcastStream;
use tracing::{debug, instrument, Level};

use crate::app_data::AppData;

#[get("/remote_control/{key}")]
async fn remote_control_ws(
    data: web::Data<AppData>,
    req: HttpRequest,
    body: web::Payload,
    key: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    run_remote_control_ws(data, req, body, key.into_inner()).await
}

pub type ControlSender = Sender<ControlMessage>;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    RecvError(#[from] RecvError),
    #[error(transparent)]
    SendError(#[from] SendError<ControlMessage>),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Closed(#[from] actix_ws::Closed),
    #[error(transparent)]
    ProtocolError(#[from] actix_ws::ProtocolError),
}

#[instrument(skip(rec, session), err)]
async fn send_to_clients(rec: Receiver<ControlMessage>, mut session: Session) -> Result<(), Error> {
    let mut rec_stream = BroadcastStream::new(rec);

    while let Some(Ok(message)) = rec_stream.next().await {
        let text = serde_json::to_string(&message)?;
        session.text(text).await?;
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
    sender: &mut Sender<ControlMessage>,
) -> Result<(), Error> {
    while let Some(Ok(message)) = stream.next().await {
        if let Some(control) = get_text(message)? {
            debug!(?control, "receive");
            sender.send(control)?;
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await
        }
    }

    Ok(())
}

#[autometrics]
#[tracing::instrument(level = Level::DEBUG, skip(data, body), ret)]
async fn run_remote_control_ws(
    data: web::Data<AppData>,
    req: HttpRequest,
    body: web::Payload,
    key: String,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, session, mut msg_stream) = actix_ws::handle(&req, body)?;
    let mut sender = data.remote_control.clone();
    let rec = data.remote_control.subscribe();

    actix_web::rt::spawn(async move {
        if let Err(err) = send_to_clients(rec, session).await {
            tracing::debug!(?err, "failed sending");
        }
    });

    actix_web::rt::spawn(async move {
        if let Err(err) = read_from_clients(&mut msg_stream, &mut sender).await {
            tracing::debug!(?err, "failed reading");
        }
    });

    Ok(response)
}
