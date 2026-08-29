//! The `{ data, errors, warnings }` response envelope shared by the APIs.

use axum::Json;
use axum::extract::ws::{Message, WebSocket};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::sink::SinkExt;
use futures::stream::SplitSink;
use serde::Serialize;

/// Serialize a value and send it over a websocket.
/// Returns false when the connection closed and the caller should stop.
pub(crate) async fn send_json<T: serde::Serialize>(
    sender: &mut SplitSink<WebSocket, Message>,
    value: &T,
) -> bool {
    match serde_json::to_string(value) {
        Ok(text) => match sender.send(Message::Text(text.into())).await {
            Ok(()) => true,
            Err(_) => false,
        },
        Err(err) => {
            tracing::warn!(?err, "failed serializing");
            true
        }
    }
}

pub(crate) fn data_json(value: impl Serialize, status: StatusCode) -> Response {
    (status, Json(serde_json::json!({ "data": value }))).into_response()
}

pub(crate) fn error_json(status: StatusCode, code: &str, message: impl Into<String>) -> Response {
    (
        status,
        Json(serde_json::json!({
            "errors": [ { "code": code, "message": message.into() } ]
        })),
    )
        .into_response()
}

pub(crate) fn not_found(message: impl Into<String>) -> Response {
    error_json(StatusCode::NOT_FOUND, "not_found", message)
}

pub(crate) fn invalid_key(message: impl Into<String>) -> Response {
    error_json(StatusCode::FORBIDDEN, "invalid_key", message)
}

pub(crate) fn not_started(message: impl Into<String>) -> Response {
    error_json(StatusCode::FORBIDDEN, "not_started", message)
}
