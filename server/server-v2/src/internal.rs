//! The internal API, per `doc/event-api.md`.
//!
//! All endpoints are private: HTTP Basic authentication with the token
//! configured at startup (`--internal-token`). Responses use the
//! `{ data, errors, warnings }` envelope.

use axum::Router;
use axum::extract::FromRequestParts;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header, request::Parts};
use axum::http::header::AUTHORIZATION;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post};
use axum::Json;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::Deserialize;

use crate::AppState;
use service::event_store::{ContestConfig, EventState, EventStore, Run, SiteConfig, StoreError};

/// Extractor: rejects requests without valid Basic credentials.
///
/// The username is ignored; the password must be the configured token.
pub struct InternalAuth;

impl FromRequestParts<AppState> for InternalAuth {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let authorized = match &state.internal_token {
            None => false,
            Some(expected) => basic_password(parts).is_some_and(|found| found.as_str() == expected.as_str()),
        };

        if authorized {
            Ok(InternalAuth)
        } else {
            Err(unauthorized_response())
        }
    }
}

fn basic_password(parts: &Parts) -> Option<String> {
    let header = parts.headers.get(AUTHORIZATION)?.to_str().ok()?;
    let encoded = header.strip_prefix("Basic ")?;
    let decoded = BASE64.decode(encoded).ok()?;
    let decoded = String::from_utf8(decoded).ok()?;
    let (_, password) = decoded.split_once(':')?;
    Some(password.to_string())
}

fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Basic")],
        Json(serde_json::json!({
            "errors": [ { "code": "unauthorized", "message": "credenciais ausentes ou inválidas" } ]
        })),
    )
        .into_response()
}

fn data_json(value: impl serde::Serialize, status: StatusCode) -> Response {
    crate::envelope::data_json(value, status)
}

fn error_json(status: StatusCode, code: &str, message: impl Into<String>) -> Response {
    crate::envelope::error_json(status, code, message)
}

fn store_error(err: StoreError) -> Response {
    match err {
        StoreError::AlreadyExists(message) => {
            error_json(StatusCode::CONFLICT, "conflict", message)
        }
        StoreError::NotFound(message) => error_json(StatusCode::NOT_FOUND, "not_found", message),
        StoreError::InvalidValue(message) => {
            error_json(StatusCode::BAD_REQUEST, "invalid_value", message)
        }
        StoreError::InvalidRegex(message) => {
            error_json(StatusCode::BAD_REQUEST, "invalid_regex", message)
        }
    }
}

/// Maps JSON parse failures to the envelope's canonical codes.
fn map_json_rejection(err: JsonRejection) -> Response {
    let message = err.to_string();
    let code = if message.contains("missing field") {
        "missing_field"
    } else if message.contains("unknown variant") {
        // An enum field (e.g. `answer`) with an unknown value: the JSON is
        // well-formed, the value is not.
        "invalid_value"
    } else {
        "invalid_json"
    };
    error_json(StatusCode::BAD_REQUEST, code, message)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/events", get(list_events))
        .route(
            "/events/{event_name}",
            get(get_event).post(create_event).put(put_event).delete(delete_event),
        )
        .route("/events/{event_name}/contests", get(list_contests))
        .route(
            "/events/{event_name}/contests/{contest_name}/sites",
            get(list_sites),
        )
        .route("/events/{event_name}/time", patch(patch_time))
        .route(
            "/events/{event_name}/runs",
            post(post_runs).delete(delete_runs),
        )
        .route("/events/{event_name}/salt", post(post_event_salt))
        .route(
            "/contests/{event_name}/{contest_name}",
            post(create_contest).put(put_contest).delete(delete_contest),
        )
        .route(
            "/contests/{event_name}/{contest_name}/salt",
            post(post_contest_salt),
        )
        .route(
            "/sites/{event_name}/{contest_name}/{site_name}",
            post(create_site).put(put_site).delete(delete_site),
        )
        .route(
            "/sites/{event_name}/{contest_name}/{site_name}/salt",
            post(post_site_salt),
        )
}

/// Whether an event/contest/site name is valid as a path segment.
fn valid_name(name: &str) -> bool {
    !name.is_empty()
}

// Events

async fn create_event(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path(event_name): Path<String>,
    body: Result<Json<EventState>, JsonRejection>,
) -> Response {
    let Json(state) = match body {
        Ok(body) => body,
        Err(err) => return map_json_rejection(err),
    };
    if !valid_name(&event_name) {
        return error_json(StatusCode::NOT_FOUND, "not_found", "evento inexistente");
    }
    match store.create_event(&event_name, state.clone()).await {
        Ok(()) => data_json(state, StatusCode::CREATED),
        Err(err) => store_error(err),
    }
}

async fn get_event(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path(event_name): Path<String>,
) -> Response {
    match store.get_event(&event_name).await {
        Some(state) => data_json(state, StatusCode::OK),
        None => error_json(StatusCode::NOT_FOUND, "not_found", "evento não existe"),
    }
}

/// Lists the names of all events, in creation order.
async fn list_events(_auth: InternalAuth, State(store): State<EventStore>) -> Response {
    data_json(store.list_events().await, StatusCode::OK)
}

/// Lists the contests of an event, with their salts (internal scope).
async fn list_contests(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path(event_name): Path<String>,
) -> Response {
    match store.list_contests(&event_name).await {
        Some(contests) => data_json(contests, StatusCode::OK),
        None => error_json(StatusCode::NOT_FOUND, "not_found", "evento não existe"),
    }
}

/// Lists the sites of a contest, with their salts (internal scope).
async fn list_sites(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event_name, contest_name)): Path<(String, String)>,
) -> Response {
    match store.list_sites(&event_name, &contest_name).await {
        Some(sites) => data_json(sites, StatusCode::OK),
        None => error_json(StatusCode::NOT_FOUND, "not_found", "evento ou contest não existe"),
    }
}

async fn put_event(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path(event_name): Path<String>,
    body: Result<Json<EventState>, JsonRejection>,
) -> Response {
    let Json(state) = match body {
        Ok(body) => body,
        Err(err) => return map_json_rejection(err),
    };
    match store.put_event(&event_name, state.clone()).await {
        Ok(()) => data_json(state, StatusCode::OK),
        Err(err) => store_error(err),
    }
}

async fn delete_event(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path(event_name): Path<String>,
) -> Response {
    if store.delete_event(&event_name).await {
        StatusCode::NO_CONTENT.into_response()
    } else {
        error_json(StatusCode::NOT_FOUND, "not_found", "evento não existe")
    }
}

#[derive(Deserialize)]
struct TimeBody {
    time_seconds: i64,
}

async fn patch_time(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path(event_name): Path<String>,
    body: Result<Json<TimeBody>, JsonRejection>,
) -> Response {
    let Json(body) = match body {
        Ok(body) => body,
        Err(err) => return map_json_rejection(err),
    };
    // Negative values are allowed: the contest starts with a countdown.
    match store.patch_time(&event_name, body.time_seconds).await {
        Some(seconds) => data_json(serde_json::json!({ "time_seconds": seconds }), StatusCode::OK),
        None => error_json(StatusCode::NOT_FOUND, "not_found", "evento não existe"),
    }
}

#[derive(Deserialize)]
struct RunsBody {
    runs: Vec<Run>,
}

async fn post_runs(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path(event_name): Path<String>,
    body: Result<Json<RunsBody>, JsonRejection>,
) -> Response {
    let Json(body) = match body {
        Ok(body) => body,
        Err(err) => return map_json_rejection(err),
    };
    match store.add_runs(&event_name, body.runs).await {
        Ok((added, updated)) => {
            data_json(serde_json::json!({ "added": added, "updated": updated }), StatusCode::OK)
        }
        Err(err) => store_error(err),
    }
}

async fn delete_runs(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path(event_name): Path<String>,
) -> Response {
    if store.clear_runs(&event_name).await {
        StatusCode::NO_CONTENT.into_response()
    } else {
        error_json(StatusCode::NOT_FOUND, "not_found", "evento não existe")
    }
}

#[derive(Deserialize)]
struct SaltBody {
    salt: Option<String>,
}

async fn post_event_salt(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path(event_name): Path<String>,
    body: Result<Option<Json<SaltBody>>, JsonRejection>,
) -> Response {
    let body = match body {
        Ok(body) => body,
        Err(err) => return map_json_rejection(err),
    };
    let salt = body.and_then(|Json(body)| body.salt);
    match store.set_event_salt(&event_name, salt).await {
        Ok(salt) => data_json(serde_json::json!({ "salt": salt }), StatusCode::OK),
        Err(err) => store_error(err),
    }
}

// Contests

async fn create_contest(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event_name, contest_name)): Path<(String, String)>,
    body: Result<Json<ContestConfig>, JsonRejection>,
) -> Response {
    let Json(config) = match body {
        Ok(body) => body,
        Err(err) => return map_json_rejection(err),
    };
    match store
        .create_contest(&event_name, &contest_name, config.clone())
        .await
    {
        Ok(()) => data_json(config, StatusCode::CREATED),
        Err(err) => store_error(err),
    }
}

async fn put_contest(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event_name, contest_name)): Path<(String, String)>,
    body: Result<Json<ContestConfig>, JsonRejection>,
) -> Response {
    let Json(config) = match body {
        Ok(body) => body,
        Err(err) => return map_json_rejection(err),
    };
    match store
        .put_contest(&event_name, &contest_name, config.clone())
        .await
    {
        Ok(()) => data_json(config, StatusCode::OK),
        Err(err) => store_error(err),
    }
}

async fn delete_contest(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event_name, contest_name)): Path<(String, String)>,
) -> Response {
    if store.delete_contest(&event_name, &contest_name).await {
        StatusCode::NO_CONTENT.into_response()
    } else {
        error_json(StatusCode::NOT_FOUND, "not_found", "evento ou contest não existe")
    }
}

async fn post_contest_salt(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event_name, contest_name)): Path<(String, String)>,
    body: Result<Option<Json<SaltBody>>, JsonRejection>,
) -> Response {
    let body = match body {
        Ok(body) => body,
        Err(err) => return map_json_rejection(err),
    };
    let salt = body.and_then(|Json(body)| body.salt);
    match store.set_contest_salt(&event_name, &contest_name, salt).await {
        Ok(salt) => data_json(serde_json::json!({ "salt": salt }), StatusCode::OK),
        Err(err) => store_error(err),
    }
}

// Sites

async fn create_site(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event_name, contest_name, site_name)): Path<(String, String, String)>,
    body: Result<Json<SiteConfig>, JsonRejection>,
) -> Response {
    let Json(config) = match body {
        Ok(body) => body,
        Err(err) => return map_json_rejection(err),
    };
    if !valid_name(&site_name) {
        return error_json(StatusCode::NOT_FOUND, "not_found", "site inexistente");
    }
    match store
        .create_site(&event_name, &contest_name, &site_name, config.clone())
        .await
    {
        Ok(()) => data_json(config, StatusCode::CREATED),
        Err(err) => store_error(err),
    }
}

async fn put_site(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event_name, contest_name, site_name)): Path<(String, String, String)>,
    body: Result<Json<SiteConfig>, JsonRejection>,
) -> Response {
    let Json(config) = match body {
        Ok(body) => body,
        Err(err) => return map_json_rejection(err),
    };
    if !valid_name(&site_name) {
        return error_json(StatusCode::NOT_FOUND, "not_found", "site inexistente");
    }
    match store
        .put_site(&event_name, &contest_name, &site_name, config.clone())
        .await
    {
        Ok(()) => data_json(config, StatusCode::OK),
        Err(err) => store_error(err),
    }
}

async fn delete_site(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event_name, contest_name, site_name)): Path<(String, String, String)>,
) -> Response {
    if store
        .delete_site(&event_name, &contest_name, &site_name)
        .await
    {
        StatusCode::NO_CONTENT.into_response()
    } else {
        error_json(StatusCode::NOT_FOUND, "not_found", "evento, contest ou site não existe")
    }
}

async fn post_site_salt(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event_name, contest_name, site_name)): Path<(String, String, String)>,
    body: Result<Option<Json<SaltBody>>, JsonRejection>,
) -> Response {
    let body = match body {
        Ok(body) => body,
        Err(err) => return map_json_rejection(err),
    };
    let salt = body.and_then(|Json(body)| body.salt);
    match store
        .set_site_salt(&event_name, &contest_name, &site_name, salt)
        .await
    {
        Ok(salt) => data_json(serde_json::json!({ "salt": salt }), StatusCode::OK),
        Err(err) => store_error(err),
    }
}
