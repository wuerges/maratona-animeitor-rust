//! The public API, per `doc/public-api.md`.
//!
//! Mirrors the internal hierarchy under `/api`: `events → contests → sites`.
//! No authentication except `runs_secret` (site key via Bearer header).

use actix_web::*;

use service::event_store::{EventStore, PublicTimer};

use crate::envelope::{data_json, invalid_key, not_found, send_json};
use crate::remote_control::relay_remote_control;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_events)
        .service(get_contest_state)
        .service(get_config)
        .service(runs_ws)
        .service(get_runs_secret)
        .service(timer_ws)
        .service(remote_control_ws);
}

/// The site key sent in the `Authorization` header.
fn bearer_key(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get(actix_web::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::to_string)
}

#[get("/events")]
async fn list_events(store: web::Data<EventStore>) -> HttpResponse {
    data_json(store.list_events().await, actix_web::http::StatusCode::OK)
}

#[get("/events/{event_name}/contests/{contest_name:[^/]*}/contest")]
async fn get_contest_state(
    store: web::Data<EventStore>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (event_name, contest_name) = path.into_inner();
    match store.public_state(&event_name, &contest_name).await {
        Some(state) => data_json(state, actix_web::http::StatusCode::OK),
        None => not_found("evento ou contest não existe"),
    }
}

#[get("/events/{event_name}/contests/{contest_name:[^/]*}/config")]
async fn get_config(
    store: web::Data<EventStore>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (event_name, contest_name) = path.into_inner();
    match store.public_config(&event_name, &contest_name).await {
        Some(config) => data_json(config, actix_web::http::StatusCode::OK),
        None => not_found("evento ou contest não existe"),
    }
}

#[get("/events/{event_name}/contests/{contest_name:[^/]*}/runs_ws")]
async fn runs_ws(
    store: web::Data<EventStore>,
    path: web::Path<(String, String)>,
    req: HttpRequest,
    body: web::Payload,
) -> Result<HttpResponse, Error> {
    let (event_name, contest_name) = path.into_inner();

    // The replay carries every run since event creation; the client applies
    // the freeze. Filtering happens here by the contest codes.
    let Some(codes) = store.contest_codes(&event_name, &contest_name).await else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let Some(mut runs_rx) = store.subscribe_runs(&event_name).await else {
        return Ok(HttpResponse::NotFound().finish());
    };

    let (response, mut session, _msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        loop {
            match runs_rx.recv().await {
                Ok(run) => {
                    if codes.is_match(&run.team_login) && !send_json(&mut session, &run).await {
                        tracing::debug!("ws connection closed");
                        break;
                    }
                }
                Err(err) => {
                    tracing::warn!(?err, "recv failed");
                    break;
                }
            }
        }
    });

    Ok(response)
}

#[get("/events/{event_name}/contests/{contest_name:[^/]*}/runs_secret")]
async fn get_runs_secret(
    store: web::Data<EventStore>,
    path: web::Path<(String, String)>,
    req: HttpRequest,
) -> HttpResponse {
    let (event_name, contest_name) = path.into_inner();

    // The key never travels in the URL (avoids leaking into access logs).
    let Some(key) = bearer_key(&req) else {
        return invalid_key("chave do site ausente");
    };

    match store.site_by_key(&event_name, &contest_name, &key).await {
        None => invalid_key("chave não casa com nenhum site do contest"),
        Some((site_name, _)) => match store.site_runs(&event_name, &contest_name, &site_name).await {
            Some(runs) => data_json(
                serde_json::json!({ "runs": runs }),
                actix_web::http::StatusCode::OK,
            ),
            None => not_found("evento, contest ou site não existe"),
        },
    }
}

#[get("/events/{event_name}/timer")]
async fn timer_ws(
    store: web::Data<EventStore>,
    path: web::Path<String>,
    req: HttpRequest,
    body: web::Payload,
) -> Result<HttpResponse, Error> {
    let event_name = path.into_inner();

    let Some(current) = store.current_timer(&event_name).await else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let Some(mut time_rx) = store.subscribe_timer(&event_name).await else {
        return Ok(HttpResponse::NotFound().finish());
    };

    let (response, mut session, _msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        // The current value is sent immediately; the stream keeps it fresh,
        // suppressing consecutive duplicates.
        let mut last: Option<PublicTimer> = None;
        loop {
            let time = match &last {
                None => current,
                Some(_) => match time_rx.recv().await {
                    Ok(time) => time,
                    Err(err) => {
                        tracing::warn!(?err, "recv failed");
                        break;
                    }
                },
            };
            if last.is_some_and(|previous| previous == time) {
                continue;
            }
            last = Some(time);
            if !send_json(&mut session, &time).await {
                tracing::debug!("ws connection closed");
                break;
            }
        }
    });

    Ok(response)
}

#[get("/events/{event_name}/contests/{contest_name:[^/]*}/remote_control/{key}")]
async fn remote_control_ws(
    store: web::Data<EventStore>,
    path: web::Path<(String, String, String)>,
    req: HttpRequest,
    body: web::Payload,
) -> Result<HttpResponse, Error> {
    let (event_name, contest_name, key) = path.into_inner();
    let Some(sender) = store
        .remote_control_sender(&event_name, &contest_name, &key)
        .await
    else {
        return Ok(HttpResponse::NotFound().finish());
    };
    relay_remote_control(sender, req, body).await
}
