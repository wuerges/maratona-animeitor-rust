//! One-shot tests for the public API (`doc/public-api.md`).
//!
//! Each test seeds the store directly (no HTTP) and performs exactly one
//! request against the endpoint under test — including the websockets, which
//! run over a real listener because axum's upgrade extractor cannot be
//! driven through `oneshot`.

use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use futures::{SinkExt, StreamExt};
use http_body_util::BodyExt;
use server_v2::{AppState, app as make_app};
use service::event_store::{ContestConfig, EventState, EventStore, Run, SiteConfig, site_key};
use tower::ServiceExt;

const TOKEN: &str = "token-de-teste";

fn event_body() -> serde_json::Value {
    serde_json::json!({
        "name": "ensaio",
        "problems": ["A", "B"],
        "teams": [
            { "login": "teambr001", "escola": "FACOM - UFMS", "nome": "Time de Teste" }
        ],
        "score_freeze_time_seconds": 2040,
        "penalty_seconds": 1200,
        "time_seconds": -60
    })
}

fn contest_body() -> serde_json::Value {
    serde_json::json!({
        "name": "brasil",
        "codes": ["teambr"],
        "salt": "salt-do-contest"
    })
}

fn site_body() -> serde_json::Value {
    serde_json::json!({
        "name": "fiemg",
        "codes": ["teambr"],
        "salt": "salt-do-site"
    })
}

fn app_for(store: EventStore) -> Router {
    make_app(AppState {
        store,
        internal_token: Some(TOKEN.to_string()),
    })
}

fn empty_request(method: Method, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

fn bearer_request(uri: &str, key: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method(Method::GET).uri(uri);
    if let Some(key) = key {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {key}"));
    }
    builder.body(Body::empty()).unwrap()
}

async fn send(app: &Router, request: Request<Body>) -> (StatusCode, serde_json::Value) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

async fn send_raw(app: &Router, request: Request<Body>) -> (StatusCode, Vec<u8>, HeaderMap) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, bytes.to_vec(), headers)
}

fn error_code(json: &serde_json::Value) -> &str {
    json["errors"][0]["code"].as_str().expect("envelope error code")
}

// Seeding (direct store calls; no HTTP).

async fn seed_event(store: &EventStore) {
    let state: EventState = serde_json::from_value(event_body()).unwrap();
    store.create_event("ensaio", state).await.unwrap();
}

async fn seed_event_salt(store: &EventStore) {
    store.set_event_salt("ensaio", Some("salt-do-evento".to_string())).await.unwrap();
}

async fn seed_contest(store: &EventStore) {
    let config: ContestConfig = serde_json::from_value(contest_body()).unwrap();
    store.create_contest("ensaio", "brasil", config).await.unwrap();
}

async fn seed_site(store: &EventStore) {
    let config: SiteConfig = serde_json::from_value(site_body()).unwrap();
    store.create_site("ensaio", "brasil", "fiemg", config).await.unwrap();
}

/// A site without a salt: no key can ever match it.
async fn seed_site_without_salt(store: &EventStore) {
    let config: SiteConfig = serde_json::from_value(serde_json::json!({
        "name": "sem-salt",
        "codes": ["teambr"]
    }))
    .unwrap();
    store.create_site("ensaio", "brasil", "sem-salt", config).await.unwrap();
}

async fn seed_run(store: &EventStore) {
    let run: Run = serde_json::from_value(serde_json::json!({
        "id": 1, "team_login": "teambr001", "prob": "A", "time_seconds": 56, "answer": "Y"
    }))
    .unwrap();
    store.add_runs("ensaio", vec![run]).await.unwrap();
}

async fn seed_started(store: &EventStore) {
    store.patch_time("ensaio", 0).await;
}

async fn seed_all(store: &EventStore) {
    seed_event(store).await;
    seed_event_salt(store).await;
    seed_contest(store).await;
    seed_site(store).await;
    seed_started(store).await;
}

// Websocket harness: axum's upgrade extractor needs a real connection.

async fn spawn_server(app: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("ws://{addr}")
}

/// Builds the handshake request from the URL (host, key, version and
/// upgrade headers included), adding `Accept-Encoding` for browser parity:
/// the compression layer must not break the upgrade.
fn ws_request(base: &str, path: &str) -> Request<()> {
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    let url = format!("{base}{path}");
    let mut request = url.as_str().into_client_request().expect("ws url is a valid request");
    request
        .headers_mut()
        .insert("Accept-Encoding", "gzip".parse().unwrap());
    request
}

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn connect(base: &str, path: &str) -> WsStream {
    let (ws, _resp) = tokio_tungstenite::connect_async(ws_request(base, path))
        .await
        .expect("websocket handshake");
    ws
}

/// Connects expecting a handshake rejection: tungstenite surfaces non-101
/// responses as `Error::Http`.
async fn connect_error(base: &str, path: &str) -> StatusCode {
    let err = tokio_tungstenite::connect_async(ws_request(base, path)).await.unwrap_err();
    match err {
        tokio_tungstenite::tungstenite::Error::Http(response) => response.status(),
        other => panic!("expected an HTTP handshake error, got {other:?}"),
    }
}

async fn next_text(ws: &mut WsStream) -> String {
    let frame = tokio::time::timeout(Duration::from_secs(5), ws.next())
        .await
        .expect("frame within 5s")
        .expect("stream alive")
        .expect("frame is a message");
    match frame {
        tokio_tungstenite::tungstenite::Message::Text(text) => text.to_string(),
        other => panic!("expected a text frame, got {other:?}"),
    }
}

// Events and contests

#[tokio::test]
async fn list_events_ok() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let (status, json) = send(&app, empty_request(Method::GET, "/api/events")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"], serde_json::json!(["ensaio"]));
}

#[tokio::test]
async fn contests_pre_start_forbidden() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let (status, json) = send(&app, empty_request(Method::GET, "/api/events/ensaio/contests")).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error_code(&json), "not_started");
}

#[tokio::test]
async fn contests_after_start() {
    let store = EventStore::new();
    seed_event(&store).await;
    seed_contest(&store).await;
    seed_started(&store).await;
    let app = app_for(store);
    let (status, json) = send(&app, empty_request(Method::GET, "/api/events/ensaio/contests")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"], serde_json::json!(["brasil"]));
}

// Contest state and config

#[tokio::test]
async fn contest_state_pre_start_forbidden() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let (status, json) = send(
        &app,
        empty_request(Method::GET, "/api/events/ensaio/contests/brasil/contest"),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error_code(&json), "not_started");
}

#[tokio::test]
async fn contest_state_after_start() {
    let store = EventStore::new();
    seed_all(&store).await;
    let app = app_for(store);
    let (status, json) = send(
        &app,
        empty_request(Method::GET, "/api/events/ensaio/contests/brasil/contest"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"]["problems"], serde_json::json!(["A", "B"]));
    assert_eq!(json["data"]["time_seconds"], 0);
}

#[tokio::test]
async fn config_pre_start_forbidden() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let (status, json) = send(
        &app,
        empty_request(Method::GET, "/api/events/ensaio/contests/brasil/config"),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error_code(&json), "not_started");
}

#[tokio::test]
async fn config_after_start_no_salts() {
    let store = EventStore::new();
    seed_all(&store).await;
    let app = app_for(store);
    let (status, bytes, _) = send_raw(
        &app,
        empty_request(Method::GET, "/api/events/ensaio/contests/brasil/config"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["data"]["name"], "brasil");
    let body = String::from_utf8(bytes).unwrap();
    assert!(!body.contains("salt"), "salts must never leak: {body}");
}

// Runs secret

#[tokio::test]
async fn runs_secret_missing_key() {
    let store = EventStore::new();
    seed_all(&store).await;
    let app = app_for(store);
    let (status, json) = send(
        &app,
        bearer_request("/api/events/ensaio/contests/brasil/runs_secret", None),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error_code(&json), "invalid_key");
}

#[tokio::test]
async fn runs_secret_wrong_key() {
    let store = EventStore::new();
    seed_all(&store).await;
    let app = app_for(store);
    let (status, json) = send(
        &app,
        bearer_request("/api/events/ensaio/contests/brasil/runs_secret", Some("chave-errada")),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error_code(&json), "invalid_key");
}

#[tokio::test]
async fn runs_secret_valid_key() {
    let store = EventStore::new();
    seed_all(&store).await;
    seed_run(&store).await;
    let app = app_for(store);
    let key = site_key(
        Some("salt-do-evento"),
        Some("salt-do-contest"),
        Some("salt-do-site"),
        "brasil",
        "fiemg",
    )
    .expect("site key with all three salts");
    let (status, json) = send(
        &app,
        bearer_request("/api/events/ensaio/contests/brasil/runs_secret", Some(&key)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"]["runs"][0]["id"], 1);
    assert_eq!(json["data"]["runs"][0]["answer"], "Y");
}

#[tokio::test]
async fn runs_secret_site_without_salt() {
    // A site without a salt derives no key: nothing may unlock it.
    let store = EventStore::new();
    seed_event(&store).await;
    seed_event_salt(&store).await;
    seed_contest(&store).await;
    seed_site_without_salt(&store).await;
    seed_started(&store).await;
    let app = app_for(store);
    let (status, json) = send(
        &app,
        bearer_request("/api/events/ensaio/contests/brasil/runs_secret", Some("qualquer")),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(error_code(&json), "invalid_key");
}

// Websockets

#[tokio::test]
async fn runs_ws_happy() {
    let store = EventStore::new();
    seed_all(&store).await;
    seed_run(&store).await;
    let app = app_for(store);
    let base = spawn_server(app).await;

    let mut ws = connect(&base, "/api/events/ensaio/contests/brasil/runs_ws").await;
    let frame = next_text(&mut ws).await;
    let run: serde_json::Value = serde_json::from_str(&frame).expect("run is JSON");
    assert_eq!(run["id"], 1);
    assert_eq!(run["team_login"], "teambr001");
    assert_eq!(run["prob"], "A");
    assert_eq!(run["time_seconds"], 56);
    assert_eq!(run["answer"], "Y");
}

#[tokio::test]
async fn runs_ws_freezes_runs_after_the_freeze_time() {
    // Runs at or after the score freeze time are served as `?`; the real
    // answers stay behind the reveal (runs_secret).
    let store = EventStore::new();
    seed_all(&store).await;
    let run: Run = serde_json::from_value(serde_json::json!({
        "id": 2, "team_login": "teambr001", "prob": "A", "time_seconds": 2040, "answer": "Y"
    }))
    .unwrap();
    store.add_runs("ensaio", vec![run]).await.unwrap();
    let app = app_for(store);
    let base = spawn_server(app).await;

    let mut ws = connect(&base, "/api/events/ensaio/contests/brasil/runs_ws").await;
    let frame = next_text(&mut ws).await;
    let run: serde_json::Value = serde_json::from_str(&frame).expect("run is JSON");
    assert_eq!(run["id"], 2);
    assert_eq!(run["time_seconds"], 2040);
    assert_eq!(run["answer"], "?");
}

#[tokio::test]
async fn runs_ws_404() {
    let app = app_for(EventStore::new());
    let base = spawn_server(app).await;
    let status = connect_error(&base, "/api/events/ensaio/contests/brasil/runs_ws").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn runs_ws_403_pre_start() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let base = spawn_server(app).await;
    let status = connect_error(&base, "/api/events/ensaio/contests/brasil/runs_ws").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn timer_ws_happy() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let base = spawn_server(app).await;

    let mut ws = connect(&base, "/api/events/ensaio/timer").await;
    let frame = next_text(&mut ws).await;
    assert_eq!(
        frame,
        serde_json::json!({ "current_time_seconds": -60, "score_freeze_time_seconds": 2040 })
            .to_string()
    );
}

#[tokio::test]
async fn timer_ws_404() {
    let app = app_for(EventStore::new());
    let base = spawn_server(app).await;
    let status = connect_error(&base, "/api/events/ensaio/timer").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn timer_ws_survives_production_layers() {
    // serve_config() wraps app() in TraceLayer + CorsLayer; the upgrade must
    // survive them (a dropped OnUpgrade extension yields a 101 whose socket
    // never speaks websocket, which the browser reports as 1006).
    use tower_http::{cors::CorsLayer, trace::TraceLayer};

    let store = EventStore::new();
    seed_event(&store).await;
    let app = make_app(AppState {
        store,
        internal_token: Some(TOKEN.to_string()),
    })
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive());
    let base = spawn_server(app).await;

    let mut ws = connect(&base, "/api/events/ensaio/timer").await;
    let frame = next_text(&mut ws).await;
    assert_eq!(
        frame,
        serde_json::json!({ "current_time_seconds": -60, "score_freeze_time_seconds": 2040 })
            .to_string()
    );
}

#[tokio::test]
async fn timer_ws_survives_browser_handshake_and_storm() {
    // Mimics Chrome's handshake (Extensions, Origin, gzip) and a reconnect
    // storm: connections opened in a tight loop, each read once, kept open,
    // then some dropped without a close frame. Every handshake response must
    // be a clean 101 without encoding headers, and every connection must
    // receive its first frame.
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tower_http::{cors::CorsLayer, trace::TraceLayer};

    let store = EventStore::new();
    seed_event(&store).await;
    let app = make_app(AppState {
        store: store.clone(),
        internal_token: Some(TOKEN.to_string()),
    })
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive());
    let base = spawn_server(app).await;

    let mut sockets = Vec::new();
    for i in 0..10 {
        let mut request = (format!("{base}/api/events/ensaio/timer"))
            .as_str()
            .into_client_request()
            .unwrap();
        let headers = request.headers_mut();
        headers.insert("Accept-Encoding", "gzip, deflate".parse().unwrap());
        headers.insert("Origin", "http://localhost:8000".parse().unwrap());
        headers.insert(
            "Sec-WebSocket-Extensions",
            "permessage-deflate; client_max_window_bits".parse().unwrap(),
        );
        let (ws, response) = tokio_tungstenite::connect_async(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
        assert!(
            !response.headers().contains_key(header::CONTENT_ENCODING),
            "the 101 must not carry a content encoding: {:?}",
            response.headers()
        );
        sockets.push(ws);
        // Each connection gets its first frame immediately.
        let frame = next_text(sockets.last_mut().unwrap()).await;
        assert_eq!(
            frame,
            serde_json::json!({ "current_time_seconds": -60, "score_freeze_time_seconds": 2040 })
                .to_string(),
            "connection {i} first frame"
        );
    }
    // Abruptly drop half of them (no close frame), like the browser's 1006.
    sockets.truncate(5);
    // The survivors still receive publishes.
    store.patch_time("ensaio", 1).await;
    let frame = next_text(sockets.first_mut().unwrap()).await;
    assert_eq!(
        frame,
        serde_json::json!({ "current_time_seconds": 1, "score_freeze_time_seconds": 2040 })
            .to_string()
    );
}

#[tokio::test]
async fn remote_control_relay() {
    let store = EventStore::new();
    seed_event(&store).await;
    seed_contest(&store).await;
    let app = app_for(store);
    let base = spawn_server(app).await;

    // A is the screen being controlled; B is the controller.
    let mut a = connect(&base, "/api/events/ensaio/contests/brasil/remote_control/chave").await;
    // Let the server-side subscription land before B sends.
    tokio::time::sleep(Duration::from_millis(100)).await;
    let mut b = connect(&base, "/api/events/ensaio/contests/brasil/remote_control/chave").await;
    b.send(tokio_tungstenite::tungstenite::Message::Text(
        r#"{"query":"?sede=fiemg"}"#.into(),
    ))
    .await
    .expect("B sends a control message");

    let frame = next_text(&mut a).await;
    let message: serde_json::Value = serde_json::from_str(&frame).expect("control message is JSON");
    assert_eq!(message["query"], "?sede=fiemg");
}

#[tokio::test]
async fn remote_control_404() {
    let app = app_for(EventStore::new());
    let base = spawn_server(app).await;
    let status =
        connect_error(&base, "/api/events/ensaio/contests/brasil/remote_control/chave").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// Metrics

#[tokio::test]
async fn metrics_ok() {
    // Never call metrics::setup() here: init() panics if called twice per
    // process, and encode_to_string() self-initializes lazily.
    let app = app_for(EventStore::new());
    let (status, body, headers) = send_raw(&app, empty_request(Method::GET, "/api/metrics")).await;
    assert_eq!(status, StatusCode::OK);
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .expect("metrics content type");
    assert!(content_type.starts_with("text/plain"), "{content_type}");
    // The API handlers carry #[autometrics]: the registry has content.
    let text = String::from_utf8(body).expect("metrics body is text");
    assert!(text.contains("function_calls"), "expected function metrics: {text}");
}
