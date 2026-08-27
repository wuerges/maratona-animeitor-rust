//! Integration tests for the public API (`doc/public-api.md`).

use actix_web::{App, http::header, test, web};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use server_v2::internal::{self, InternalToken};
use server_v2::public;
use service::event_store::{EventStore, site_key};

const TOKEN: &str = "token-de-teste";

fn auth_header() -> (&'static str, String) {
    let auth = BASE64.encode(format!("usuario:{TOKEN}"));
    (header::AUTHORIZATION.as_str(), format!("Basic {auth}"))
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

async fn app() -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    test::init_service(
        App::new()
            .app_data(web::Data::new(EventStore::new()))
            .app_data(web::Data::new(InternalToken(Some(TOKEN.to_string()))))
            .service(web::scope("internal").configure(internal::configure))
            .service(web::scope("api").configure(public::configure)),
    )
    .await
}

async fn seed(app: &impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
>) {
    let (name, auth) = auth_header();

    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .set_json(&event_body())
        .to_request();
    let resp = test::call_service(app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    let contest = serde_json::json!({
        "name": "brasil",
        "codes": ["teambr"],
        "style": "brasil",
        "ouro": 4,
        "salt": "salt-do-contest",
        "photo_url_format": "https://static.example.com/photos/{team_login}.webp"
    });
    let req = test::TestRequest::post()
        .uri("/internal/contests/ensaio/brasil")
        .insert_header((name, auth.clone()))
        .set_json(&contest)
        .to_request();
    let resp = test::call_service(app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    let site = serde_json::json!({
        "name": "fiemg",
        "codes": ["teambr"],
        "salt": "salt-do-site"
    });
    let req = test::TestRequest::post()
        .uri("/internal/sites/ensaio/brasil/fiemg")
        .insert_header((name, auth.clone()))
        .set_json(&site)
        .to_request();
    let resp = test::call_service(app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    // Event salt, so the site key mixes all three levels.
    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio/salt")
        .insert_header((name, auth.clone()))
        .set_json(&serde_json::json!({ "salt": "salt-do-evento" }))
        .to_request();
    let resp = test::call_service(app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn lists_events() {
    let app = app().await;
    seed(&app).await;

    let req = test::TestRequest::get().uri("/api/events").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"], serde_json::json!(["ensaio"]));
}

#[actix_web::test]
async fn problems_hidden_during_countdown() {
    let app = app().await;
    seed(&app).await;

    let req = test::TestRequest::get()
        .uri("/api/events/ensaio/contests/brasil/contest")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["time_seconds"], -60);
    assert!(body["data"].get("problems").is_none(), "problems must be hidden before the start");

    // The contest starts: problems are revealed.
    let (name, auth) = auth_header();
    let req = test::TestRequest::patch()
        .uri("/internal/events/ensaio/time")
        .insert_header((name, auth.clone()))
        .set_json(&serde_json::json!({ "time_seconds": 0 }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

    let req = test::TestRequest::get()
        .uri("/api/events/ensaio/contests/brasil/contest")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["problems"], serde_json::json!(["A", "B"]));
}

#[actix_web::test]
async fn config_never_leaks_salts() {
    let app = app().await;
    seed(&app).await;

    let req = test::TestRequest::get()
        .uri("/api/events/ensaio/contests/brasil/config")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
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

#[actix_web::test]
async fn secret_runs_require_the_site_key() {
    let app = app().await;
    seed(&app).await;

    let (name, auth) = auth_header();
    let runs = serde_json::json!({
        "runs": [
            { "id": 1, "team_login": "teambr001", "prob": "A", "time_seconds": 56, "answer": "Y" }
        ]
    });
    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio/runs")
        .insert_header((name, auth.clone()))
        .set_json(&runs)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

    // Without a key: 403 invalid_key.
    let req = test::TestRequest::get()
        .uri("/api/events/ensaio/contests/brasil/runs_secret")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::FORBIDDEN);
    let body: serde_json::Value = test::read_body_json(resp).await;
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
    let req = test::TestRequest::get()
        .uri("/api/events/ensaio/contests/brasil/runs_secret")
        .insert_header((header::AUTHORIZATION, format!("Bearer {key}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["runs"][0]["id"], 1);
    assert_eq!(body["data"]["runs"][0]["answer"], "Y");
}

#[actix_web::test]
async fn unknown_resources_are_not_found() {
    let app = app().await;

    for uri in [
        "/api/events/inexistente/contests/brasil/contest",
        "/api/events/inexistente/contests/brasil/config",
        "/api/events/ensaio/contests/inexistente/contest",
    ] {
        let req = test::TestRequest::get().uri(uri).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND, "{uri}");
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["errors"][0]["code"], "not_found", "{uri}");
    }
}
