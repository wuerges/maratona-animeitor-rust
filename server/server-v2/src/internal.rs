//! The internal API, per `doc/event-api.md`.
//!
//! All endpoints are private: HTTP Basic authentication with the token
//! configured at startup (`--internal-token`). Responses use the
//! `{ data, errors, warnings }` envelope.

use std::future::{Ready, ready};

use actix_web::*;
use actix_web::dev::Payload;
use actix_web::http::StatusCode;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::Deserialize;
use serde::Serialize;

use service::event_store::{ContestConfig, EventState, EventStore, Run, SiteConfig, StoreError};

/// The token that unlocks the internal API; `None` disables it entirely.
#[derive(Debug, Clone)]
pub struct InternalToken(pub Option<String>);

/// Extractor: rejects requests without valid Basic credentials.
///
/// The username is ignored; the password must be the configured token.
pub struct InternalAuth;

impl FromRequest for InternalAuth {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let configured = req
            .app_data::<web::Data<InternalToken>>()
            .map(|token| token.0.clone())
            .flatten();

        let authorized = match configured {
            None => false,
            Some(expected) => basic_password(req).is_some_and(|found| found == expected),
        };

        if authorized {
            ready(Ok(InternalAuth))
        } else {
            ready(Err(actix_web::error::InternalError::from_response(
                "unauthorized",
                unauthorized_response(),
            )
            .into()))
        }
    }
}

fn basic_password(req: &HttpRequest) -> Option<String> {
    let header = req
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    let encoded = header.strip_prefix("Basic ")?;
    let decoded = BASE64.decode(encoded).ok()?;
    let decoded = String::from_utf8(decoded).ok()?;
    let (_, password) = decoded.split_once(':')?;
    Some(password.to_string())
}

fn unauthorized_response() -> HttpResponse {
    HttpResponse::Unauthorized()
        .insert_header((actix_web::http::header::WWW_AUTHENTICATE, "Basic"))
        .json(serde_json::json!({
            "errors": [ { "code": "unauthorized", "message": "credenciais ausentes ou inválidas" } ]
        }))
}

fn data_json(value: impl Serialize, status: StatusCode) -> HttpResponse {
    crate::envelope::data_json(value, status)
}

fn error_json(status: StatusCode, code: &str, message: impl Into<String>) -> HttpResponse {
    crate::envelope::error_json(status, code, message)
}

fn store_error(err: StoreError) -> HttpResponse {
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
fn json_error(
    err: actix_web::error::JsonPayloadError,
    _req: &HttpRequest,
) -> actix_web::Error {
    let message = err.to_string();
    let code = if message.contains("missing field") {
        "missing_field"
    } else {
        "invalid_json"
    };
    actix_web::error::InternalError::from_response(
        err,
        error_json(StatusCode::BAD_REQUEST, code, message),
    )
    .into()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.app_data(web::JsonConfig::default().error_handler(json_error))
        .service(create_event)
        .service(get_event)
        .service(put_event)
        .service(delete_event)
        .service(patch_time)
        .service(post_runs)
        .service(delete_runs)
        .service(post_event_salt)
        .service(create_contest)
        .service(put_contest)
        .service(delete_contest)
        .service(post_contest_salt)
        .service(create_site)
        .service(put_site)
        .service(delete_site)
        .service(post_site_salt);
}

/// Whether an event/contest/site name is valid as a path segment.
fn valid_name(name: &str) -> bool {
    !name.is_empty()
}

// Events

#[post("/events/{event_name}")]
async fn create_event(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<String>,
    body: web::Json<EventState>,
) -> HttpResponse {
    let event_name = path.into_inner();
    if !valid_name(&event_name) {
        return error_json(StatusCode::NOT_FOUND, "not_found", "evento inexistente");
    }
    let state = body.into_inner();
    match store.create_event(&event_name, state.clone()).await {
        Ok(()) => data_json(state, StatusCode::CREATED),
        Err(err) => store_error(err),
    }
}

#[get("/events/{event_name}")]
async fn get_event(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<String>,
) -> HttpResponse {
    let event_name = path.into_inner();
    match store.get_event(&event_name).await {
        Some(state) => data_json(state, StatusCode::OK),
        None => error_json(StatusCode::NOT_FOUND, "not_found", "evento não existe"),
    }
}

#[put("/events/{event_name}")]
async fn put_event(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<String>,
    body: web::Json<EventState>,
) -> HttpResponse {
    let event_name = path.into_inner();
    let state = body.into_inner();
    match store.put_event(&event_name, state.clone()).await {
        Ok(()) => data_json(state, StatusCode::OK),
        Err(err) => store_error(err),
    }
}

#[delete("/events/{event_name}")]
async fn delete_event(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<String>,
) -> HttpResponse {
    let event_name = path.into_inner();
    if store.delete_event(&event_name).await {
        HttpResponse::NoContent().finish()
    } else {
        error_json(StatusCode::NOT_FOUND, "not_found", "evento não existe")
    }
}

#[derive(Deserialize)]
struct TimeBody {
    time_seconds: i64,
}

#[patch("/events/{event_name}/time")]
async fn patch_time(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<String>,
    body: web::Json<TimeBody>,
) -> HttpResponse {
    let event_name = path.into_inner();
    // Negative values are allowed: the contest starts with a countdown.
    match store.patch_time(&event_name, body.time_seconds).await {
        Some(seconds) => data_json(
            serde_json::json!({ "time_seconds": seconds }),
            StatusCode::OK,
        ),
        None => error_json(StatusCode::NOT_FOUND, "not_found", "evento não existe"),
    }
}

#[derive(Deserialize)]
struct RunsBody {
    runs: Vec<Run>,
}

#[post("/events/{event_name}/runs")]
async fn post_runs(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<String>,
    body: web::Json<RunsBody>,
) -> HttpResponse {
    let event_name = path.into_inner();
    match store.add_runs(&event_name, body.into_inner().runs).await {
        Ok((added, updated)) => data_json(
            serde_json::json!({ "added": added, "updated": updated }),
            StatusCode::OK,
        ),
        Err(err) => store_error(err),
    }
}

#[delete("/events/{event_name}/runs")]
async fn delete_runs(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<String>,
) -> HttpResponse {
    let event_name = path.into_inner();
    if store.clear_runs(&event_name).await {
        HttpResponse::NoContent().finish()
    } else {
        error_json(StatusCode::NOT_FOUND, "not_found", "evento não existe")
    }
}

#[derive(Deserialize)]
struct SaltBody {
    salt: String,
}

#[post("/events/{event_name}/salt")]
async fn post_event_salt(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<String>,
    body: Option<web::Json<SaltBody>>,
) -> HttpResponse {
    let event_name = path.into_inner();
    let salt = body.map(|body| body.into_inner().salt);
    match store.set_event_salt(&event_name, salt).await {
        Ok(salt) => data_json(serde_json::json!({ "salt": salt }), StatusCode::OK),
        Err(err) => store_error(err),
    }
}

// Contests

#[post("/contests/{event_name}/{contest_name:[^/]*}")]
async fn create_contest(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<(String, String)>,
    body: web::Json<ContestConfig>,
) -> HttpResponse {
    let (event_name, contest_name) = path.into_inner();
    let config = body.into_inner();
    match store
        .create_contest(&event_name, &contest_name, config.clone())
        .await
    {
        Ok(()) => data_json(config, StatusCode::CREATED),
        Err(err) => store_error(err),
    }
}

#[put("/contests/{event_name}/{contest_name:[^/]*}")]
async fn put_contest(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<(String, String)>,
    body: web::Json<ContestConfig>,
) -> HttpResponse {
    let (event_name, contest_name) = path.into_inner();
    let config = body.into_inner();
    match store
        .put_contest(&event_name, &contest_name, config.clone())
        .await
    {
        Ok(()) => data_json(config, StatusCode::OK),
        Err(err) => store_error(err),
    }
}

#[delete("/contests/{event_name}/{contest_name:[^/]*}")]
async fn delete_contest(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (event_name, contest_name) = path.into_inner();
    if store.delete_contest(&event_name, &contest_name).await {
        HttpResponse::NoContent().finish()
    } else {
        error_json(StatusCode::NOT_FOUND, "not_found", "evento ou contest não existe")
    }
}

#[post("/contests/{event_name}/{contest_name:[^/]*}/salt")]
async fn post_contest_salt(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<(String, String)>,
    body: Option<web::Json<SaltBody>>,
) -> HttpResponse {
    let (event_name, contest_name) = path.into_inner();
    let salt = body.map(|body| body.into_inner().salt);
    match store
        .set_contest_salt(&event_name, &contest_name, salt)
        .await
    {
        Ok(salt) => data_json(serde_json::json!({ "salt": salt }), StatusCode::OK),
        Err(err) => store_error(err),
    }
}

// Sites

#[post("/sites/{event_name}/{contest_name:[^/]*}/{site_name:[^/]*}")]
async fn create_site(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<(String, String, String)>,
    body: web::Json<SiteConfig>,
) -> HttpResponse {
    let (event_name, contest_name, site_name) = path.into_inner();
    if !valid_name(&site_name) {
        return error_json(StatusCode::NOT_FOUND, "not_found", "site inexistente");
    }
    let config = body.into_inner();
    match store
        .create_site(&event_name, &contest_name, &site_name, config.clone())
        .await
    {
        Ok(()) => data_json(config, StatusCode::CREATED),
        Err(err) => store_error(err),
    }
}

#[put("/sites/{event_name}/{contest_name:[^/]*}/{site_name:[^/]*}")]
async fn put_site(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<(String, String, String)>,
    body: web::Json<SiteConfig>,
) -> HttpResponse {
    let (event_name, contest_name, site_name) = path.into_inner();
    if !valid_name(&site_name) {
        return error_json(StatusCode::NOT_FOUND, "not_found", "site inexistente");
    }
    let config = body.into_inner();
    match store
        .put_site(&event_name, &contest_name, &site_name, config.clone())
        .await
    {
        Ok(()) => data_json(config, StatusCode::OK),
        Err(err) => store_error(err),
    }
}

#[delete("/sites/{event_name}/{contest_name:[^/]*}/{site_name:[^/]*}")]
async fn delete_site(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<(String, String, String)>,
) -> HttpResponse {
    let (event_name, contest_name, site_name) = path.into_inner();
    if store
        .delete_site(&event_name, &contest_name, &site_name)
        .await
    {
        HttpResponse::NoContent().finish()
    } else {
        error_json(StatusCode::NOT_FOUND, "not_found", "evento, contest ou site não existe")
    }
}

#[post("/sites/{event_name}/{contest_name:[^/]*}/{site_name:[^/]*}/salt")]
async fn post_site_salt(
    _auth: InternalAuth,
    store: web::Data<EventStore>,
    path: web::Path<(String, String, String)>,
    body: Option<web::Json<SaltBody>>,
) -> HttpResponse {
    let (event_name, contest_name, site_name) = path.into_inner();
    let salt = body.map(|body| body.into_inner().salt);
    match store
        .set_site_salt(&event_name, &contest_name, &site_name, salt)
        .await
    {
        Ok(salt) => data_json(serde_json::json!({ "salt": salt }), StatusCode::OK),
        Err(err) => store_error(err),
    }
}
