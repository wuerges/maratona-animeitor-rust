use std::time::Duration;

use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_ws::{Message, MessageStream, Session};
use autometrics::autometrics;
use data::remote_control::ControlMessage;
use tokio::sync::broadcast::{
    error::{RecvError, SendError},
    Receiver, Sender,
};
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
async fn send_to_clients(
    rec: &mut Receiver<ControlMessage>,
    session: &mut Session,
) -> Result<(), Error> {
    let message = rec.recv().await?;
    debug!(?message, "send");
    let text = serde_json::to_string(&message)?;

    session.text(text).await?;

    Ok(())
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
    if let Some(message) = stream.recv().await {
        if let Some(control) = get_text(message?)? {
            debug!(?control, "receive");
            sender.send(control)?;
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await
        }
    } else {
        tokio::time::sleep(Duration::from_secs(1)).await
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
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;
    let mut sender = data.remote_control.clone();
    let mut rec = data.remote_control.subscribe();

    actix_web::rt::spawn(async move {
        loop {
            if let Err(_) = send_to_clients(&mut rec, &mut session).await {
                break;
            }
        }
    });

    actix_web::rt::spawn(async move {
        loop {
            if let Err(_) = read_from_clients(&mut msg_stream, &mut sender).await {
                break;
            }
        }
    });

    Ok(response)
}
