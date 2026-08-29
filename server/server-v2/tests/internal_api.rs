//! Integration tests for the internal API (`doc/event-api.md`).

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header, HeaderName};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use http_body_util::BodyExt;
use server_v2::{AppState, app as make_app};
use service::event_store::EventStore;
use tower::ServiceExt;

const TOKEN: &str = "token-de-teste";

fn basic_header() -> (HeaderName, String) {
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

#[tokio::test]
async fn rejects_missing_credentials() {
    let app = app();
    let req = Request::builder()
        .uri("/internal/events/ensaio")
        .body(Body::empty())
        .unwrap();
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["errors"][0]["code"], "unauthorized");
}

#[tokio::test]
async fn rejects_wrong_token() {
    let app = app();
    let auth = BASE64.encode("usuario:senha-errada");
    let req = Request::builder()
        .uri("/internal/events/ensaio")
        .header(header::AUTHORIZATION, format!("Basic {auth}"))
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn event_lifecycle() {
    let app = app();
    let auth = basic_header();

    // Create.
    let req = json_request(
        Method::POST,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
        &event_body(),
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["data"]["name"], "ensaio");
    assert_eq!(body["data"]["time_seconds"], -60);

    // Read.
    let req = empty_request(
        Method::GET,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);

    // Duplicate create conflicts.
    let req = json_request(
        Method::POST,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
        &event_body(),
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["errors"][0]["code"], "conflict");

    // Delete.
    let req = empty_request(
        Method::DELETE,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Delete again: not found.
    let req = empty_request(
        Method::DELETE,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn bad_json_and_missing_field() {
    let app = app();
    let auth = basic_header();

    let req = raw_json_request(
        Method::POST,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
        "{ não é json",
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["errors"][0]["code"], "invalid_json");

    let missing = serde_json::json!({
        "name": "ensaio",
        "problems": ["A"],
        "score_freeze_time_seconds": 2040
        // teams e penalty_seconds ausentes
    });
    let req = json_request(
        Method::POST,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
        &missing,
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["errors"][0]["code"], "missing_field");
}

#[tokio::test]
async fn patch_time_allows_negative_countdown() {
    let app = app();
    let auth = basic_header();

    let req = json_request(
        Method::POST,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
        &event_body(),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    let req = json_request(
        Method::PATCH,
        "/internal/events/ensaio/time",
        Some((&auth.0, auth.1.clone())),
        &serde_json::json!({ "time_seconds": -120 }),
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["time_seconds"], -120);
}

#[tokio::test]
async fn runs_are_added_and_corrected() {
    let app = app();
    let auth = basic_header();

    let req = json_request(
        Method::POST,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
        &event_body(),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    let runs = serde_json::json!({
        "runs": [
            { "id": 1, "team_login": "teambr001", "prob": "A", "time_seconds": 56, "answer": "N" },
            { "id": 2, "team_login": "teambr001", "prob": "B", "time_seconds": 139, "answer": "Y" }
        ]
    });
    let req = json_request(
        Method::POST,
        "/internal/events/ensaio/runs",
        Some((&auth.0, auth.1.clone())),
        &runs,
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["added"], 2);
    assert_eq!(body["data"]["updated"], 0);

    // Same id: judge correction, last value wins.
    let correction = serde_json::json!({
        "runs": [
            { "id": 1, "team_login": "teambr001", "prob": "A", "time_seconds": 56, "answer": "Y" }
        ]
    });
    let req = json_request(
        Method::POST,
        "/internal/events/ensaio/runs",
        Some((&auth.0, auth.1.clone())),
        &correction,
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["added"], 0);
    assert_eq!(body["data"]["updated"], 1);

    // Unknown team/prob → 400 invalid_value.
    let bad = serde_json::json!({
        "runs": [
            { "id": 3, "team_login": "desconhecido", "prob": "A", "time_seconds": 1, "answer": "Y" }
        ]
    });
    let req = json_request(
        Method::POST,
        "/internal/events/ensaio/runs",
        Some((&auth.0, auth.1.clone())),
        &bad,
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["errors"][0]["code"], "invalid_value");
}

#[tokio::test]
async fn empty_contest_name_is_rejected() {
    let app = app();
    let auth = basic_header();

    let req = json_request(
        Method::POST,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
        &event_body(),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    // There is no default contest: empty names are rejected on create.
    let contest = serde_json::json!({
        "name": "",
        "codes": ["teambr"]
    });
    let req = json_request(
        Method::POST,
        "/internal/contests/ensaio/brasil",
        Some((&auth.0, auth.1.clone())),
        &contest,
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["errors"][0]["code"], "invalid_value");

    // ... and on replace of an existing contest.
    let named = serde_json::json!({ "name": "brasil", "codes": ["teambr"] });
    let req = json_request(
        Method::POST,
        "/internal/contests/ensaio/brasil",
        Some((&auth.0, auth.1.clone())),
        &named,
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    let req = json_request(
        Method::PUT,
        "/internal/contests/ensaio/brasil",
        Some((&auth.0, auth.1.clone())),
        &serde_json::json!({ "name": "", "codes": ["teambr"] }),
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["errors"][0]["code"], "invalid_value");
}

#[tokio::test]
async fn named_contest_and_site_lifecycle() {
    let app = app();
    let auth = basic_header();

    let req = json_request(
        Method::POST,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
        &event_body(),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    let contest = serde_json::json!({
        "name": "brasil",
        "codes": ["teambr"]
    });
    let req = json_request(
        Method::POST,
        "/internal/contests/ensaio/brasil",
        Some((&auth.0, auth.1.clone())),
        &contest,
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    let site = serde_json::json!({
        "name": "fiemg",
        "codes": ["teambr"]
    });
    let req = json_request(
        Method::POST,
        "/internal/sites/ensaio/brasil/fiemg",
        Some((&auth.0, auth.1.clone())),
        &site,
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    // Deleting the contest removes its sites.
    let req = empty_request(
        Method::DELETE,
        "/internal/contests/ensaio/brasil",
        Some((&auth.0, auth.1.clone())),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let req = empty_request(
        Method::GET,
        "/internal/events/ensaio/contests/brasil/sites",
        Some((&auth.0, auth.1.clone())),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn salt_endpoints_generate_when_body_is_absent() {
    let app = app();
    let auth = basic_header();

    let req = json_request(
        Method::POST,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
        &event_body(),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    let contest = serde_json::json!({ "name": "brasil", "codes": ["teambr"] });
    let req = json_request(
        Method::POST,
        "/internal/contests/ensaio/brasil",
        Some((&auth.0, auth.1.clone())),
        &contest,
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    // Explicit salt is returned as-is.
    let req = json_request(
        Method::POST,
        "/internal/contests/ensaio/brasil/salt",
        Some((&auth.0, auth.1.clone())),
        &serde_json::json!({ "salt": "meu-salt" }),
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["salt"], "meu-salt");

    // Missing body generates a random salt.
    let req = empty_request(
        Method::POST,
        "/internal/events/ensaio/salt",
        Some((&auth.0, auth.1.clone())),
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    let salt = body["data"]["salt"].as_str().expect("salt gerado");
    assert_eq!(salt.len(), 32);
    assert!(salt.chars().all(|c| c.is_ascii_alphanumeric()));

    // Salt on a missing site: not found.
    let req = empty_request(
        Method::POST,
        "/internal/sites/ensaio/brasil/inexistente/salt",
        Some((&auth.0, auth.1.clone())),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn unknown_answer_is_invalid_value() {
    let app = app();
    let auth = basic_header();

    let req = json_request(
        Method::POST,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
        &event_body(),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    let bad = serde_json::json!({
        "runs": [
            { "id": 1, "team_login": "teambr001", "prob": "A", "time_seconds": 1, "answer": "Z" }
        ]
    });
    let req = json_request(
        Method::POST,
        "/internal/events/ensaio/runs",
        Some((&auth.0, auth.1.clone())),
        &bad,
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["errors"][0]["code"], "invalid_value");
}

#[tokio::test]
async fn salt_endpoints_accept_empty_object_and_empty_salt() {
    let app = app();
    let auth = basic_header();

    let req = json_request(
        Method::POST,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
        &event_body(),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    // Empty object body generates a random salt.
    let req = json_request(
        Method::POST,
        "/internal/events/ensaio/salt",
        Some((&auth.0, auth.1.clone())),
        &serde_json::json!({}),
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    let salt = body["data"]["salt"].as_str().expect("salt gerado");
    assert_eq!(salt.len(), 32);

    // Empty salt value also generates a random salt.
    let req = json_request(
        Method::POST,
        "/internal/events/ensaio/salt",
        Some((&auth.0, auth.1.clone())),
        &serde_json::json!({ "salt": "" }),
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    let salt = body["data"]["salt"].as_str().expect("salt gerado");
    assert_eq!(salt.len(), 32);
}

#[tokio::test]
async fn read_endpoints_list_events_contests_and_sites() {
    let app = app();
    let auth = basic_header();

    // Empty store: events list is empty.
    let req = empty_request(
        Method::GET,
        "/internal/events",
        Some((&auth.0, auth.1.clone())),
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"], serde_json::json!([]));

    let req = json_request(
        Method::POST,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
        &event_body(),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    let contest = serde_json::json!({
        "name": "brasil",
        "codes": ["teambr"],
        "salt": "salt-do-contest"
    });
    let req = json_request(
        Method::POST,
        "/internal/contests/ensaio/brasil",
        Some((&auth.0, auth.1.clone())),
        &contest,
    );
    let (status, _) = send(&app, req).await;
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
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    // Events.
    let req = empty_request(
        Method::GET,
        "/internal/events",
        Some((&auth.0, auth.1.clone())),
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"], serde_json::json!(["ensaio"]));

    // Contests, with salts (internal scope).
    let req = empty_request(
        Method::GET,
        "/internal/events/ensaio/contests",
        Some((&auth.0, auth.1.clone())),
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"][0]["name"], "brasil");
    assert_eq!(body["data"][0]["salt"], "salt-do-contest");

    // Sites, with salts.
    let req = empty_request(
        Method::GET,
        "/internal/events/ensaio/contests/brasil/sites",
        Some((&auth.0, auth.1.clone())),
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"][0]["name"], "fiemg");
    assert_eq!(body["data"][0]["salt"], "salt-do-site");

    // Missing event/contest: not found.
    let req = empty_request(
        Method::GET,
        "/internal/events/inexistente/contests",
        Some((&auth.0, auth.1.clone())),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let req = empty_request(
        Method::GET,
        "/internal/events/ensaio/contests/inexistente/sites",
        Some((&auth.0, auth.1.clone())),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn invalid_regex_is_rejected() {
    let app = app();
    let auth = basic_header();

    let req = json_request(
        Method::POST,
        "/internal/events/ensaio",
        Some((&auth.0, auth.1.clone())),
        &event_body(),
    );
    let (status, _) = send(&app, req).await;
    assert_eq!(status, StatusCode::CREATED);

    let bad = serde_json::json!({ "name": "ruim", "codes": ["("] });
    let req = json_request(
        Method::POST,
        "/internal/contests/ensaio/ruim",
        Some((&auth.0, auth.1.clone())),
        &bad,
    );
    let (status, body) = send(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["errors"][0]["code"], "invalid_regex");
}
