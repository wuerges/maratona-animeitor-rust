//! Integration tests for the public API (`doc/public-api.md`).

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header, HeaderName};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use http_body_util::BodyExt;
use server_v2::{AppState, app as make_app};
use service::event_store::{EventStore, site_key};
use tower::ServiceExt;

const TOKEN: &str = "token-de-teste";

fn auth_header() -> (HeaderName, String) {
    let auth = BASE64.encode(format!("usuario:{TOKEN}"));
    (header::AUTHORIZATION, format!("Basic {auth}"))
}

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

fn app() -> Router {
    make_app(AppState {
        store: EventStore::new(),
        internal_token: Some(TOKEN.to_string()),
    })
}

fn json_request(
    method: Method,
    uri: &str,
    auth: Option<(&HeaderName, String)>,
    body: &serde_json::Value,
) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some((name, value)) = auth {
        builder = builder.header(name.clone(), value.clone());
    }
    builder
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn empty_request(method: Method, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

async fn send(app: &Router, request: Request<Body>) -> (StatusCode, serde_json::Value) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

async fn seed(app: &Router) {
    let auth = auth_header();

    let req = json_request(
        Method::POST,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
        &event_body(),
    );
    let (status, _) = send(app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    let contest = serde_json::json!({
        "name": "brasil",
        "codes": ["teambr"],
        "style": "brasil",
        "ouro": 4,
        "salt": "salt-do-contest",
        "photo_url_format": "https://static.example.com/photos/{team_login}.webp"
    });
    let req = json_request(
        Method::POST,
        "/internal/contests/ensaio/brasil",
        Some((&auth.0, auth.1.clone())),
        &contest,
    );
    let (status, _) = send(app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    let site = serde_json::json!({
        "name": "fiemg",
        "codes": ["teambr"],
        "salt": "salt-do-site"
    });
    let req = json_request(
        Method::POST,
        "/internal/sites/ensaio/brasil/fiemg",
        Some((&auth.0, auth.1.clone())),
        &site,
    );
    let (status, _) = send(app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    // Event salt, so the site key mixes all three levels.
    let req = json_request(
        Method::POST,
        "/internal/events/ensaio/salt",
        Some((&auth.0, auth.1.clone())),
        &serde_json::json!({ "salt": "salt-do-evento" }),
    );
    let (status, _) = send(app, req).await;
    assert_eq!(status, StatusCode::OK);
}

async fn start(app: &Router) {
    let auth = auth_header();
    let req = json_request(
        Method::PATCH,
        "/internal/events/ensaio/time",
        Some((&auth.0, auth.1.clone())),
        &serde_json::json!({ "time_seconds": 0 }),
    );
    let (status, _) = send(app, req).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn lists_events() {
    let app = app();
    seed(&app).await;

    let (status, body) = send(&app, empty_request(Method::GET, "/api/events")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"], serde_json::json!(["ensaio"]));
}

#[tokio::test]
async fn contests_are_listed_after_start() {
    let app = app();
    seed(&app).await;

    // Before the start, the contest list is not served (no name leaks).
    let (status, body) = send(
        &app,
        empty_request(Method::GET, "/api/events/ensaio/contests"),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["errors"][0]["code"], "not_started");

    // After the start, the names are listed.
    start(&app).await;
    let (status, body) = send(
        &app,
        empty_request(Method::GET, "/api/events/ensaio/contests"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"], serde_json::json!(["brasil"]));

    // Unknown event: not found.
    let (status, _) = send(
        &app,
        empty_request(Method::GET, "/api/events/inexistente/contests"),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn pre_start_endpoints_are_forbidden() {
    let app = app();
    seed(&app).await;

    // Contest state, config and secret runs all 403 with `not_started`.
    for uri in [
        "/api/events/ensaio/contests/brasil/contest",
        "/api/events/ensaio/contests/brasil/config",
        "/api/events/ensaio/contests/brasil/runs_secret",
    ] {
        let (status, body) = send(&app, empty_request(Method::GET, uri)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{uri}");
        assert_eq!(body["errors"][0]["code"], "not_started", "{uri}");
    }

    // Even a valid site key does not unlock data before the start.
    let key = site_key(
        Some("salt-do-evento"),
        Some("salt-do-contest"),
        Some("salt-do-site"),
        "brasil",
        "fiemg",
    )
    .expect("site has a salt");
    let req = Request::builder()
        .uri("/api/events/ensaio/contests/brasil/runs_secret")
        .header(header::AUTHORIZATION, format!("Bearer {key}"))
        .body(Body::empty())
        .unwrap();
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["errors"][0]["code"], "not_started");

    // The event list stays available for the landing.
    let (status, _) = send(&app, empty_request(Method::GET, "/api/events")).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn contest_state_requires_start() {
    let app = app();
    seed(&app).await;

    // Before the start: the state is not served at all.
    let (status, body) = send(
        &app,
        empty_request(
            Method::GET,
            "/api/events/ensaio/contests/brasil/contest",
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["errors"][0]["code"], "not_started");

    // The contest starts: the state (with problems) is served.
    start(&app).await;
    let (status, body) = send(
        &app,
        empty_request(
            Method::GET,
            "/api/events/ensaio/contests/brasil/contest",
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["time_seconds"], 0);
    assert_eq!(body["data"]["problems"], serde_json::json!(["A", "B"]));
}

#[tokio::test]
async fn config_never_leaks_salts() {
    let app = app();
    seed(&app).await;
    start(&app).await;

    let (status, body) = send(
        &app,
        empty_request(Method::GET, "/api/events/ensaio/contests/brasil/config"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["name"], "brasil");
    assert_eq!(body["data"]["ouro"], 4);
    assert_eq!(
        body["data"]["photo_url_format"],
        "https://static.example.com/photos/{team_login}.webp"
    );
    assert_eq!(body["data"]["sites"][0]["name"], "fiemg");
    let text = body.to_string();
    assert!(!text.contains("salt"), "no salt may leak: {text}");
    assert!(!text.contains("key"), "no derived key may leak: {text}");
}

#[tokio::test]
async fn secret_runs_require_the_site_key() {
    let app = app();
    seed(&app).await;

    let auth = auth_header();
    let runs = serde_json::json!({
        "runs": [
            { "id": 1, "team_login": "teambr001", "prob": "A", "time_seconds": 56, "answer": "Y" }
        ]
    });
    let req = json_request(
        Method::POST,
        "/internal/events/ensaio/runs",
        Some((&auth.0, auth.1.clone())),
        &runs,
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);

    // Secret runs are only served after the start.
    start(&app).await;

    // Without a key: 403 invalid_key.
    let (status, body) = send(
        &app,
        empty_request(
            Method::GET,
            "/api/events/ensaio/contests/brasil/runs_secret",
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["errors"][0]["code"], "invalid_key");

    // With the derived site key: the site's runs.
    let key = site_key(
        Some("salt-do-evento"),
        Some("salt-do-contest"),
        Some("salt-do-site"),
        "brasil",
        "fiemg",
    )
    .expect("site has a salt");
    let req = Request::builder()
        .uri("/api/events/ensaio/contests/brasil/runs_secret")
        .header(header::AUTHORIZATION, format!("Bearer {key}"))
        .body(Body::empty())
        .unwrap();
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["runs"][0]["id"], 1);
    assert_eq!(body["data"]["runs"][0]["answer"], "Y");
}

#[tokio::test]
async fn unknown_resources_are_not_found() {
    let app = app();

    for uri in [
        "/api/events/inexistente/contests/brasil/contest",
        "/api/events/inexistente/contests/brasil/config",
        "/api/events/ensaio/contests/inexistente/contest",
    ] {
        let (status, body) = send(&app, empty_request(Method::GET, uri)).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
        assert_eq!(body["errors"][0]["code"], "not_found", "{uri}");
    }
}
