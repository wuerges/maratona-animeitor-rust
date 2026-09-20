//! The public API, per `doc/public-api.md`.
//!
//! Mirrors the internal hierarchy under `/api`: `events → contests → sites`.
//! No authentication except `runs_secret` (site key via Bearer header).

use axum::Router;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use futures::StreamExt;
use utoipa::OpenApi;

use autometrics::autometrics;

use crate::AppState;
use service::event_store::{EventStore, PublicTimer};

use crate::envelope::{data_json, invalid_key, not_found, not_started, send_json};
use crate::remote_control::relay_remote_control;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/openapi.json", get(openapi_json))
        .route("/docs", get(openapi_docs))
        .route("/events", get(list_events))
        .route("/events/{event_name}/contests", get(list_contests))
        .route(
            "/events/{event_name}/contests/{contest_name}/contest",
            get(get_contest_state),
        )
        .route(
            "/events/{event_name}/contests/{contest_name}/config",
            get(get_config),
        )
        .route(
            "/events/{event_name}/contests/{contest_name}/runs_ws",
            get(runs_ws),
        )
        .route(
            "/events/{event_name}/contests/{contest_name}/runs_secret",
            get(get_runs_secret),
        )
        .route("/events/{event_name}/timer", get(timer_ws))
        .route(
            "/events/{event_name}/contests/{contest_name}/remote_control/{key}",
            get(remote_control_ws),
        )
}

async fn openapi_json() -> Response {
    axum::Json(crate::openapi::PublicApiDoc::openapi()).into_response()
}

async fn openapi_docs() -> Response {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        crate::openapi::swagger_html("/api/openapi.json"),
    )
        .into_response()
}

/// The site key sent in the `Authorization` header.
fn bearer_key(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::to_string)
}

#[autometrics]
async fn list_events(State(store): State<EventStore>) -> Response {
    data_json(
        crate::store_call!(store.list_events().await),
        StatusCode::OK,
    )
}

/// Lists contest names before and after start so the landing page can link
/// to upcoming contests and their countdown screens.
#[autometrics]
async fn list_contests(
    State(store): State<EventStore>,
    Path(event_name): Path<String>,
) -> Response {
    match crate::store_call!(store.list_contests(&event_name).await) {
        Some(contests) => {
            let mut names: Vec<String> = contests.into_iter().map(|config| config.name).collect();
            names.sort();
            data_json(names, StatusCode::OK)
        }
        None => not_found("evento não existe"),
    }
}

/// Contest state, configuration, and runs remain gated until start.
/// Event/contest discovery and the timer remain available for the landing
/// page and countdown.
#[autometrics]
async fn contest_gate(store: &EventStore, event_name: &str) -> Result<(), Response> {
    match store
        .is_started(event_name)
        .await
        .map_err(crate::internal::store_error)?
    {
        None => Err(not_found("evento ou contest não existe")),
        Some(false) => Err(not_started("o evento ainda não começou")),
        Some(true) => Ok(()),
    }
}

#[autometrics]
async fn get_contest_state(
    State(store): State<EventStore>,
    Path((event_name, contest_name)): Path<(String, String)>,
) -> Response {
    if let Err(response) = contest_gate(&store, &event_name).await {
        return response;
    }
    match crate::store_call!(store.public_state(&event_name, &contest_name).await) {
        Some(state) => data_json(state, StatusCode::OK),
        None => not_found("evento ou contest não existe"),
    }
}

#[autometrics]
async fn get_config(
    State(store): State<EventStore>,
    Path((event_name, contest_name)): Path<(String, String)>,
) -> Response {
    if let Err(response) = contest_gate(&store, &event_name).await {
        return response;
    }
    match crate::store_call!(store.public_config(&event_name, &contest_name).await) {
        Some(config) => data_json(config, StatusCode::OK),
        None => not_found("evento ou contest não existe"),
    }
}

#[autometrics]
async fn runs_ws(
    State(store): State<EventStore>,
    Path((event_name, contest_name)): Path<(String, String)>,
    ws: WebSocketUpgrade,
) -> Response {
    // Handshake errors carry no body: 404 for missing resources, bare 403
    // while the event has not started.
    match crate::store_call!(store.is_started(&event_name).await) {
        None => return StatusCode::NOT_FOUND.into_response(),
        Some(false) => return StatusCode::FORBIDDEN.into_response(),
        Some(true) => {}
    }

    // The replay carries every run since event creation; filtering happens
    // here by the contest codes. Runs at or after the score freeze time are
    // served as `?`: only the reveal (`runs_secret`) receives the real
    // answers.
    let Some(codes) = crate::store_call!(store.contest_codes(&event_name, &contest_name).await)
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(mut runs_rx) = crate::store_call!(store.subscribe_runs(&event_name).await) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let freeze = crate::store_call!(store.get_event(&event_name).await)
        .map(|event| event.score_freeze_time_seconds)
        .unwrap_or(0);

    ws.on_upgrade(move |socket| async move {
        let (mut sender, mut receiver) = socket.split();
        loop {
            tokio::select! {
                recv = runs_rx.recv() => {
                    match recv {
                        Ok(mut run) => {
                            if codes.is_match(&run.team_login) {
                                if run.time_seconds >= freeze {
                                    run.answer = data::event::Answer::Unknown;
                                }
                                if !send_json(&mut sender, &run).await {
                                    tracing::debug!("ws connection closed");
                                    break;
                                }
                            }
                        }
                        Err(err) => {
                            tracing::warn!(?err, "recv failed");
                            break;
                        }
                    }
                }
                // The read half of the connection: while no runs arrive, this
                // is what detects that the client went away and releases the
                // socket instead of leaking the file descriptor.
                msg = receiver.next() => {
                    if let Some(Err(err)) = msg {
                        tracing::warn!(?err, "failed reading ws messages");
                    } else {
                        tracing::debug!("ws stream ended");
                    }
                    break;
                }
            }
        }
    })
}

#[autometrics]
async fn get_runs_secret(
    State(store): State<EventStore>,
    Path((event_name, contest_name)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    // Pre-start, no key works: nothing about the contest may be served.
    if let Err(response) = contest_gate(&store, &event_name).await {
        return response;
    }

    // The key never travels in the URL (avoids leaking into access logs).
    let Some(key) = bearer_key(&headers) else {
        return invalid_key("chave do site ausente");
    };

    match crate::store_call!(store.site_by_key(&event_name, &contest_name, &key).await) {
        None => invalid_key("chave não casa com nenhum site do contest"),
        Some((site_name, _)) => match crate::store_call!(
            store
                .site_runs(&event_name, &contest_name, &site_name)
                .await
        ) {
            Some(runs) => data_json(serde_json::json!({ "runs": runs }), StatusCode::OK),
            None => not_found("evento, contest ou site não existe"),
        },
    }
}

#[autometrics]
async fn timer_ws(
    State(store): State<EventStore>,
    Path(event_name): Path<String>,
    ws: WebSocketUpgrade,
) -> Response {
    let Some(current) = crate::store_call!(store.current_timer(&event_name).await) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(mut time_rx) = crate::store_call!(store.subscribe_timer(&event_name).await) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    ws.on_upgrade(move |socket| async move {
        let (mut sender, mut receiver) = socket.split();

        // The current value is sent immediately; the stream keeps it fresh,
        // suppressing consecutive duplicates.
        let mut last: Option<PublicTimer> = Some(current);
        if !send_json(&mut sender, &current).await {
            tracing::debug!("ws connection closed");
            return;
        }
        loop {
            let time = tokio::select! {
                recv = time_rx.recv() => {
                    match recv {
                        Ok(time) => time,
                        Err(err) => {
                            tracing::warn!(?err, "recv failed");
                            break;
                        }
                    }
                }
                // The read half of the connection: detects dead clients even
                // while the clock is frozen and nothing is being written.
                msg = receiver.next() => {
                    if let Some(Err(err)) = msg {
                        tracing::warn!(?err, "failed reading ws messages");
                    } else {
                        tracing::debug!("ws stream ended");
                    }
                    break;
                }
            };
            if last.is_some_and(|previous| previous == time) {
                continue;
            }
            last = Some(time);
            if !send_json(&mut sender, &time).await {
                tracing::debug!("ws connection closed");
                break;
            }
        }
    })
}

#[autometrics]
async fn remote_control_ws(
    State(store): State<EventStore>,
    Path((event_name, contest_name, key)): Path<(String, String, String)>,
    ws: WebSocketUpgrade,
) -> Response {
    let Some(sender) = crate::store_call!(
        store
            .remote_control_sender(&event_name, &contest_name, &key)
            .await
    ) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    relay_remote_control(sender, ws).await
}
