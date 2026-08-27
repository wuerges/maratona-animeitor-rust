//! The `{ data, errors, warnings }` response envelope shared by the APIs.

use actix_web::{HttpResponse, http::StatusCode};
use actix_ws::Closed;
use serde::Serialize;

/// Serialize a value and send it over a websocket.
/// Returns false when the connection closed and the caller should stop.
pub(crate) async fn send_json<T: serde::Serialize>(
    session: &mut actix_ws::Session,
    value: &T,
) -> bool {
    match serde_json::to_string(value) {
        Ok(text) => match session.text(text).await {
            Ok(()) => true,
            Err(Closed) => false,
        },
        Err(err) => {
            tracing::warn!(?err, "failed serializing");
            true
        }
    }
}

pub(crate) fn data_json(value: impl Serialize, status: StatusCode) -> HttpResponse {
    HttpResponse::build(status).json(serde_json::json!({ "data": value }))
}

pub(crate) fn error_json(status: StatusCode, code: &str, message: impl Into<String>) -> HttpResponse {
    HttpResponse::build(status).json(serde_json::json!({
        "errors": [ { "code": code, "message": message.into() } ]
    }))
}

pub(crate) fn not_found(message: impl Into<String>) -> HttpResponse {
    error_json(StatusCode::NOT_FOUND, "not_found", message)
}

pub(crate) fn invalid_key(message: impl Into<String>) -> HttpResponse {
    error_json(StatusCode::FORBIDDEN, "invalid_key", message)
}
