//! The internal API, per `doc/event-api.md`.
//!
//! All endpoints are private: HTTP Basic authentication with the token
//! configured in the server configuration. Responses use the
//! `{ data, errors, warnings }` envelope.

mod incremental;

use axum::Json;
use axum::Router;
use axum::extract::FromRequestParts;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::header::AUTHORIZATION;
use axum::http::{StatusCode, header, request::Parts};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::Deserialize;
use utoipa::OpenApi;

use autometrics::autometrics;

use crate::AppState;
use service::event_store::{ContestConfig, EventState, EventStore, Run, SiteConfig, StoreError};

/// Extractor: rejects requests without valid Basic credentials.
///
/// Both the username and its configured token must match.
pub struct InternalAuth;

impl FromRequestParts<AppState> for InternalAuth {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let authorized = basic_credentials(parts).is_some_and(|(name, password)| {
            state
                .internal_tokens
                .get(&name)
                .is_some_and(|expected| expected == &password)
        });

        if authorized {
            Ok(InternalAuth)
        } else {
            Err(unauthorized_response())
        }
    }
}

fn basic_credentials(parts: &Parts) -> Option<(String, String)> {
    let header = parts.headers.get(AUTHORIZATION)?.to_str().ok()?;
    let encoded = header.strip_prefix("Basic ")?;
    let decoded = BASE64.decode(encoded).ok()?;
    let decoded = String::from_utf8(decoded).ok()?;
    let (name, password) = decoded.split_once(':')?;
    Some((name.to_string(), password.to_string()))
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
        StoreError::Conflict(message) | StoreError::AlreadyExists(message) => {
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
        .route("/openapi.json", get(internal_openapi_json))
        .route("/docs", get(internal_openapi_docs))
        .route("/events", get(list_events))
        .route(
            "/events/{event_name}/revelation_urls",
            get(list_revelation_urls),
        )
        .route(
            "/events/{event_name}",
            get(get_event)
                .post(create_event)
                .put(put_event)
                .patch(incremental::patch_event)
                .delete(delete_event),
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
            get(incremental::get_contest)
                .post(create_contest)
                .put(put_contest)
                .patch(incremental::patch_contest)
                .delete(delete_contest),
        )
        .route(
            "/contests/{event_name}/{contest_name}/salt",
            post(post_contest_salt),
        )
        .route(
            "/sites/{event_name}/{contest_name}/{site_name}",
            get(incremental::get_site)
                .post(create_site)
                .put(put_site)
                .patch(incremental::patch_site)
                .delete(delete_site),
        )
        .route(
            "/sites/{event_name}/{contest_name}/{site_name}/salt",
            post(post_site_salt),
        )
        .route("/events/{event_name}/teams", post(incremental::add_team))
        .route(
            "/events/{event_name}/teams/{login}",
            get(incremental::get_team)
                .patch(incremental::patch_team)
                .delete(incremental::remove_team),
        )
        .route(
            "/events/{event_name}/problems",
            post(incremental::add_problem),
        )
        .route(
            "/events/{event_name}/problems/{problem}",
            axum::routing::delete(incremental::remove_problem),
        )
        .route(
            "/contests/{event_name}/{contest_name}/codes",
            patch(incremental::patch_contest_codes),
        )
        .route(
            "/sites/{event_name}/{contest_name}/{site_name}/codes",
            patch(incremental::patch_site_codes),
        )
        .route("/metrics", get(get_metrics))
}

async fn internal_openapi_json(_auth: InternalAuth) -> Response {
    axum::Json(crate::openapi::InternalApiDoc::openapi()).into_response()
}

async fn internal_openapi_docs(_auth: InternalAuth) -> Response {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        crate::openapi::swagger_html("/internal/openapi.json"),
    )
        .into_response()
}

/// Whether an event/contest/site name is valid as a path segment.
fn valid_name(name: &str) -> bool {
    !name.is_empty()
}

/// Prometheus metrics (autometrics), behind the internal token.
#[autometrics]
async fn get_metrics(_auth: InternalAuth) -> Response {
    let (status, text) = crate::metrics::get_metrics().await;
    (status, text).into_response()
}

// Events

#[autometrics]
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

#[autometrics]
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
#[autometrics]
async fn list_events(_auth: InternalAuth, State(store): State<EventStore>) -> Response {
    data_json(store.list_events().await, StatusCode::OK)
}

/// Lists the contests of an event, with their salts (internal scope).
#[autometrics]
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
#[autometrics]
async fn list_sites(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event_name, contest_name)): Path<(String, String)>,
) -> Response {
    match store.list_sites(&event_name, &contest_name).await {
        Some(sites) => data_json(sites, StatusCode::OK),
        None => error_json(
            StatusCode::NOT_FOUND,
            "not_found",
            "evento ou contest não existe",
        ),
    }
}

#[autometrics]
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

#[autometrics]
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

#[derive(Deserialize, utoipa::ToSchema)]
pub(crate) struct TimeBody {
    /// Required elapsed seconds. Negative means countdown; zero opens public contest endpoints.
    time_seconds: i64,
}

#[autometrics]
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
        Some(seconds) => data_json(
            serde_json::json!({ "time_seconds": seconds }),
            StatusCode::OK,
        ),
        None => error_json(StatusCode::NOT_FOUND, "not_found", "evento não existe"),
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub(crate) struct RunsBody {
    /// Submissions to ingest; may be empty. Existing IDs are corrected, not duplicated.
    runs: Vec<Run>,
}

#[autometrics]
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
        Ok((added, updated, ignored)) => {
            // Ignored runs (unknown teams, e.g. judge users of the MOJ
            // feed) are reported as warnings, not errors: the batch
            // succeeded.
            let warnings: Vec<data::event::ErrorEntry> = ignored
                .into_iter()
                .map(|run| data::event::ErrorEntry {
                    code: "unknown_team".into(),
                    message: format!(
                        "run {} do time {} ignorada: o time não pertence ao evento",
                        run.id, run.team_login
                    ),
                })
                .collect();
            if warnings.is_empty() {
                data_json(
                    serde_json::json!({ "added": added, "updated": updated }),
                    StatusCode::OK,
                )
            } else {
                crate::envelope::data_json_with_warnings(
                    serde_json::json!({ "added": added, "updated": updated }),
                    warnings,
                    StatusCode::OK,
                )
            }
        }
        Err(err) => store_error(err),
    }
}

#[autometrics]
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

#[derive(Deserialize, utoipa::ToSchema)]
pub(crate) struct SaltBody {
    /// Omitted, null, or empty generates a random salt; nonempty sets the supplied value.
    salt: Option<String>,
}

#[autometrics]
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

#[autometrics]
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

#[autometrics]
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

#[autometrics]
async fn delete_contest(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event_name, contest_name)): Path<(String, String)>,
) -> Response {
    if store.delete_contest(&event_name, &contest_name).await {
        StatusCode::NO_CONTENT.into_response()
    } else {
        error_json(
            StatusCode::NOT_FOUND,
            "not_found",
            "evento ou contest não existe",
        )
    }
}

#[autometrics]
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
    match store
        .set_contest_salt(&event_name, &contest_name, salt)
        .await
    {
        Ok(salt) => data_json(serde_json::json!({ "salt": salt }), StatusCode::OK),
        Err(err) => store_error(err),
    }
}

// Sites

#[autometrics]
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

#[autometrics]
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

#[autometrics]
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
        error_json(
            StatusCode::NOT_FOUND,
            "not_found",
            "evento, contest ou site não existe",
        )
    }
}

#[autometrics]
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

/// Ready-to-use private frontend links for all sites of an event.
async fn list_revelation_urls(
    auth: Result<InternalAuth, Response>,
    State(state): State<AppState>,
    Path(event_name): Path<String>,
) -> Response {
    let mut response = match auth {
        Err(response) => response,
        Ok(_) => match state
            .store
            .revelation_urls(&event_name, &state.public_url)
            .await
        {
            Some(urls) => data_json(urls, StatusCode::OK),
            None => error_json(StatusCode::NOT_FOUND, "not_found", "evento não existe"),
        },
    };
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    response
}
