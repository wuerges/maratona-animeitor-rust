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

use crate::AppState;
use service::event_store::{EventStore, PublicTimer};

use crate::envelope::{data_json, invalid_key, not_found, not_started, send_json};
use crate::remote_control::relay_remote_control;

pub fn router() -> Router<AppState> {
    Router::new()
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
        .route("/metrics", get(crate::metrics::get_metrics))
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

async fn list_events(State(store): State<EventStore>) -> Response {
    data_json(store.list_events().await, StatusCode::OK)
}

/// Lists the contest names of an event (landing page). Like the rest of the
/// contest scope, unavailable before the start.
async fn list_contests(
    State(store): State<EventStore>,
    Path(event_name): Path<String>,
) -> Response {
    if let Err(response) = contest_gate(&store, &event_name).await {
        return response;
    }
    match store.list_contests(&event_name).await {
        Some(contests) => {
            let mut names: Vec<String> = contests.into_iter().map(|config| config.name).collect();
            names.sort();
            data_json(names, StatusCode::OK)
        }
        None => not_found("evento não existe"),
    }
}

/// Nothing about a contest may be served before it starts: the state, the
/// config and the runs all 403 with `not_started` (the timer and the event
/// list stay available for the countdown and the landing).
async fn contest_gate(store: &EventStore, event_name: &str) -> Result<(), Response> {
    match store.is_started(event_name).await {
        None => Err(not_found("evento ou contest não existe")),
        Some(false) => Err(not_started("o evento ainda não começou")),
        Some(true) => Ok(()),
    }
}

async fn get_contest_state(
    State(store): State<EventStore>,
    Path((event_name, contest_name)): Path<(String, String)>,
) -> Response {
    if let Err(response) = contest_gate(&store, &event_name).await {
        return response;
    }
    match store.public_state(&event_name, &contest_name).await {
        Some(state) => data_json(state, StatusCode::OK),
        None => not_found("evento ou contest não existe"),
    }
}

async fn get_config(
    State(store): State<EventStore>,
    Path((event_name, contest_name)): Path<(String, String)>,
) -> Response {
    if let Err(response) = contest_gate(&store, &event_name).await {
        return response;
    }
    match store.public_config(&event_name, &contest_name).await {
        Some(config) => data_json(config, StatusCode::OK),
        None => not_found("evento ou contest não existe"),
    }
}

async fn runs_ws(
    State(store): State<EventStore>,
    Path((event_name, contest_name)): Path<(String, String)>,
    ws: WebSocketUpgrade,
) -> Response {
    // Handshake errors carry no body: 404 for missing resources, bare 403
    // while the event has not started.
    match store.is_started(&event_name).await {
        None => return StatusCode::NOT_FOUND.into_response(),
        Some(false) => return StatusCode::FORBIDDEN.into_response(),
        Some(true) => {}
    }

    // The replay carries every run since event creation; the client applies
    // the freeze. Filtering happens here by the contest codes.
    let Some(codes) = store.contest_codes(&event_name, &contest_name).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(mut runs_rx) = store.subscribe_runs(&event_name).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    ws.on_upgrade(move |socket| async move {
        let (mut sender, mut receiver) = socket.split();
        loop {
            tokio::select! {
                recv = runs_rx.recv() => {
                    match recv {
                        Ok(run) => {
                            if codes.is_match(&run.team_login) && !send_json(&mut sender, &run).await {
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

    match store.site_by_key(&event_name, &contest_name, &key).await {
        None => invalid_key("chave não casa com nenhum site do contest"),
        Some((site_name, _)) => match store.site_runs(&event_name, &contest_name, &site_name).await {
            Some(runs) => data_json(serde_json::json!({ "runs": runs }), StatusCode::OK),
            None => not_found("evento, contest ou site não existe"),
        },
    }
}

async fn timer_ws(
    State(store): State<EventStore>,
    Path(event_name): Path<String>,
    ws: WebSocketUpgrade,
) -> Response {
    let Some(current) = store.current_timer(&event_name).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(mut time_rx) = store.subscribe_timer(&event_name).await else {
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

async fn remote_control_ws(
    State(store): State<EventStore>,
    Path((event_name, contest_name, key)): Path<(String, String, String)>,
    ws: WebSocketUpgrade,
) -> Response {
    let Some(sender) = store
        .remote_control_sender(&event_name, &contest_name, &key)
        .await
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    relay_remote_control(sender, ws).await
}
