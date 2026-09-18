//! One-shot tests for the internal API (`doc/event-api.md`).
//!
//! Each test seeds the store directly (no HTTP) and performs exactly one
//! request against the endpoint under test. The scenario tests in
//! `internal_api.rs` cover multi-request flows; these pin every
//! (route, method) pair individually.

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderName, Method, Request, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use http_body_util::BodyExt;
use server_v2::{AppState, app as make_app};
use service::event_store::{ContestConfig, EventState, EventStore, SiteConfig};
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
        public_url: "https://example.com".parse().unwrap(),
        store,
        internal_tokens: std::sync::Arc::new(std::collections::HashMap::from([(
            "usuario".to_string(),
            TOKEN.to_string(),
        )])),
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

fn empty_request(method: Method, uri: &str, auth: Option<(&HeaderName, String)>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some((name, value)) = auth {
        builder = builder.header(name.clone(), value.clone());
    }
    builder.body(Body::empty()).unwrap()
}

fn raw_json_request(
    method: Method,
    uri: &str,
    auth: Option<(&HeaderName, String)>,
    payload: &str,
) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some((name, value)) = auth {
        builder = builder.header(name.clone(), value.clone());
    }
    builder
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap()
}

async fn send(app: &Router, request: Request<Body>) -> (StatusCode, serde_json::Value) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

async fn send_bytes(app: &Router, request: Request<Body>) -> (StatusCode, Vec<u8>) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, bytes.to_vec())
}

fn error_code(json: &serde_json::Value) -> &str {
    json["errors"][0]["code"]
        .as_str()
        .expect("envelope error code")
}

// Seeding (direct store calls; no HTTP).

async fn seed_event(store: &EventStore) {
    let state: EventState = serde_json::from_value(event_body()).unwrap();
    store.create_event("ensaio", state).await.unwrap();
}

async fn seed_event_salt(store: &EventStore) {
    store
        .set_event_salt("ensaio", Some("salt-do-evento".to_string()))
        .await
        .unwrap();
}

async fn seed_contest(store: &EventStore) {
    let config: ContestConfig = serde_json::from_value(contest_body()).unwrap();
    store
        .create_contest("ensaio", "brasil", config)
        .await
        .unwrap();
}

async fn seed_site(store: &EventStore) {
    let config: SiteConfig = serde_json::from_value(site_body()).unwrap();
    store
        .create_site("ensaio", "brasil", "fiemg", config)
        .await
        .unwrap();
}

async fn seed_all(store: &EventStore) {
    seed_event(store).await;
    seed_event_salt(store).await;
    seed_contest(store).await;
    seed_site(store).await;
}

// Auth

#[tokio::test]
async fn rejects_missing_credentials() {
    let app = app_for(EventStore::new());
    let (status, json) = send(&app, empty_request(Method::GET, "/internal/events", None)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(error_code(&json), "unauthorized");
}

// Events: list/get

#[tokio::test]
async fn list_events_empty() {
    let app = app_for(EventStore::new());
    let auth = auth_header();
    let (status, json) = send(
        &app,
        empty_request(
            Method::GET,
            "/internal/events",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"], serde_json::json!([]));
}

#[tokio::test]
async fn list_events() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        empty_request(
            Method::GET,
            "/internal/events",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"], serde_json::json!(["ensaio"]));
}

#[tokio::test]
async fn get_event() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        empty_request(
            Method::GET,
            "/internal/events/ensaio",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"]["name"], "ensaio");
    assert_eq!(json["data"]["time_seconds"], -60);
}

#[tokio::test]
async fn get_event_404() {
    let app = app_for(EventStore::new());
    let auth = auth_header();
    let (status, json) = send(
        &app,
        empty_request(
            Method::GET,
            "/internal/events/inexistente",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

// Events: create/put/delete

#[tokio::test]
async fn create_event() {
    let app = app_for(EventStore::new());
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/events/ensaio",
            Some((&auth.0, auth.1.clone())),
            &event_body(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["data"]["name"], "ensaio");
}

#[tokio::test]
async fn create_event_conflict() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/events/ensaio",
            Some((&auth.0, auth.1.clone())),
            &event_body(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error_code(&json), "conflict");
}

#[tokio::test]
async fn create_event_bad_json() {
    let app = app_for(EventStore::new());
    let auth = auth_header();
    let (status, json) = send(
        &app,
        raw_json_request(
            Method::POST,
            "/internal/events/ensaio",
            Some((&auth.0, auth.1.clone())),
            "{ não é json",
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&json), "invalid_json");
}

#[tokio::test]
async fn put_event() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let mut body = event_body();
    body["time_seconds"] = serde_json::json!(300);
    let (status, json) = send(
        &app,
        json_request(
            Method::PUT,
            "/internal/events/ensaio",
            Some((&auth.0, auth.1.clone())),
            &body,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"]["time_seconds"], 300);
}

#[tokio::test]
async fn put_event_404() {
    let app = app_for(EventStore::new());
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::PUT,
            "/internal/events/inexistente",
            Some((&auth.0, auth.1.clone())),
            &event_body(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

#[tokio::test]
async fn delete_event() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, _) = send(
        &app,
        empty_request(
            Method::DELETE,
            "/internal/events/ensaio",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn delete_event_404() {
    let app = app_for(EventStore::new());
    let auth = auth_header();
    let (status, json) = send(
        &app,
        empty_request(
            Method::DELETE,
            "/internal/events/inexistente",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

// Contests and sites: list

#[tokio::test]
async fn list_contests() {
    let store = EventStore::new();
    seed_event(&store).await;
    seed_contest(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        empty_request(
            Method::GET,
            "/internal/events/ensaio/contests",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"][0]["name"], "brasil");
    assert_eq!(json["data"][0]["salt"], "salt-do-contest");
}

#[tokio::test]
async fn list_contests_404() {
    let app = app_for(EventStore::new());
    let auth = auth_header();
    let (status, json) = send(
        &app,
        empty_request(
            Method::GET,
            "/internal/events/inexistente/contests",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

#[tokio::test]
async fn list_sites() {
    let store = EventStore::new();
    seed_event(&store).await;
    seed_contest(&store).await;
    seed_site(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        empty_request(
            Method::GET,
            "/internal/events/ensaio/contests/brasil/sites",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"][0]["name"], "fiemg");
    assert_eq!(json["data"][0]["salt"], "salt-do-site");
}

#[tokio::test]
async fn list_sites_404() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        empty_request(
            Method::GET,
            "/internal/events/ensaio/contests/brasil/sites",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

// Time and runs

#[tokio::test]
async fn patch_time() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::PATCH,
            "/internal/events/ensaio/time",
            Some((&auth.0, auth.1.clone())),
            &serde_json::json!({ "time_seconds": 123 }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"]["time_seconds"], 123);
}

#[tokio::test]
async fn patch_time_404() {
    let app = app_for(EventStore::new());
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::PATCH,
            "/internal/events/inexistente/time",
            Some((&auth.0, auth.1.clone())),
            &serde_json::json!({ "time_seconds": 123 }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

#[tokio::test]
async fn post_runs() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let body = serde_json::json!({
        "runs": [
            { "id": 1, "team_login": "teambr001", "prob": "A", "time_seconds": 56, "answer": "Y" },
            { "id": 2, "team_login": "teambr001", "prob": "B", "time_seconds": 139, "answer": "N" }
        ]
    });
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/events/ensaio/runs",
            Some((&auth.0, auth.1.clone())),
            &body,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"]["added"], 2);
    assert_eq!(json["data"]["updated"], 0);
}

#[tokio::test]
async fn post_runs_warns_on_unknown_teams() {
    // Runs from teams not in the event (e.g. judge users of the MOJ feed)
    // are skipped and reported in the warnings array; the batch succeeds.
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let body = serde_json::json!({
        "runs": [
            { "id": 1, "team_login": "desconhecido", "prob": "A", "time_seconds": 56, "answer": "Y" }
        ]
    });
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/events/ensaio/runs",
            Some((&auth.0, auth.1.clone())),
            &body,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"]["added"], 0);
    assert_eq!(json["data"]["updated"], 0);
    assert_eq!(json["warnings"][0]["code"], "unknown_team");
}

#[tokio::test]
async fn post_runs_invalid_prob() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let body = serde_json::json!({
        "runs": [
            { "id": 1, "team_login": "teambr001", "prob": "Z", "time_seconds": 56, "answer": "Y" }
        ]
    });
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/events/ensaio/runs",
            Some((&auth.0, auth.1.clone())),
            &body,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&json), "invalid_value");
}

#[tokio::test]
async fn post_runs_404() {
    let app = app_for(EventStore::new());
    let auth = auth_header();
    let body = serde_json::json!({
        "runs": [
            { "id": 1, "team_login": "teambr001", "prob": "A", "time_seconds": 56, "answer": "Y" }
        ]
    });
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/events/inexistente/runs",
            Some((&auth.0, auth.1.clone())),
            &body,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

#[tokio::test]
async fn delete_runs() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, _) = send(
        &app,
        empty_request(
            Method::DELETE,
            "/internal/events/ensaio/runs",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn delete_runs_404() {
    let app = app_for(EventStore::new());
    let auth = auth_header();
    let (status, json) = send(
        &app,
        empty_request(
            Method::DELETE,
            "/internal/events/inexistente/runs",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

// Event salt

#[tokio::test]
async fn event_salt_explicit() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/events/ensaio/salt",
            Some((&auth.0, auth.1.clone())),
            &serde_json::json!({ "salt": "meu-salt" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"]["salt"], "meu-salt");
}

#[tokio::test]
async fn event_salt_generated() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        empty_request(
            Method::POST,
            "/internal/events/ensaio/salt",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let salt = json["data"]["salt"].as_str().expect("generated salt");
    assert_eq!(salt.len(), 32);
    assert!(salt.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[tokio::test]
async fn event_salt_404() {
    let app = app_for(EventStore::new());
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/events/inexistente/salt",
            Some((&auth.0, auth.1.clone())),
            &serde_json::json!({ "salt": "x" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

// Contests: create/put/delete

#[tokio::test]
async fn create_contest() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/contests/ensaio/brasil",
            Some((&auth.0, auth.1.clone())),
            &contest_body(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["data"]["name"], "brasil");
}

#[tokio::test]
async fn create_contest_conflict() {
    let store = EventStore::new();
    seed_event(&store).await;
    seed_contest(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/contests/ensaio/brasil",
            Some((&auth.0, auth.1.clone())),
            &contest_body(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error_code(&json), "conflict");
}

#[tokio::test]
async fn create_contest_404() {
    let app = app_for(EventStore::new());
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/contests/ensaio/brasil",
            Some((&auth.0, auth.1.clone())),
            &contest_body(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

#[tokio::test]
async fn create_contest_invalid_regex() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let body = serde_json::json!({ "name": "ruim", "codes": ["("] });
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/contests/ensaio/ruim",
            Some((&auth.0, auth.1.clone())),
            &body,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&json), "invalid_regex");
}

#[tokio::test]
async fn put_contest() {
    let store = EventStore::new();
    seed_event(&store).await;
    seed_contest(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let body = serde_json::json!({ "name": "brasil", "codes": ["teambr"], "ouro": 7 });
    let (status, json) = send(
        &app,
        json_request(
            Method::PUT,
            "/internal/contests/ensaio/brasil",
            Some((&auth.0, auth.1.clone())),
            &body,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"]["ouro"], 7);
}

#[tokio::test]
async fn put_contest_404() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::PUT,
            "/internal/contests/ensaio/inexistente",
            Some((&auth.0, auth.1.clone())),
            &contest_body(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

#[tokio::test]
async fn delete_contest() {
    let store = EventStore::new();
    seed_event(&store).await;
    seed_contest(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, _) = send(
        &app,
        empty_request(
            Method::DELETE,
            "/internal/contests/ensaio/brasil",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn delete_contest_404() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        empty_request(
            Method::DELETE,
            "/internal/contests/ensaio/inexistente",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

#[tokio::test]
async fn contest_salt_explicit() {
    let store = EventStore::new();
    seed_event(&store).await;
    seed_contest(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/contests/ensaio/brasil/salt",
            Some((&auth.0, auth.1.clone())),
            &serde_json::json!({ "salt": "meu-salt" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"]["salt"], "meu-salt");
}

#[tokio::test]
async fn contest_salt_404() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/contests/ensaio/inexistente/salt",
            Some((&auth.0, auth.1.clone())),
            &serde_json::json!({ "salt": "x" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

// Sites: create/put/delete

#[tokio::test]
async fn create_site() {
    let store = EventStore::new();
    seed_event(&store).await;
    seed_contest(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/sites/ensaio/brasil/fiemg",
            Some((&auth.0, auth.1.clone())),
            &site_body(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(json["data"]["name"], "fiemg");
}

#[tokio::test]
async fn create_site_conflict() {
    let store = EventStore::new();
    seed_all(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/sites/ensaio/brasil/fiemg",
            Some((&auth.0, auth.1.clone())),
            &site_body(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error_code(&json), "conflict");
}

#[tokio::test]
async fn create_site_404() {
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/sites/ensaio/brasil/fiemg",
            Some((&auth.0, auth.1.clone())),
            &site_body(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

#[tokio::test]
async fn put_site() {
    let store = EventStore::new();
    seed_all(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let body = serde_json::json!({ "name": "fiemg", "codes": ["teambr", "outros"] });
    let (status, json) = send(
        &app,
        json_request(
            Method::PUT,
            "/internal/sites/ensaio/brasil/fiemg",
            Some((&auth.0, auth.1.clone())),
            &body,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        json["data"]["codes"],
        serde_json::json!(["teambr", "outros"])
    );
}

#[tokio::test]
async fn put_site_404() {
    let store = EventStore::new();
    seed_event(&store).await;
    seed_contest(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::PUT,
            "/internal/sites/ensaio/brasil/inexistente",
            Some((&auth.0, auth.1.clone())),
            &site_body(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

#[tokio::test]
async fn delete_site() {
    let store = EventStore::new();
    seed_all(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, _) = send(
        &app,
        empty_request(
            Method::DELETE,
            "/internal/sites/ensaio/brasil/fiemg",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn delete_site_404() {
    let store = EventStore::new();
    seed_event(&store).await;
    seed_contest(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        empty_request(
            Method::DELETE,
            "/internal/sites/ensaio/brasil/inexistente",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

#[tokio::test]
async fn site_salt_explicit() {
    let store = EventStore::new();
    seed_all(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/sites/ensaio/brasil/fiemg/salt",
            Some((&auth.0, auth.1.clone())),
            &serde_json::json!({ "salt": "meu-salt" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["data"]["salt"], "meu-salt");
}

#[tokio::test]
async fn site_salt_404() {
    let store = EventStore::new();
    seed_event(&store).await;
    seed_contest(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    let (status, json) = send(
        &app,
        json_request(
            Method::POST,
            "/internal/sites/ensaio/brasil/inexistente/salt",
            Some((&auth.0, auth.1.clone())),
            &serde_json::json!({ "salt": "x" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&json), "not_found");
}

#[tokio::test]
async fn metrics_ok() {
    // The recorder must be installed BEFORE instrumented handlers run
    // (samples recorded with no recorder are dropped); this is the only
    // test touching the global exporter, so the init cannot double-fire.
    server_v2::metrics::setup();
    let store = EventStore::new();
    seed_event(&store).await;
    let app = app_for(store);
    let auth = auth_header();
    // One instrumented request first, so the registry has samples.
    let _ = send(
        &app,
        empty_request(
            Method::GET,
            "/internal/events",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    let (status, bytes) = send_bytes(
        &app,
        empty_request(
            Method::GET,
            "/internal/metrics",
            Some((&auth.0, auth.1.clone())),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let text = String::from_utf8(bytes).expect("metrics body is text");
    assert!(
        text.contains("function_calls"),
        "expected function metrics: {text}"
    );
}
